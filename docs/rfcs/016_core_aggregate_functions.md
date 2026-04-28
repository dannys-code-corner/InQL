# InQL RFC 016: Core aggregate functions

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset carriers and aggregation surface)
  - InQL RFC 003 (`query {}` aggregate rules)
  - InQL RFC 012 (scalar expressions and aggregate measures)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines InQL's core aggregate function set: `count`, `sum`, `avg`, `min`, and `max`, including argument forms, input type rules, null behavior, empty-input behavior, result type rules, aliases, and required diagnostics. These aggregates form the minimum portable aggregate surface for dataframe and query-block work.

## Motivation

InQL already exposes `sum` and argument-free `count`, but real grouped analysis needs a stable minimum aggregate family. Beam's SQL aggregate surface centers on `COUNT`, `AVG`, `SUM`, `MAX`, and `MIN`; DataFusion, Spark, and Snowflake also support these as foundational aggregates. InQL should define these before broader statistical, collection, approximate, or ordered aggregates.

Without explicit rules, aggregate behavior can drift across authoring surfaces and backends. Null skipping, empty groups, `count(*)` versus `count(expr)`, and numeric result type rules are observable semantics, not implementation details.

## Goals

- Define canonical core aggregate functions.
- Distinguish `count()` / `count(*)` from `count(expr)`.
- Define null behavior and empty-input behavior.
- Define aggregate output naming expectations.
- Require explicit errors for invalid aggregate positions and invalid argument types.

## Non-Goals

- Defining aggregate `DISTINCT`, `FILTER`, or ordered aggregate modifiers.
- Defining statistical aggregates beyond `avg`.
- Defining collection, string, approximate, or sketch aggregates.
- Defining window function semantics.
- Defining streaming trigger or watermark semantics.

## Guide-level explanation (how authors think about it)

Authors can summarize grouped or whole-relation data with the core aggregate set:

```incan
from pub::inql.functions import avg, col, count, max, min, sum

summary = (
    orders
        .group_by([col("customer_id")])
        .agg([
            count(),
            count(col("discount_code")),
            sum(col("amount")),
            avg(col("amount")),
            min(col("created_at")),
            max(col("created_at")),
        ])
)
```

`count()` counts rows. `count(col("discount_code"))` counts non-null values in that expression. Numeric aggregates ignore null input values unless every value is null or the group is empty, in which case result behavior follows the rules below.

## Reference-level explanation (precise rules)

InQL must define canonical aggregate entries for `count`, `sum`, `avg`, `min`, and `max`.

`count()` must count input rows in the current relation or group. `count(expr)` must count rows where `expr` evaluates to a non-null value. `count` must return a non-null integer count. For an empty input relation or empty group, `count()` and `count(expr)` must return zero.

`sum(expr)` must accept numeric input expressions. It must ignore null input values. If no non-null value exists in the group or relation, `sum` must return null unless a later RFC defines an explicit defaulting aggregate. The result type must be derived from the input numeric type according to the numeric type policy.

`avg(expr)` must accept numeric input expressions. It must ignore null input values. If no non-null value exists in the group or relation, `avg` must return null. Its result type must be capable of representing fractional results unless the input type family explicitly defines otherwise.

`min(expr)` and `max(expr)` must accept orderable input expressions. They must ignore null input values. If no non-null value exists in the group or relation, they must return null.

Aggregate outputs must be aggregate measures, not row-level scalar expressions. They may appear only in aggregate-capable positions defined by query blocks, dataframe aggregation methods, or later RFCs.

The registry must record compatibility aliases where useful. `mean` may be an alias for `avg` if InQL accepts dataframe-style naming. Warehouse compatibility names such as `count_if` remain outside this core RFC unless modeled as aggregate modifiers by InQL RFC 017.

## Design details

### Syntax

This RFC requires importable aggregate functions. Query syntax may support SQL spellings such as `COUNT(*)`, but that spelling must map to the same `count()` aggregate semantics.

### Semantics

Core aggregates skip null values except for `count()`, which counts rows regardless of null values. `count(expr)` counts non-null expression results.

Aggregate argument expressions must be scalar expressions under InQL RFC 012. Aggregate arguments must not themselves contain aggregate outputs unless a later RFC defines nested aggregate semantics.

### Interaction with other InQL surfaces

`query {}` blocks must apply grouped-reference rules consistently with these aggregate definitions. Dataframe `.group_by(...).agg(...)` calls must produce the same aggregate semantics as equivalent query-block aggregates.

### Compatibility / migration

Existing `sum` and `count` helpers should be treated as compatibility-compatible forms of the canonical registry entries. `count()` behavior remains compatible with the current row-count intent.

## Alternatives considered

- **Define all Spark aggregates at once.** Rejected because core aggregate semantics should be stabilized before statistics, collection, approximate, and sketch families.
- **Make `count` accept no arguments forever.** Rejected because `count(expr)` is foundational SQL/dataframe behavior.
- **Return zero from `sum` on empty input.** Rejected for core semantics because SQL-style null-on-no-value behavior better distinguishes "no values" from "values that sum to zero."

## Drawbacks

- Null and empty-input behavior can surprise authors coming from APIs that default missing sums to zero.
- Result type policy for numeric aggregates is a cross-cutting dependency on scalar numeric types.
- Supporting both `count()` and `count(expr)` increases overload complexity.

## Layers affected

- **InQL specification** — aggregate measures must remain distinct from scalar expressions and consistent with InQL RFC 012.
- **InQL library package** — public aggregate helpers should support the core aggregate set and argument forms.
- **Incan compiler** — query-block aggregate checking must validate grouped and aggregated positions against these rules.
- **Execution / interchange** — Prism and Substrait lowering must preserve null skipping, empty-input behavior, and count forms.
- **Documentation** — aggregate reference docs should document null and empty-input behavior explicitly.

## Unresolved questions

- What exact integer type should `count` return across local and backend execution?
- Should `avg` over integers return a floating type, decimal type, or backend-derived numeric type?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
