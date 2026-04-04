# InQL RFC 007: Prism logical planning and optimization engine

- **Status:** Draft
- **Created:** 2026-04-02
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 001 (dataset types and carriers — Prism-backed carriers must remain consistent with `DataSet[T]` semantics)
  - InQL RFC 002 (Apache Substrait integration — Substrait remains the normative emitted contract at the boundary)
  - InQL RFC 003 (`query {}` — lowers through Prism-managed logical work before Substrait emission)
  - InQL RFC 004 (execution context — session executes Prism-backed plans but does not define Prism)
  - InQL RFC 005 (optional pipe-forward — must stay Prism-consistent with equivalent surfaces)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines **Prism** as InQL's immutable internal logical planning and optimization engine. Prism owns persistent plan storage, cheap branching through structural sharing, lineage-preserving rewrites, and logical optimization prior to Substrait emission or session execution. Prism is an **internal planning substrate**, not the normative interchange contract: **Apache Substrait** remains the boundary format per InQL RFC 002. `LazyFrame`, `DataFrame`, and `DataStream` are carrier experiences over Prism-managed plan state; `Session` and `SessionContext` bind and execute those plans per InQL RFC 004.

## Motivation

InQL already has a strong external story around typed carriers, Substrait emission, and the execution boundary, but it lacks a dedicated specification for the internal planning layer that sits between authored logic and emitted plans. Without that layer being named and scoped, plan construction, optimization, lineage, interactive behavior, and future explain/debug tooling risk becoming an accidental mix of implementation details spread across InQL RFC 001, InQL RFC 002, and InQL RFC 004.

Prism gives that layer a home. It lets InQL say clearly that:

- authored transformations build immutable logical plans
- carriers stay cheap by sharing planning state instead of cloning whole plans
- optimization is a first-class responsibility, not an incidental backend side effect
- lineage must survive rewrites so optimized plans remain explainable

This matters for more than simple query lowering. Complex multi-hop pipelines, future interactive environments, and prospective reuse of the planning substrate beyond InQL all benefit from a stable definition of what the internal plan engine is allowed and required to do.

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
- Promising Prism as a general-purpose platform beyond InQL today. This RFC scopes Prism normatively to InQL, while requiring a clean enough module boundary that future extraction remains possible.

## Guide-level explanation

From an author's point of view, Prism is not something they use directly. Authors work with InQL carriers such as `LazyFrame[T]`, `DataFrame[T]`, and (later) `DataStream[T]`. Those carriers build or operate over logical work that Prism stores and optimizes internally.

```incan
orders = session.table("orders")
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

Prism is the reason this can work efficiently. It stores the shared logical planning state, allows both `high_value` and `recent` to branch from the same base plan, and may optimize the resulting logical graph before the plan is emitted to Substrait or executed by a session.

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

An implementation **may** use Prism-native node kinds or overlays internally, but emitted plans that claim conformance **must** still follow InQL RFC 002.

### Relationship to session execution

Prism does not execute plans. `Session` / `SessionContext` own execution.

Execution-oriented flows **must** treat Prism as an input to lowering and execution, not as the executor itself. Session-backed operations may request optimized views from Prism before emission or execution, but the existence of Prism **must not** collapse the execution boundary defined in InQL RFC 004.

### Authored state vs optimized state

Prism **should** conceptually distinguish between:

- **authored plan state**: persistent construction history closest to user intent
- **optimized plan state**: semantically equivalent rewritten state used for lowering or execution
- **lineage metadata**: mappings from optimized state back to authored history

This distinction is normative at the conceptual level, but implementations retain freedom in how they realize it. A single persistent graph with overlays, separate graphs with references, or another equivalent structure are all acceptable if the invariants below hold.

### Required invariants

The following invariants **must** hold:

1. Adding a new carrier transformation never mutates prior authored history.
2. Any optimized representation remains semantically equivalent to the authored representation.
3. Schema facts remain derivable and trustworthy across rewrites.
4. Branching from a common carrier remains cheap enough to be a normal authoring pattern.
5. Optimization may change plan shape, but it must not destroy lineage traceability.

### Optimization responsibilities

Optimization is a core Prism responsibility, not merely a downstream backend concern.

Prism **may** perform:

- projection pruning
- predicate pushdown
- redundant-node elimination
- normalization of equivalent logical shapes
- shared subplan detection and sharing
- other semantically valid rewrites consistent with schema and lineage invariants

More advanced rewrites such as join reordering or sink-aware splitting **may** be added later.

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

## Unresolved questions

- Should Prism maintain one persistent graph with optimized overlays, or separate authored and optimized graphs with explicit references?
- Which optimization passes are part of the Prism north star immediately, and which should be deferred until after the first implementation?
- What is the most useful lineage metadata shape for explain/debug tooling without making normal plan construction expensive?
- Are there Incan language or tooling limitations around model-derived schema facts that Prism depends on and that may require an upstream Incan RFC?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
