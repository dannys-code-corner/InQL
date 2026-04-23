# InQL RFC 012: Unified scalar expression surface

- **Status:** Draft
- **Created:** 2026-04-22
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset carriers and method-chain API surface)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
  - InQL RFC 004 (execution context and backend execution boundary)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - Incan RFC 040 (scoped DSL glyph surfaces)
  - Incan RFC 045 (scoped DSL symbol surfaces)
- **Issue:** [InQL #25](https://github.com/dannys-code-corner/InQL/issues/25)
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

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

The exact public literal helper spelling is still unresolved in this Draft. The examples below use `lit(...)` illustratively to show the semantic model, not to settle the final helper name.

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

Separate public wrapper types may exist during migration, but they must lower into the same canonical scalar expression model rather than remaining semantically independent systems.

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

The semantic concept of a scalar literal must be unified. InQL may expose one canonical helper such as `lit(...)`, or a migration family of typed helpers that all lower into the same scalar-literal representation, but the system must not preserve separate literal hierarchies for filters, projections, and other row-level positions.

### Lowering target for future authoring surfaces

If InQL adopts future concise method-chain sugar or richer `query {}` syntax using RFC 040 and RFC 045 facilities, those surfaces must lower into the canonical scalar expression model for row-level meaning and the canonical aggregate-measure model for aggregate meaning.

## Design details

### Syntax

This RFC does not require new InQL syntax. Existing builder-call surfaces are sufficient to define the contract.

The long-term intention is that concise surfaces may exist, but they are only acceptable if they lower into the same canonical expression model defined here.

### Semantics

The core semantic split is:

- scalar expressions describe one value per input row
- aggregate measures describe one value per group or per whole relation

Boolean predicates are ordinary scalar expressions whose result type is `bool`; they are not a separate semantic species.

Grouping keys belong on the scalar-expression side. They determine grouping identity by evaluating one scalar expression per input row. InQL may initially support only a subset of scalar expressions for grouping, but if so, that restriction must be explicit and diagnosable.

### Interaction with other InQL surfaces

- **Dataset carriers and method chains (InQL RFC 001):** method-chain surfaces such as `filter(...)`, `with_column(...)`, `group_by(...)`, and future projection methods should consume one scalar-expression model rather than independent mini-DSLs.
- **`query {}` blocks (InQL RFC 003):** query-block expressions should lower into the same scalar-expression and aggregate-measure contracts rather than defining a separate semantic path.
- **Execution context (InQL RFC 004):** session execution should receive one row-level expression contract and one aggregate-measure contract, not surface-specific variants.
- **Prism (InQL RFC 007):** Prism should represent row-level expression meaning once and reuse it across logical operators instead of duplicating per-surface expression semantics.
- **Incan `model` types and lexical scope:** model fields remain the source of column naming and typing, and ordinary lexical scope rules still govern explicit helper references until scoped DSL facilities are adopted.

### Compatibility / migration

This RFC is additive as a design direction, but it may require cleanup of existing public builder families.

The expected migration shape is:

- legacy typed literal helpers may remain temporarily as compatibility shims
- legacy predicate-specific wrappers may remain temporarily if they lower into the same scalar-expression model
- docs and diagnostics should steer authors toward one canonical scalar-expression concept
- split row-level helper families should be deprecated and eventually removed once compatibility windows close

Correctness takes precedence over convenience during migration. If a permissive compatibility path would silently change semantics, InQL should reject that path instead.

## Alternatives considered

- **Keep predicate, literal, and projection surfaces separate.** Rejected because the split duplicates concepts that are semantically the same and makes drift between authoring surfaces more likely.
- **Unify only literals.** Rejected because the problem is broader than helper naming; the real issue is multiple row-level semantic systems.
- **Treat aggregate calls as ordinary scalar expressions everywhere.** Rejected because aggregate outputs are group-level values, not row-level values, and collapsing that distinction makes typing and position rules less coherent.
- **Wait for concise DSL syntax first.** Rejected because concise syntax without a canonical lowering target would just create more semantic drift, not less.

## Drawbacks

- Existing InQL surfaces that grew independently may need migration and deprecation work.
- Tooling and diagnostics become more demanding because the system must enforce the scalar-versus-aggregate boundary more consistently.
- Some previously tolerated expression shapes may need to become hard errors if they only "worked" through accidental or degraded behavior.
- The RFC makes inconsistencies more visible, which can force earlier cleanup across docs, examples, planning, and lowering.

## Layers affected

- **InQL specification** — RFCs 001, 003, 004, and 007 must stay coherent with one shared scalar-expression and aggregate-measure contract.
- **InQL library package** — public `.incn` APIs should converge on one canonical row-level expression model and explicit aggregate-measure wrappers.
- **Incan compiler** — if InQL adopts scoped DSL sugar later, parser, checker, lowering, and diagnostics must preserve the scalar-expression lowering contract rather than inventing a separate semantic path.
- **Execution / interchange** — Prism and Substrait lowering must preserve the scalar-versus-aggregate boundary and must not silently rewrite unsupported expression shapes.
- **Documentation** — user docs and reference pages should describe one row-level expression model instead of multiple parallel mini-DSLs.

## Unresolved questions

- Should InQL standardize a canonical public literal helper spelling such as `lit(...)`, or only standardize the semantic concept and allow multiple helper spellings during a long compatibility period?
- Should grouping keys eventually accept all scalar expressions that projections accept, or should InQL permanently support a narrower grouping subset?
- Should aggregate outputs remain completely disallowed inside row-level scalar expressions, or is there future design space for explicitly scoped mixed aggregate projections?
- How much of this contract should be represented as shared InQL conventions versus first-class vocabulary metadata that tooling can inspect directly?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
