# InQL RFC 012: Unified scalar expression surface

- **Status:** Implemented
- **Created:** 2026-04-22
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset carriers and method-chain API surface)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
  - InQL RFC 004 (execution context and backend execution boundary)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - Incan RFC 025 (multi-instantiation trait dispatch)
  - Incan RFC 028 (trait-based operator overloading)
  - Incan RFC 029 (union types and type narrowing)
  - Incan RFC 040 (scoped DSL glyph surfaces)
  - Incan RFC 045 (scoped DSL symbol surfaces)
- **Issue:** [InQL #25](https://github.com/dannys-code-corner/InQL/issues/25)
- **RFC PR:** —
- **Written against:** Incan v0.3.0
- **Shipped in:** v0.1

## Summary

This RFC defines a single canonical scalar expression model for row-level relational meaning in InQL. Filter predicates, computed projection values, grouping keys, and aggregate arguments must all be expressed through the same scalar expression surface, while aggregate outputs remain a distinct aggregate-measure layer. The goal is to replace split mini-DSLs for predicates, literals, and projection expressions with one coherent authoring and lowering contract that all InQL surfaces share.

## Motivation

InQL has reached the point where split builder surfaces are becoming a design liability rather than a harmless implementation detail. Filters, computed projections, grouping keys, and aggregate arguments all describe row-level meaning, but they are currently easy to model as separate families because features landed incrementally. That split creates three problems.

First, it makes the author contract harder to understand. Authors have to learn which helper family belongs to which surface even when the underlying intent is the same. A numeric literal used in a filter and a numeric literal used in a computed projection should not feel like different concepts.

Second, it encourages duplicated or drifting semantics across package layers. If one path accepts richer expression shapes than another, the library either accumulates inconsistent behavior or exposes APIs that appear broader than they really are.

Third, split expression families produce the worst kind of failure mode: silent degradation. If a public method accepts a broad expression type but only truly supports direct column references in that position, unsupported shapes can be dropped or rewritten instead of rejected explicitly. A query library must not treat "unsupported" as "quietly mean something else."

This RFC is also needed before InQL can take proper advantage of the scoped DSL work in Incan. RFC 040 and RFC 045 create a path toward concise surfaces such as `.amount > 100` or ambient `sum(.amount)`, but those surfaces need one canonical lowering target. Without that, InQL would accumulate parallel semantic paths instead of one coherent expression system.

## Goals

- Define one canonical scalar expression model for row-level relational authoring in InQL.
- Require row-level authoring surfaces such as `filter(...)`, `with_column(...)`, future `select(...)`, and grouping keys to consume that same scalar expression model.
- Keep aggregate outputs as a distinct aggregate-measure layer while allowing aggregates to consume scalar-expression inputs.
- Require explicit errors for unsupported expression shapes; silent degradation is not allowed.
- Provide one lowering target for future concise DSL surfaces in method chains and `query {}` blocks.
- Give the InQL package, Prism, and Substrait emission layers one shared semantic contract for row-level expressions.

## Non-Goals

- Defining the full catalog of numeric, string, datetime, conditional, or aggregate functions.
- Introducing new parser syntax in this RFC.
- Defining join output typing, relation schema evolution, or materialization semantics beyond expression authoring.
- Making aggregate outputs behave as ordinary row-level scalar expressions in all positions.
- Standardizing every public helper spelling across all possible InQL libraries or future extensions.

## Guide-level explanation (how authors think about it)

Authors should be able to think in terms of one row-level expression language.

The canonical public literal helper is `lit(...)`. Typed literal helpers are entrypoints over the same scalar-literal representation rather than a separate predicate-only or projection-only literal family.

If an author filters rows or computes a new column, those operations should be using the same underlying scalar expression model:

```incan
from pub::inql import LazyFrame
from pub::inql.functions import col, lit, gt, add
from models import Order

def enrich_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return (
        orders
            .filter(gt(col("amount"), lit(100)))
            .with_column("amount_plus_fee", add(col("amount"), lit(5)))
    )
```

If an author groups rows and supplies arguments to aggregates, those aggregate inputs should still be ordinary scalar expressions even though the aggregate outputs are not:

```incan
from pub::inql import LazyFrame
from pub::inql.functions import col, sum, count
from models import Order, OrderSummary

def summarize_orders(orders: LazyFrame[Order]) -> LazyFrame[OrderSummary]:
    return (
        orders
            .group_by([col("customer_id")])
            .agg([sum(col("amount")), count()])
    )
```

In that example:

- `col("amount")` is a scalar expression
- `lit(100)` and `lit(5)` are scalar expressions
- `gt(...)` and `add(...)` produce scalar expressions
- `filter(...)` requires a scalar expression that resolves to `bool`
- `with_column(...)` requires a row-level scalar expression
- `sum(...)` consumes a scalar expression and produces an aggregate measure
- `count()` is a distinct aggregate form because it does not require a scalar input

This RFC does not require concise sugar, but it defines what concise sugar should mean later. If InQL later supports surfaces such as:

```incan
orders
    .filter(.amount > 100)
    .with_column("amount_plus_fee", .amount + 5)
    .group_by([.customer_id])
    .agg([sum(.amount), count()])
```

those surfaces should lower into the same scalar-expression and aggregate-measure model, not into a separate semantic system.

Authors should also get explicit failure for unsupported shapes. If a library or backend only supports direct column references in one position, `group_by([add(col("a"), lit(1))])` must fail clearly. It must not be silently treated as if no grouping key had been provided.

## Reference-level explanation (precise rules)

### Canonical scalar expression model

InQL must define one canonical scalar expression model for row-level relational meaning. That model may evolve over time, but it must be the semantic target for all row-level expression-bearing surfaces in the package.

At minimum, the canonical scalar expression model must be able to represent:

- column references
- scalar literals
- scalar function or operator application over scalar-expression inputs

Separate public wrapper types may exist as implementation details, but they must lower into the same canonical scalar expression model rather than remaining semantically independent systems.

### Row-level consumers

The following InQL positions must consume scalar expressions:

- row filters
- computed projection values
- future projection lists
- grouping keys, when grouping expressions are supported
- aggregate input arguments

Each position must still enforce its own result-type contract:

- `filter(...)` must require a scalar expression whose result type is `bool`
- `with_column(...)` and projection positions must require a non-aggregate row-level scalar expression
- grouping-key positions must require scalar expressions that are valid grouping keys under the current InQL contract

### Aggregate measures

Aggregate outputs must remain distinct from row-level scalar expressions.

Aggregate functions such as `sum(...)` must consume scalar expressions as arguments and produce aggregate measures. Argument-free aggregates such as `count()` may exist without a scalar-expression input.

Aggregate measures must not be treated as ordinary row-level scalar expressions unless a later RFC defines explicit mixed-scope semantics.

### Explicit failure requirement

InQL package APIs, Prism planning, and Substrait lowering must not silently degrade unsupported expression shapes.

If a public authoring surface accepts an expression shape that cannot be represented or executed faithfully in the target position, the system must produce an explicit diagnostic or planning error.

The following behaviors are forbidden:

- dropping an unsupported grouping expression and treating the operation as a global aggregate
- rewriting a non-column aggregate input into an argument-free aggregate
- replacing an unsupported predicate with a constant truth value

### Canonical literal concept

The semantic concept of a scalar literal must be unified. InQL standardizes `lit(...)` as the canonical public helper for scalar literals. Because Incan RFC 029 gives Incan first-class union types, `lit(...)` may accept a closed union of supported literal input types such as `int | float | str | bool` while still returning one scalar-expression representation.

Typed helpers such as `int_expr(...)`, `float_expr(...)`, `str_expr(...)`, `bool_expr(...)`, `int_lit(...)`, `str_lit(...)`, and `bool_lit(...)` must lower into the same scalar-literal representation as `lit(...)`. The system must not preserve separate literal hierarchies for filters, projections, and other row-level positions.

### Lowering target for future authoring surfaces

If InQL adopts future concise method-chain sugar or richer `query {}` syntax using Incan RFC 025, RFC 028, RFC 029, RFC 040, and RFC 045 facilities, those surfaces must lower into the canonical scalar expression model for row-level meaning and the canonical aggregate-measure model for aggregate meaning.

## Design details

### Syntax

This RFC does not require new InQL syntax. Existing builder-call surfaces are sufficient to define the contract.

The long-term intention is that concise surfaces may exist, but they are only acceptable if they lower into the same canonical expression model defined here.

### Semantics

The core semantic split is:

- scalar expressions describe one value per input row
- aggregate measures describe one value per group or per whole relation

Boolean predicates are ordinary scalar expressions whose result type is `bool`; they are not a separate semantic species.

Grouping keys belong on the scalar-expression side. They determine grouping identity by evaluating one scalar expression per input row. The north-star contract allows deterministic, row-level scalar expressions as grouping keys when their result type is valid for grouping. InQL may initially support only a subset of scalar expressions for grouping, but if so, that restriction must be explicit and diagnosable, not a permanent semantic narrowing and not a silent fallback to direct-column grouping.

### Interaction with other InQL surfaces

- **Dataset carriers and method chains (InQL RFC 001):** method-chain surfaces such as `filter(...)`, `with_column(...)`, `group_by(...)`, and future projection methods should consume one scalar-expression model rather than independent mini-DSLs.
- **`query {}` blocks (InQL RFC 003):** query-block expressions should lower into the same scalar-expression and aggregate-measure contracts rather than defining a separate semantic path.
- **Execution context (InQL RFC 004):** session execution should receive one row-level expression contract and one aggregate-measure contract, not surface-specific variants.
- **Prism (InQL RFC 007):** Prism should represent row-level expression meaning once and reuse it across logical operators instead of duplicating per-surface expression semantics.
- **Incan `model` types and lexical scope:** model fields remain the source of column naming and typing, and ordinary lexical scope rules still govern explicit helper references until scoped DSL facilities are adopted.

### Surface cleanup

This RFC consolidates builder families before InQL's first released API.

The expected cleanup shape is:

- typed literal helpers must not create separate scalar-literal hierarchies
- predicate-specific wrappers must lower into the same scalar-expression model when they are exposed
- docs and diagnostics should steer authors toward one canonical scalar-expression concept
- split row-level helper families should be collapsed before release

Correctness takes precedence over convenience. If a permissive helper path would silently change semantics, InQL should reject that path instead.

## Alternatives considered

- **Keep predicate, literal, and projection surfaces separate.** Rejected because the split duplicates concepts that are semantically the same and makes drift between authoring surfaces more likely.
- **Unify only literals.** Rejected because the problem is broader than helper naming; the real issue is multiple row-level semantic systems.
- **Treat aggregate calls as ordinary scalar expressions everywhere.** Rejected because aggregate outputs are group-level values, not row-level values, and collapsing that distinction makes typing and position rules less coherent.
- **Wait for concise DSL syntax first.** Rejected because concise syntax without a canonical lowering target would just create more semantic drift, not less.

## Drawbacks

- InQL surfaces that grew independently may need cleanup before release.
- Tooling and diagnostics become more demanding because the system must enforce the scalar-versus-aggregate boundary more consistently.
- Some previously tolerated expression shapes may need to become hard errors if they only "worked" through accidental or degraded behavior.
- The RFC makes inconsistencies more visible, which can force earlier cleanup across docs, examples, planning, and lowering.

## Layers affected

- **InQL specification** — RFCs 001, 003, 004, and 007 must stay coherent with one shared scalar-expression and aggregate-measure contract.
- **InQL library package** — public `.incn` APIs should converge on one canonical row-level expression model and explicit aggregate-measure wrappers.
- **Incan compiler** — if InQL adopts scoped DSL sugar later, parser, checker, lowering, and diagnostics must preserve the scalar-expression lowering contract rather than inventing a separate semantic path.
- **Execution / interchange** — Prism and Substrait lowering must preserve the scalar-versus-aggregate boundary and must not silently rewrite unsupported expression shapes.
- **Documentation** — user docs and reference pages should describe one row-level expression model instead of multiple parallel mini-DSLs.

## Implementation Plan

### Phase 1: Canonical scalar expression model

- Introduce one canonical scalar-expression model that can represent column references, scalar literals, and scalar function/operator applications.
- Add `lit(...)` as the canonical literal helper using the Incan RFC 029 union-type surface for supported literal input types.
- Keep typed literal helpers as constructors that lower into the canonical scalar-expression model.
- Keep public helper names aligned with the canonical scalar-expression model.

### Phase 2: Row-level consumers

- Make `filter(...)` consume a scalar expression whose result contract is boolean.
- Make `with_column(...)` and projection assignment helpers consume row-level scalar expressions.
- Make `group_by(...)` consume grouping-key scalar expressions, with explicit errors for scalar expression shapes that the current implementation cannot yet lower faithfully.
- Make aggregate helpers consume scalar expressions as inputs while continuing to produce aggregate measures.

### Phase 3: Prism and Substrait lowering

- Store scalar expressions consistently in Prism filter, projection, grouping, and aggregate-input positions.
- Lower scalar expressions through one shared Substrait expression lowering path where the target position supports the expression.
- Preserve aggregate measures as a distinct group-level representation rather than collapsing them into row-level scalar expressions.
- Replace silent fallback or direct-column-only assumptions with explicit diagnostics or planning errors for unsupported expression shapes.

### Phase 4: Tests, docs, and surface coherence

- Add package tests covering `lit(...)` and typed literal helpers across filters, computed projections, grouping keys, and aggregate inputs.
- Add negative tests for unsupported scalar-expression shapes in grouping and aggregate positions when the implementation cannot lower them faithfully.
- Update reference and explanation docs so users see one scalar expression model instead of separate filter/projection literal families.
- Decide whether this user-visible package change requires an InQL package version bump, and if so keep `incan.toml` and `src/metadata.incn` synchronized.

## Implementation Log

### Spec / design

- [x] Resolve canonical literal helper spelling as `lit(...)`.
- [x] Resolve grouping keys as broad scalar-expression positions with explicit temporary capability errors allowed.
- [x] Resolve aggregate outputs as distinct aggregate measures, not row-level scalar expressions.
- [x] Resolve first-class metadata as a registry/tooling north star owned by the function registry RFCs rather than by RFC 012 alone.
- [x] Verify Incan RFC 025, RFC 028, and RFC 029 dependencies against Incan `origin/main`.

### Scalar expression model

- [x] Define the canonical scalar-expression model in package code.
- [x] Represent column references, scalar literals, and scalar function/operator applications in that model.
- [x] Add canonical `lit(...)` helper for supported literal input types.
- [x] Convert typed literal helpers into constructors over the canonical model.
- [x] Keep public helper imports aligned with the canonical scalar-expression model.

### Row-level consumers

- [x] Update filter APIs to consume scalar expressions.
- [x] Update computed projection APIs to consume scalar expressions.
- [x] Update grouping APIs to consume grouping-key scalar expressions.
- [x] Update aggregate helpers to consume scalar-expression inputs and produce aggregate measures.
- [x] Add explicit errors for accepted expression shapes that cannot yet be represented or lowered faithfully in a target position.

### Prism and Substrait

- [x] Store scalar expressions consistently in Prism filter, projection, grouping, and aggregate-input nodes.
- [x] Lower scalar expressions through one shared Substrait expression-lowering path.
- [x] Preserve aggregate-measure lowering as a separate group-level path.
- [x] Add coverage for direct-column grouping and computed grouping expressions.
- [x] Add coverage for aggregate inputs that are scalar expressions rather than direct column names only.

### Tests

- [x] Add package tests for `lit(...)` in filters.
- [x] Add package tests for `lit(...)` in computed projections.
- [x] Add package tests for scalar-expression grouping keys or explicit unsupported-shape diagnostics.
- [x] Add package tests for aggregate scalar-expression inputs or explicit unsupported-shape diagnostics.
- [x] Keep typed helper tests passing as shared-model coverage.

### Documentation and release

- [x] Update `docs/language/reference/dataset_methods.md`.
- [x] Update builder/function reference docs to teach canonical scalar expressions and typed helper entrypoints.
- [x] Update explanation docs and examples that currently teach separate filter/projection literal families.
- [x] Add release notes if package behavior changes are user-visible.
- [x] No package version bump was required for this pre-release v0.1 slice.

## Design Decisions

- **Canonical literal helper:** `lit(...)` is the canonical public scalar-literal helper. Typed literal helpers construct the same scalar-literal representation and are not separate semantic systems.
- **Incan dependency boundary:** Incan RFC 029 enables a clean union-typed `lit(...)` input surface. Incan RFC 025 and RFC 028 are relevant to later operator-heavy expression sugar, including multiple operator trait instantiations for different right-hand operand types. Those RFCs do not replace the canonical scalar-expression model; they provide authoring surfaces that lower into it.
- **Grouping keys:** Grouping keys are scalar expressions in the north-star contract. Implementations may temporarily reject unsupported grouping expression shapes, but they must do so explicitly and must not permanently encode direct-column grouping as the only semantic model.
- **Aggregate boundary:** Aggregate outputs remain aggregate measures, not row-level scalar expressions. Mixed aggregate/scalar expression semantics require a later RFC.
- **Metadata boundary:** RFC 012 defines the semantic scalar-versus-aggregate contract. First-class function and vocabulary metadata belongs to the function registry and catalog RFCs, but implementations should avoid prose-only or helper-name-only behavior that tooling cannot eventually inspect.
