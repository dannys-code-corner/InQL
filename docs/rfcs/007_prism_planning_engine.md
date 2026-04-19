# InQL RFC 007: Prism logical planning and optimization engine

- **Status:** In Progress
- **Created:** 2026-04-02
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 001 (dataset types and carriers — Prism-backed carriers must remain consistent with `DataSet[T]` semantics)
  - InQL RFC 002 (Apache Substrait integration — Substrait remains the normative emitted contract at the boundary)
  - InQL RFC 003 (`query {}` — lowers through Prism-managed logical work before Substrait emission)
  - InQL RFC 004 (execution context — session executes Prism-backed plans but does not define Prism)
  - InQL RFC 005 (optional pipe-forward — must stay Prism-consistent with equivalent surfaces)
- **Issue:** [InQL #16](https://github.com/dannys-code-corner/InQL/issues/16)
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines **Prism** as InQL's immutable internal logical planning and optimization engine. Prism owns persistent plan storage, cheap branching through structural sharing, lineage-preserving rewrites, and logical optimization prior to Substrait emission or session execution. Prism is an **internal planning substrate**, not the normative interchange contract: **Apache Substrait** remains the boundary format per InQL RFC 002. `LazyFrame`, `DataFrame`, and `DataStream` are carrier experiences over Prism-managed plan state; `Session` and `SessionContext` bind and execute those plans per InQL RFC 004.

RFC 007 is the design and implementation record for the first Prism adoption slice. Optimizer-boundary ownership is further clarified by [InQL RFC 008](008_optimizer_boundary_stats_cbo_aqe.md): Prism owns immutable authored state, lineage-preserving logical work, and internal optimized views, while RFC 008 narrows the split with `Session` around backend-facing statistics, physical planning, and adaptive execution concerns. Follow-on RFC 007 hardening includes an Incan-native typed store-id allocator (`static` + `newtype`) and cross-store adoption dedup for equivalent reachable RHS nodes; this remains internal Prism substrate work and does not expand RFC 008 scope.

## Motivation

InQL already has a strong external story around typed carriers, Substrait emission, and the execution boundary, but it lacks a dedicated specification for the internal planning layer that sits between authored logic and emitted plans. Without that layer being named and scoped, plan construction, optimization, lineage, interactive behavior, and future explain/debug tooling risk becoming an accidental mix of implementation details spread across InQL RFC 001, InQL RFC 002, and InQL RFC 004.

Prism gives that layer a home. It lets InQL say clearly that:

- authored transformations build immutable logical plans
- carriers stay cheap by sharing planning state instead of cloning whole plans
- optimization is a first-class responsibility, not an incidental backend side effect
- lineage must survive rewrites so optimized plans remain explainable

This matters for more than simple query lowering. Complex multi-hop pipelines, future interactive environments, and future explain/debug tooling all benefit from a stable definition of what the internal plan engine is allowed and required to do.

## Goals

- Define **Prism** as the immutable logical planning engine for InQL.
- Specify Prism's core responsibilities: persistent plan storage, logical optimization, lineage preservation, and preparation for Substrait emission.
- Clarify the relationship between Prism and InQL carriers (`LazyFrame`, `DataFrame`, `DataStream`, `DataSet`).
- Clarify the relationship between Prism and sibling boundaries: Substrait at interchange boundaries and `Session` / `SessionContext` at execution boundaries.
- Require that Prism-backed plan construction remain cheap through structural sharing rather than deep-cloning carrier state.
- Define the conceptual distinction between authored plan state and optimized plan state without over-constraining the final implementation.

## Non-Goals

- Replacing Apache Substrait as InQL's normative emitted logical contract — that remains InQL RFC 002.
- Defining physical execution behavior, backend binding, or secret management — that remains outside Prism and is scoped by InQL RFC 004 and surrounding operational layers.
- Defining new author-facing query syntax — Prism is an internal planning engine, not a new language surface.
- Forcing one exact in-memory data structure implementation for authored and optimized plan state.
- Promising Prism as a general-purpose platform beyond InQL today. This RFC scopes Prism normatively to InQL; future extraction remains a possible consequence of a clean boundary, not a current requirement.

## Guide-level explanation

From an author's point of view, Prism is not something they use directly. Authors work with InQL carriers such as `LazyFrame[T]`, `DataFrame[T]`, and (later) `DataStream[T]`. Those carriers build or operate over logical work that Prism stores and optimizes internally.

```incan
orders = session.table[Order]("orders")?
cutoff = ...  # some appropriate value

high_value = orders.filter(.amount > 1000)
recent = orders.filter(.created_at >= cutoff)

summary = high_value.join(recent, on=.order_id)
```

The important user-visible behavior is:

- each transformation returns a new carrier
- earlier carriers still exist unchanged
- branching from a shared base plan is cheap
- execution still belongs to the session boundary

Prism is the reason this can work efficiently. It stores the shared authored planning state, allows both `high_value` and `recent` to branch from the same base plan, and may derive optimized views of that state before the plan is emitted to Substrait or executed by a session.

Prism should be thought of as the internal engine that **thinks** about the plan. Substrait is how the plan is **communicated** at the boundary. Session is how the plan is **executed**.

## Reference-level explanation

### Prism role

Prism is the internal logical planning and optimization substrate for InQL.

Prism **must**:

- store logical relational author intent in persistent plan state
- support cheap plan branching through structural sharing
- preserve lineage across plan construction and optimization
- provide an optimized logical view for lowering and execution

Prism **must not**:

- become the normative interchange format
- require destructive mutation of prior authored history
- own physical execution or backend-specific binding

### Relationship to carriers

`LazyFrame[T]`, `DataFrame[T]`, and `DataStream[T]` **may** present different user-facing execution behavior, but they **should** be able to share Prism-managed planning state.

Carrier operations that extend logical work **must** produce new logical tips rather than mutating prior history. Implementations **should** make returned carriers cheap immutable handles over shared Prism-managed state.

### Relationship to Substrait

Prism is internal (for now). Apache Substrait remains the normative boundary contract.

The relationship is:

- Prism = internal logical planning, lineage, and optimization
- Substrait = emitted logical interchange contract

An implementation **may** use Prism-native node kinds or derived optimized views internally, but emitted plans that claim conformance **must** still follow InQL RFC 002.

### Relationship to session execution

Prism does not execute plans. `Session` / `SessionContext` own execution.

Execution-oriented flows **must** treat Prism as an input to lowering and execution, not as the executor itself. Session-backed operations may request optimized views from Prism before emission or execution, but the existence of Prism **must not** collapse the execution boundary defined in InQL RFC 004.

### Authored state vs optimized state

Prism **should** conceptually distinguish between:

- **authored plan state**: persistent construction history closest to user intent
- **optimized plan state**: semantically equivalent rewritten state used for lowering or execution
- **lineage metadata**: mappings from optimized state back to authored history

This distinction is normative at the conceptual level, but implementations retain freedom in how they realize it. The intended first implementation centers on a persistent authored graph with derived optimized views and explicit origin mappings. Equivalent implementations, including separate optimized graphs, remain acceptable if the invariants below hold.

### Required invariants

The following invariants **must** hold:

1. Adding a new carrier transformation never mutates prior authored history.
2. Any optimized representation remains semantically equivalent to the authored representation.
3. Schema facts remain derivable and trustworthy across rewrites.
4. Branching from a common carrier remains cheap enough to be a normal authoring pattern.
5. Optimization may change plan shape, but it must not destroy lineage traceability.

### Optimization responsibilities

Optimization is a core Prism responsibility, not merely a downstream backend concern.

For the first Prism slice, Prism **may** perform:

- projection pruning
- predicate pushdown
- redundant-node elimination
- normalization of equivalent logical shapes
- simple shared subplan detection and sharing
- other semantically valid rewrites consistent with schema and lineage invariants

More advanced rewrites such as join reordering, cost-based optimization, or sink-aware splitting **may** be added later.

Implementations **may** apply some rewrites incrementally during plan construction and defer others until lowering or explicit analysis, provided authored history remains intact.

## Design details

### Syntax

This RFC introduces no new author-facing syntax.

### Semantics

Prism is the internal engine that owns logical planning and optimization for InQL carriers.

At minimum, a Prism-backed carrier should be representable as:

- a reference to Prism-managed persistent plan state
- a current logical tip
- schema facts associated with that tip

The exact representation is intentionally not fixed by this RFC, but the semantics of immutability, structural sharing, and lineage preservation are.

### Interaction with other InQL surfaces

- **`DataSet[T]` APIs:** method-chain surfaces defined by InQL RFC 001 **must** build or manipulate Prism-backed logical state without violating carrier immutability.
- **`query {}`:** checked query blocks defined by InQL RFC 003 **should** lower into Prism-managed logical work before final Substrait emission.
- **Pipe-forward (`|>`):** if supported per InQL RFC 005, desugared pipe-forward **must** remain Prism-consistent with the equivalent method-chain or query-block form.
- **Incan `model` types:** Prism optimization legality **must** remain consistent with model-derived schema semantics and must not fall back to runtime-authored schema truth.
- **Substrait / execution:** Prism prepares plans for InQL RFC 002 emission and InQL RFC 004 execution, but it does not replace either sibling boundary.

### Compatibility / migration

This RFC is additive and architectural. It clarifies and stabilizes internal InQL planning semantics; it does not by itself introduce a source-level breaking change for authors or a serialized-plan breaking change for Substrait consumers.

It may, however, motivate refactoring of implementation architecture so that planning, optimization, and emission concerns are separated more clearly than they were before this RFC existed.

## Alternatives considered

- **Keep Prism as a research note only** — rejected for now; the planning and optimization substrate is foundational enough that leaving it undocumented as an implementation note would keep key architectural boundaries implicit.
- **Fold Prism fully into InQL RFC 002** — rejected; Substrait emission and internal planning are related but distinct concerns. Keeping them in one RFC makes the internal engine look like a boundary-format detail.
- **Define Prism as a cross-cutting platform beyond InQL immediately** — rejected for now; Prism may eventually be reused elsewhere, but this RFC keeps the normative scope concrete by defining Prism first as an InQL component with a clean standalone module boundary.

## Drawbacks

- Adds another foundational RFC to the series, which increases up-front design surface before implementation.
- Introduces a conceptual split between authored and optimized plan state that implementations must model carefully.
- Risks over-specifying internal architecture if future Incan constraints make some Prism design choices awkward.

## Layers affected

- **InQL specification** — sibling RFCs that reference logical planning, carrier behavior, Substrait lowering, or session execution **should** remain consistent with Prism as the internal planning substrate.
- **InQL library package** — public carriers and internal planning modules **should** preserve immutable carrier semantics over shared Prism-managed state.
- **Incan compiler** — if InQL surfaces lower through compiler-managed intermediate representations, those integrations **should** respect Prism's lineage and optimization invariants.
- **Execution / interchange** — Session-backed lowering and execution flows **must** treat Prism as internal preparation and Substrait as the boundary contract.
- **Documentation** — RFC indexes, architecture notes, and implementation planning notes **should** distinguish Prism from Substrait and from session execution.

## Implementation Plan

### Phase 1: Internal Prism carrier slice

- Rework `LazyFrame[T]` to become the first real Prism-backed carrier while keeping the public dataset API stable.
- Replace direct `Rel` storage in `LazyFrame[T]` with Prism-managed authored state plus a current logical tip.
- Keep Prism internal-only; do not expose public `Prism*` package APIs in this phase.

### Phase 2: Authored graph + optimized view contract

- Implement the minimum Prism authored node set needed for the first slice: read roots, filters, and joins.
- Represent optimized state as a derived view over authored state with explicit optimized-to-authored origin mappings.
- Keep the first rewrite surface limited to safe canonicalization (`Filter(true)` elimination and adjacent `Limit`/`Project`/`OrderBy` collapse) with explicit lineage bookkeeping; defer heavier rewrite families.

### Phase 3: Boundary lowering and source construction

- Keep RFC 002 as the only emitted boundary by lowering Prism-backed `LazyFrame` state into Substrait at `to_substrait_plan()`.
- Add the internal source-construction seam needed to create Prism-backed lazy carriers from named tables or equivalent read roots.
- Support joins between independently constructed lazy carriers by unifying roots into one Prism-authored graph when needed; do not keep the research-only same-graph join restriction.

### Phase 4: Tests, docs, and current-slice hardening

- Add package tests that prove immutable branching, lineage preservation, and stable lowering back to real proto-backed Substrait plans.
- Update architecture and RFC docs so the implementation status matches the intended internal design rather than the earlier research-only framing.

### Phase 5: Broader carrier and authoring-surface adoption

- Extend Prism backing beyond `LazyFrame[T]` once the remaining foundational RFCs are landed and the surrounding carrier/session story is stable enough to avoid churn.
- Evaluate where `DataFrame[T]`, `DataStream[T]`, and `query {}` should converge on shared Prism planning entry paths without forcing premature execution-boundary coupling.
- Keep advanced optimization families (for example join reordering, cost-based exploration, and AQE-adjacent behavior) out of this phase; those remain optimizer-boundary follow-on work under RFC 008.

## Progress Checklist

### Spec / design

- [x] Lock Prism as an internal-only planning substrate rather than a public package API.
- [x] Lock the intended first implementation to authored graph + derived optimized view + origin mappings.
- [x] Reject the prototype's same-graph-only join constraint as the production design.
- [x] Lock `LazyFrame[T]` as the first real Prism-backed carrier.

### Prism core

- [x] Define the current authored node set for the first implementation slice (`Read`, `Filter`, `Join`, `Project`, `GroupBy`, `Aggregate`, `OrderBy`, `Limit`, `Explode`) so the existing `LazyFrame[T]` method surface is Prism-native.
- [x] Add persistent Prism-managed plan state plus logical tip tracking for `LazyFrame[T]`.
- [x] Add derived optimized views with stable optimized-to-authored origin mappings.
- [x] Introduce a backend-native `PrismCursor[T]` handle as the internal target for `LazyFrame[T]` method delegation.
- [x] Replace temporary Rust-backed Prism store-id allocation with Incan-native typed module static allocation.
- [x] Dedup equivalent reachable RHS nodes during cross-store join adoption while keeping authored-store append-only semantics.
- [x] Retire prototype naming in package internals by moving Prism implementation to `src/prism/mod.incn` and stable `Prism*` internal type names.
- [x] Add default canonical rewrite passes for safe local simplifications (`Filter(true)` elimination, adjacent `Limit`/`Project`/`OrderBy` collapse) before RFC 002 lowering.
- [x] Keep authored graph immutable while deriving rewritten views with rewritten-to-authored origin mappings.
- [x] Add internal rewrite explain artifacts (applied-rule names and rewritten/origin cardinality facts) for test diagnostics.

### Carrier integration

- [x] Replace `LazyFrame[T]` direct `Rel` storage with Prism-backed state.
- [x] Keep the public `DataSet[T]` / `LazyFrame[T]` method surface unchanged.
- [x] Support joins across independently constructed lazy carriers by graph unification rather than prototype-only shared-graph assumptions.
- [x] Route `LazyFrame[T]` method semantics through a backend-native cursor layer instead of per-method carrier-owned graph manipulation.
- [x] Route `LazyFrame[T]` methods through Prism internal seam helpers so future authoring surfaces can reuse one planning entry path.

### Substrait boundary

- [x] Lower Prism-backed `LazyFrame[T]` into real RFC 002 Substrait only at the boundary.
- [x] Preserve current conformance behavior for `Read`, `Filter`, and `Join`.
- [x] Replace identity-only lowering with safe canonical rewritten lowering while preserving semantic equivalence.

### Tests

- [x] Add package tests for immutable branching over shared authored state.
- [x] Add package tests for optimized-view origin mapping.
- [x] Add package tests for join lowering across branches and independently constructed lazy carriers.
- [x] Add regression coverage proving Prism-backed `LazyFrame[T]` still emits real proto-backed Substrait plans.
- [x] Add regression coverage proving the current `LazyFrame[T]` method surface now maps to native Prism node kinds rather than opaque compatibility nodes.
- [x] Add rewrite regressions for canonicalization and explain artifact coherence.

### Docs

- [x] Update architecture docs to reflect Prism as active implementation work rather than purely ahead-of-code design.
- [x] Keep RFC 007, RFC index, and related architecture notes aligned as implementation lands.

### Phase 5 follow-on adoption

- [ ] Extend Prism backing to `DataFrame[T]` once the remaining foundational RFCs are complete.
- [ ] Extend Prism backing to `DataStream[T]` once the remaining foundational RFCs are complete.
- [ ] Route `query {}` authoring through shared Prism planning entry paths once the remaining foundational RFCs are complete.

## Design Decisions

### Resolved

- Prism conceptually distinguishes authored and optimized state, but the intended first implementation centers on a persistent authored graph with derived optimized views and explicit origin mappings. Equivalent implementations, including separate optimized graphs, remain acceptable if they preserve the same invariants.
- The first Prism slice commits only to safe logical rewrites: projection pruning, predicate pushdown, redundant-node elimination, normalization of equivalent logical shapes, and optional simple shared-subplan detection. Heavier work such as join reordering, cost-based optimization, and sink-aware splitting is explicitly deferred.
- The minimum lineage contract is stable authored node IDs plus optimized-to-authored origin mappings. Richer explain/debug structures may be added later, but they are not required for the RFC to be complete.
- RFC 007 does not require a new upstream Incan RFC before moving to `Planned`. Implementation may expose compiler or tooling gaps later, but those are implementation dependencies rather than specification blockers.
- Prism remains an internal InQL planning substrate for now; the first implementation does not expose public `Prism*` package APIs.
- `LazyFrame[T]` is the first real Prism-backed carrier. `DataFrame[T]`, `DataStream[T]`, and `query {}` integration remain follow-on work unless the `LazyFrame[T]` slice proves a hard dependency.
- `PrismCursor[T]` is the current backend-native handle beneath `LazyFrame[T]`. It is an internal convergence target for future `query {}` and pipe-forward lowering, not a public package API.
- The research prototype demonstrated the seam, but its clone-heavy storage and same-graph-only join restriction are not the intended production design.
- Real joins between lazy carriers must work even when the two sides were constructed independently; implementations may unify roots into one authored graph internally, but they must not require pre-shared lineage as a public contract.
