# InQL RFC 008: Optimizer boundary, statistics, cost-based optimization, and adaptive execution

- **Status:** Draft
- **Created:** 2026-04-07
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 004 (execution context — `Session` remains the execution and backend boundary)
  - InQL RFC 007 (Prism planning engine — this RFC narrows optimizer-boundary ownership without replacing Prism adoption)
- **Issue:** [InQL #18](https://github.com/dannys-code-corner/InQL/issues/18)
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines the optimizer boundary between **Prism** and **`Session`** as InQL grows beyond the first Prism adoption slice. Prism remains the internal engine for analyzed logical planning, semantic rewrites, property inference, logical alternative exploration, and cost-model hooks. `Session` remains the owner of backend capabilities, catalog statistics, physical planning, pushdown policy, runtime statistics, and adaptive re-planning during execution. This RFC does not replace RFC 007's role in establishing Prism as the internal planning substrate; it supersedes only the optimizer-boundary guidance where RFC 007 used broader wording or early-slice examples that would otherwise blur semantic optimization and backend execution policy.

## Motivation

RFC 007 was intentionally written to get Prism named, scoped, and implemented as a real planning substrate. That was the right move for the first Prism adoption slice. However, once InQL aims for stronger optimization, the remaining ambiguity becomes a liability:

- If Prism owns all optimization in the abstract, it will tend to absorb backend policy and runtime behavior.
- If `Session` owns all optimization in practice, Prism becomes a passive container rather than a serious optimizer substrate.
- If statistics, cost-based optimization, and adaptive re-planning are not assigned cleanly, explain output, reproducibility, and backend substitution all become muddled.

InQL needs the same kind of separation that high-performance query engines converge on in practice:

- a semantic logical optimizer that can reason about equivalence and properties
- a backend boundary that can exploit concrete storage and runtime facts
- a runtime layer that can react when real cardinalities, partition sizes, or skew differ from pre-execution estimates

The goal is not to copy Spark literally. The goal is to adopt the useful split in spirit: logical optimization is not the same thing as physical optimization, and neither is the same thing as adaptive re-optimization during execution.

## Goals

- Define which optimizer responsibilities belong to Prism versus `Session`.
- Establish **statistics ownership** clearly enough to support cost-based optimization without collapsing the Prism / `Session` boundary.
- Define the minimum optimizer artifacts Prism should expose as InQL moves past the initial identity-shaped optimized view.
- Reserve adaptive query re-planning as a `Session` concern rather than a Prism concern.
- Make precedence against RFC 007 explicit so future work does not rely on ambiguous wording.

## Non-Goals

- Replacing RFC 007 as the historical record of the first Prism adoption slice.
- Standardizing one exact memo structure, cost formula, or join enumeration algorithm.
- Requiring adaptive execution in v0.1.
- Defining backend-specific tuning knobs for DataFusion or any other engine.
- Defining new author-facing query syntax.

## Guide-level explanation

Authors still think in the same broad pipeline:

```text
author intent
  -> Prism raw plan
  -> Prism analyzed / optimized logical plan
  -> Session physical planning and execution
  -> optional Session adaptive re-planning while executing
```

The important mental model is:

- Prism decides what logical transformations are valid and what alternatives are semantically equivalent.
- Prism can prefer one logical alternative over another using inferred properties and cost-model hooks.
- `Session` decides what the selected backend can actually do well.
- `Session` can change physical strategy when runtime facts prove the original assumptions wrong.

Conceptual example; exact explain API names may differ:

```incan
from pub::inql import Session, LazyFrame, DataFrame
from pub::inql.functions import count
from models import Customer, Order, RegionalSummary

session = Session.default()

orders: LazyFrame[Order] = session.table("orders")
customers: LazyFrame[Customer] = session.table("customers")

summary: LazyFrame[RegionalSummary] = query {
    FROM orders
    JOIN customers ON .customer_id == .id
    WHERE .status == "completed"
    GROUP BY .region
    SELECT
        region,
        count() as order_count,
}

# Prism-owned logical surfaces
summary.raw_plan()
summary.analyzed_plan()
summary.plan_after_inql_rules()

# Session-owned execution and runtime behavior
result: DataFrame[RegionalSummary] = session.collect(summary)
session.session_plan(summary)
session.executed_plan(summary)
```

For explain and tooling, InQL should make the stages explicit instead of collapsing them into one vague “optimized plan” label. A future author or tool should be able to ask for at least:

- `raw_plan()` — authored Prism DAG
- `analyzed_plan()` — resolved names, schemas, and derived logical properties
- `plan_after_inql_rules()` — Prism-owned logical rewrites
- `session_plan()` — backend-bound physical or session-owned optimized plan
- `executed_plan()` — what actually ran, including adaptive changes when available

## Reference-level explanation (precise rules)

### 1. Boundary ownership

Prism **must** own:

- logical plan analysis and semantic validation beyond raw authored structure
- schema and expression-driven rewrites that are deterministic given rules version, plan, and available inputs
- derivation of logical properties and constraints from author intent and schema facts
- logical alternative exploration for equivalent plans
- cost-model interfaces used to compare logical alternatives
- provenance and explain mappings from rewritten logical artifacts back to authored intent

`Session` **must** own:

- backend capability discovery and backend-specific planning policy
- catalog or source statistics acquisition
- pushdown policy into concrete scans or remote engines
- physical operator selection
- runtime statistics gathered during execution
- adaptive re-planning during execution

Prism **must not** own:

- backend-specific pushdown outcomes
- physical operator choice
- runtime adaptive plan changes
- direct catalog I/O for statistics discovery as a normative responsibility

### 2. Statistics model

This RFC distinguishes three statistics families:

1. **Logical inferred facts** — bounds or properties Prism derives from schema, constraints, and expressions.
2. **Pre-execution source statistics** — row counts, NDV estimates, histograms, file sizes, partition metadata, or equivalent facts supplied through `Session` and its backend or catalog integrations.
3. **Runtime statistics** — observed row counts, partition sizes, skew signals, spill facts, and other execution-time measurements.

Ownership rules:

- Prism **may** consume all three families when available.
- Prism **must** be able to produce a valid logical result even when only logical inferred facts are available.
- `Session` **must** supply source and runtime statistics to the extent the selected backend can provide them.
- Runtime statistics **must not** be treated as Prism-authored facts; they belong to execution-time artifacts.

### 3. Cost-based optimization

Cost-based optimization in InQL is split across Prism and `Session`:

- Prism **should** compare logical alternatives using a stable cost interface over logical properties, available statistics, and backend capability hints.
- `Session` **may** provide the statistics and capability inputs Prism needs to make better logical choices.
- The exact cost model is not standardized by this RFC, but the ownership boundary is.

Inference from this boundary: if InQL later adds join reordering, memo-based exploration, or reuse/materialization decisions, those features belong primarily in Prism, but they rely on inputs that `Session` can provide.

### 4. Adaptive execution

Adaptive re-planning during execution is a `Session` concern.

That means:

- Prism may produce a preferred logical plan or ranked alternatives before execution.
- `Session` may revise physical strategy during execution based on runtime statistics.
- `Session` may record those adaptive decisions in `executed_plan()` or equivalent explain surfaces.
- Adaptive behavior **must not** mutate Prism-authored history.

### 5. Named optimizer artifacts

Implementations **should** expose distinct names for plan stages rather than one ambiguous “optimized plan” API.

Minimum intended names:

- `raw_plan()`
- `analyzed_plan()`
- `plan_after_inql_rules()`
- `session_plan()`
- `executed_plan()`

Equivalent names are acceptable if they preserve the same separation.

### 6. Precedence against older RFCs

RFC 007 remains authoritative for:

- Prism as InQL's internal planning substrate
- immutable authored state
- structural sharing
- lineage and optimized-to-authored provenance expectations

This RFC supersedes RFC 007 only where RFC 007's optimizer examples or broad wording would otherwise imply that Prism owns backend pushdown policy, physical planning, statistics ownership, or adaptive re-planning.

RFC 004 remains authoritative for:

- the existence and core shape of `Session`
- execution and collection entry points
- backend abstraction and DataFusion as the default reference backend

This RFC narrows RFC 004 by making the optimizer boundary more explicit; where optimizer ownership is discussed, this RFC governs.

## Design details

### Syntax

This RFC introduces no new author-facing syntax.

### Semantics

As Prism evolves beyond the initial implementation slice, the intended logical stack is:

- raw authored DAG
- analyzed logical plan with resolved references and derived properties
- Prism-owned logical rewrite stages
- optional logical alternative exploration and cost comparison
- `Session` handoff for backend planning and execution

Prism's optimizer legality **must** continue to derive from InQL semantics, schema facts, and expression rules rather than backend quirks.

`Session` **may** pass backend capability hints and statistics into Prism before execution, but those inputs **must not** collapse the ownership boundary defined above.

### Interaction with other InQL surfaces

- **`DataSet[T]` APIs:** carrier method chains continue to build Prism-managed authored state. Stronger optimization does not change carrier immutability rules.
- **`query {}`:** query-block lowering should target the same Prism analysis and rewrite pipeline as method chains.
- **Pipe-forward (`|>`):** if shipped, desugared forms must enter the same Prism optimizer boundary rather than defining a parallel optimizer path.
- **Substrait boundary:** Substrait remains the normative interchange contract. This RFC governs how Prism and `Session` arrive at plans around that boundary; it does not replace RFC 002.

### Compatibility / migration

This RFC is additive at the API and architecture level:

- It does not require authors to change query syntax.
- It does not require RFC 007 to be rewritten.
- It may require documentation and future implementation work to stop referring to all optimization as one undifferentiated Prism responsibility.

Existing prototype APIs that use vague names like `optimized_view` remain acceptable as transitional implementation details, but future public-facing documentation **should** migrate to more precise stage names.

## Alternatives considered

- **Keep RFC 007 as the only optimizer RFC** — rejected; RFC 007 already serves as the Prism adoption record for the first implementation slice, and retrofitting a more detailed optimizer boundary into it would mix historical adoption work with the follow-on architecture.
- **Move all optimization to `Session`** — rejected; that would reduce Prism to a plan container and make InQL-owned semantic optimization too backend-dependent.
- **Move AQE into Prism** — rejected; adaptive re-planning depends on runtime execution facts and should remain session-owned.
- **Treat statistics as purely backend-private** — rejected; Prism needs a clean way to consume statistics if it is going to perform serious cost-based logical optimization.

## Drawbacks

- Adds another foundational RFC and another precedence edge contributors must understand.
- Commits InQL to a sharper optimizer vocabulary earlier than a minimal prototype would require.
- Memo-based exploration, property inference, and stats plumbing will increase implementation complexity once work begins.

## Layers affected

- **InQL specification** — RFC 004 and RFC 007 references to optimization ownership **should** stay consistent with this boundary.
- **InQL library package** — future Prism internals **should** separate authored, analyzed, and rewritten artifacts more explicitly than the current prototype does.
- **Execution / interchange** — `Session` and backend integration layers **must** own physical planning, runtime stats, and adaptive re-planning policy.
- **Documentation** — explain surfaces and architecture notes **should** stop using “optimized plan” as an undifferentiated term.

## Unresolved questions

- Should the Prism/`Session` handoff carry source statistics as a concrete snapshot artifact, a callable capability surface, or both?
- Should `session_plan()` and `executed_plan()` be exposed on `Session`, on carriers, or through a dedicated explain object?
- When runtime statistics materially improve later planning, what information may be cached across executions without blurring authored facts, session facts, and execution facts?

<!-- Rename this section to "Design Decisions" once all questions have been resolved. An RFC cannot move from Draft to Planned until no unresolved questions remain. -->
