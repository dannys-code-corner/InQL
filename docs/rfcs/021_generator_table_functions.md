# InQL RFC 021: Generator and table-valued functions

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset carriers and relation operations)
  - InQL RFC 003 (`query {}` clause inventory)
  - InQL RFC 006 (unnest/explode Substrait lowering)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 020 (nested data functions)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines generator and table-valued functions for InQL, including `explode`, `explode_outer`, `posexplode`, `posexplode_outer`, `inline`, `inline_outer`, `flatten`, `stack`, and selected tuple-producing extraction helpers. These functions change relation shape or cardinality and therefore must be modeled as relation operations, not scalar expressions.

## Motivation

Spark exposes generators near functions, and Snowflake exposes `FLATTEN` as a table function, but their semantics are fundamentally different from scalar functions. `explode` and `flatten` turn one input row into zero or more output rows. `inline` can turn nested fields into multiple output columns. `stack` constructs multiple rows. Treating these as scalar functions would make planning, typing, and lowering unsound.

InQL already has an unnest/explode design direction through its Substrait work. This RFC gives the broader generator family a clear semantic home.

## Goals

- Define generator and table-valued function class semantics.
- Define core generator names and their relation-shape effects.
- Distinguish inner and outer generator behavior.
- Define positional output for positional generators.
- Require explicit output schema rules.

## Non-Goals

- Defining scalar nested collection functions.
- Defining JSON parsing or variant extraction semantics except where a helper is explicitly table-valued.
- Defining all SQL table functions.
- Defining backend-specific physical execution strategies.

## Guide-level explanation (how authors think about it)

Authors should use generators when one input row may become multiple output rows:

```incan
from pub::inql.functions import col

items = (
    orders
        .explode(col("line_items"), as_="line_item")
        .select(["order_id", "line_item"])
)
```

The result has a different relation shape from the input. This is not the same kind of expression as `array_contains(...)` or `cardinality(...)`.

## Reference-level explanation (precise rules)

Generator functions must be registry entries with function class `generator` or `table-valued`. They must not be valid in ordinary scalar expression positions.

`explode(array_expr)` must produce one output row for each element of the array expression. If the input array is null or empty, the non-outer form must produce zero rows for that input row.

`explode_outer(array_expr)` must preserve the input row when the input array is null or empty and must produce a null generated value according to its output schema.

`posexplode(array_expr)` and `posexplode_outer(array_expr)` must include a positional output column in addition to the generated element. The position origin must be specified before this RFC reaches Planned status.

`inline(array_of_struct_expr)` must expand each struct element into output columns. `inline_outer` must preserve outer rows for null or empty input according to the outer generator rule.

`stack` must construct multiple output rows from explicit expressions according to a declared row count and output schema.

`flatten` must be treated as a table-valued/generator operation when supported. Its exact input type, recursive behavior, path behavior, and output columns must be specified before it reaches Planned status.

Every generator must define output column names, output types, nullability, interaction with existing columns, and aliasing requirements. Name collisions must be diagnosed unless an explicit overwrite or qualification rule applies.

## Design details

### Syntax

Generators may appear as dataframe relation methods, query-block clauses, or table-valued function forms. Regardless of syntax, they must lower to relation-shaping operations.

### Semantics

Generator output schema is part of the relation schema after the generator operation. Generators may preserve input columns, replace a nested column with generated columns, or produce a new relation depending on the function and syntax, but the behavior must be explicit.

### Interaction with other InQL surfaces

`query {}` may expose an `EXPLODE` clause or table-valued function syntax. Dataframe APIs may expose `.explode(...)` and related methods. Both must use the same generator semantics.

### Compatibility / migration

Existing unnest/explode behavior should align with this RFC. If current behavior differs, docs and diagnostics should prefer the generator/table-valued model rather than scalar-function wording.

## Alternatives considered

- **Model generators as scalar functions returning arrays.** Rejected because it does not change row cardinality and therefore cannot represent `explode`.
- **Allow generators anywhere a scalar expression is allowed.** Rejected because generator placement changes relation shape and must be constrained.
- **Only support `explode`.** Rejected because positional, struct-expanding, and warehouse `flatten` forms are common enough to design the class boundary now.

## Drawbacks

- Generators complicate schema flow and output type inference.
- Outer generator semantics require careful nullability rules.
- Backend support may differ for `inline`, `stack`, and tuple-producing helpers.

## Layers affected

- **InQL specification** — generator functions must be a relation-shaping class distinct from scalar functions.
- **InQL library package** — public APIs should expose generator operations with explicit output aliases.
- **Incan compiler** — query syntax must constrain generator placement and update relation schemas.
- **Execution / interchange** — Prism and Substrait lowering must represent cardinality changes and output schemas faithfully.
- **Documentation** — generator docs should explain cardinality and schema effects before listing helper names.

## Unresolved questions

- Should positional generators use zero-based or one-based positions?
- Should `.explode(...)` preserve all input columns by default?
- What aliasing syntax should be required for generated output columns?
- What subset of Snowflake-style `flatten` behavior belongs in portable InQL versus a warehouse compatibility extension?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
