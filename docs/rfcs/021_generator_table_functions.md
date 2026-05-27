# InQL RFC 021: Generator and table-valued functions

- **Status:** Implemented
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset carriers and relation operations)
  - InQL RFC 003 (`query {}` clause inventory)
  - InQL RFC 006 (unnest/explode Substrait lowering)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 020 (nested data functions)
- **Issue:** [InQL #38](https://github.com/dannys-code-corner/InQL/issues/38)
- **RFC PR:** [InQL #47](https://github.com/dannys-code-corner/InQL/pull/47)
- **Written against:** Incan v0.2
- **Shipped in:** v0.1

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

Authors should use generators when one input row may become multiple output rows. In the current builder surface,
generators are constructed as explicit applications and then applied to a relation:

```incan
from pub::inql.functions import col, explode

items = (
    orders
        .generate(explode(col("line_items"), "line_item"))
        .select(["order_id", "line_item"])
)
```

The result has a different relation shape from the input. This is not the same kind of expression as `array_contains(...)` or `cardinality(...)`.

## Reference-level explanation (precise rules)

Generator functions must be registry entries with function class `generator` or `table-valued`. They must not be valid in ordinary scalar expression positions.

`explode(array_expr)` must produce one output row for each element of the array expression. If the input array is null or empty, the non-outer form must produce zero rows for that input row.

`explode_outer(array_expr)` must preserve the input row when the input array is null or empty and must produce a null generated value according to its output schema.

`posexplode(array_expr)` and `posexplode_outer(array_expr)` must include a positional output column in addition to the generated element. Positional output is zero-based because `posexplode` follows the Spark-compatible naming convention rather than InQL's one-based scalar collection indexing rule.

`inline(array_of_struct_expr)` must expand each struct element into output columns. `inline_outer` must preserve outer rows for null or empty input according to the outer generator rule.

`stack` must construct multiple output rows from explicit expressions according to a declared row count and output schema.

`flatten` is a table-valued/generator operation in the portable one-array form. Snowflake-style recursive/path flattening is not part of the portable core; scalar `array_flatten(...)` remains part of RFC 020 and does not change row cardinality.

Every generator must define output column names, output types, nullability, interaction with existing columns, and aliasing requirements. Name collisions must be diagnosed unless an explicit overwrite or qualification rule applies.

## Design details

### Syntax

Generators may appear as dataframe relation methods, query-block clauses, or table-valued function forms. Regardless of syntax, they must lower to relation-shaping operations. The initial builder API uses `generate(generator)` to avoid overloading the existing zero-argument compatibility `explode()` method.

### Semantics

Generator output schema is part of the relation schema after the generator operation. The initial portable generator applications preserve all input columns and append generated output columns in declaration order. Generated aliases are required, must be non-empty, and must not collide with existing columns.

### Interaction with other InQL surfaces

`query {}` may expose an `EXPLODE` clause or table-valued function syntax when the query surface is available. Dataframe APIs expose the same semantic target through `generate(...)` and registry-backed generator helpers. Both use the same generator semantics.

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

## Design Decisions

### Resolved

- Positional generators use zero-based positions for compatibility with the `posexplode` naming convention.
- Explicit generator applications preserve all input columns by default and append generated output columns.
- Generated aliases are required at builder construction time.
- Snowflake-style recursive/path `flatten` remains outside the portable core until its output schema and compatibility category are specified separately.
- `explode`, `explode_outer`, `posexplode`, `posexplode_outer`, `inline`, `inline_outer`, portable `flatten`, and `stack` are implemented as registry-backed generator applications with Substrait relation-extension metadata and DataFusion-backed Session execution.

### Remaining

- No RFC 021 generator semantics remain open. Query-block syntax itself is owned by RFC 003; when that surface lands, its generator clauses must lower to the implemented `GeneratorApplication` model rather than defining a separate generator path.
