# InQL RFC 017: Aggregate modifiers

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 003 (`query {}` aggregate rules)
  - InQL RFC 012 (scalar expressions and aggregate measures)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 016 (core aggregate functions)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines aggregate modifiers for InQL: `DISTINCT`, aggregate-local `FILTER`, ordered aggregate input, and compatibility helpers such as `count_if`. These are modeled as modifiers on aggregate measures rather than as a separate function for every combination, so InQL can support SQL-style aggregate semantics without multiplying the catalog unnecessarily.

## Motivation

Many useful aggregate forms are not truly new aggregate functions. `count_distinct`, `sum_distinct`, `count_if`, and ordered string/list aggregates are better understood as an aggregate plus a modifier. If InQL implements each spelling as an unrelated function, the catalog becomes larger and less coherent while still failing to represent SQL's compositional aggregate shape.

DataFusion documents aggregate `FILTER (WHERE ...)` and ordered aggregate forms; Spark exposes many convenience names; Snowflake makes `COUNT_IF`, `LISTAGG`, ordered percentile forms, `MIN_BY`, and `MAX_BY` important warehouse-compatibility cases. InQL should support both a clean canonical model and compatibility aliases that map into it.

## Goals

- Define aggregate `DISTINCT`.
- Define aggregate-local filtering.
- Define ordered aggregate input.
- Define how compatibility helpers map to modifiers.
- Require modifier support to preserve scalar-expression and aggregate-measure boundaries.

## Non-Goals

- Defining every aggregate that can use these modifiers.
- Defining window functions.
- Defining streaming triggers, watermarks, or accumulation modes.
- Defining SQL parser surface for every modifier spelling.

## Guide-level explanation (how authors think about it)

Authors should be able to express distinct and filtered aggregates without learning separate function names for every combination:

```incan
from pub::inql.functions import col, count, sum

summary = (
    orders
        .group_by([col("customer_id")])
        .agg([
            count().filter(col("is_completed")),
            count(col("product_id")).distinct(),
            sum(col("amount")).filter(col("is_completed")),
        ])
)
```

A query-block surface may use SQL-like spelling, but it should describe the same aggregate modifier model.

## Reference-level explanation (precise rules)

An aggregate modifier must attach to an aggregate measure. It must not turn an aggregate measure into a row-level scalar expression.

`DISTINCT` must remove duplicate non-null aggregate input values before the aggregate computes its result. For multi-argument aggregates, distinctness must apply to the tuple of aggregate inputs unless the aggregate's registry entry defines a narrower rule.

Aggregate-local `FILTER` must evaluate a boolean scalar expression for each input row before that row contributes to the aggregate. Rows where the filter is false or null must not contribute to the aggregate. The filter expression must be evaluated in the row-level scope of the aggregate input.

Ordered aggregate input must define one or more ordering expressions and optional null placement. Ordered aggregate input must be valid only for aggregate functions whose registry entry allows ordered input or for aggregate functions where ordering is semantically meaningful.

Compatibility helpers must lower to modifiers when possible. `count_distinct(expr)` should be equivalent to `count(expr).distinct()`. `count_if(predicate)` should be equivalent to `count().filter(predicate)` unless a later RFC chooses different null behavior. Ordered string aggregation names such as `listagg` should lower to ordered aggregate semantics rather than a separate non-composable function family.

If a backend cannot support a modifier faithfully, InQL must report an explicit diagnostic or use a semantics-preserving fallback. It must not ignore the modifier.

## Design details

### Syntax

This RFC does not require one final public syntax. It permits method-like aggregate modifiers, query-block SQL modifiers, or explicit helper functions if they lower to the same aggregate-measure model.

### Semantics

Modifiers are part of the aggregate measure. Modifier order must be semantically defined. Filtering happens before distinctness unless a specific aggregate entry defines otherwise; ordering applies to the input sequence seen by order-sensitive aggregates after filtering and distinct handling.

### Interaction with other InQL surfaces

`query {}` blocks may expose `COUNT(DISTINCT .id)` and `sum(.amount) FILTER (WHERE .status == "paid")` if the grammar supports those spellings. Dataframe method chains may expose method-like modifier builders. Both must produce the same aggregate-measure semantics.

### Compatibility / migration

Existing aggregate helpers remain valid. New compatibility helpers such as `count_distinct` and `count_if` may be added as aliases or sugar over modifiers, but docs should teach modifiers as the canonical model.

## Alternatives considered

- **One function per aggregate combination.** Rejected because it creates avoidable catalog sprawl and weakens composability.
- **Only SQL syntax, no method-chain modifier form.** Rejected because dataframe authoring also needs aggregate modifiers.
- **Ignore unsupported modifiers on limited backends.** Rejected because it silently changes query results.

## Drawbacks

- Aggregate measures become richer values with their own internal structure.
- Ordered aggregate semantics require a clear ordering model before every backend can support them.
- Compatibility helpers can obscure the canonical model if documented poorly.

## Layers affected

- **InQL specification** — aggregate modifiers must compose with aggregate measures without violating the scalar-versus-aggregate boundary.
- **InQL library package** — aggregate builders should expose modifier construction or compatibility helpers.
- **Incan compiler** — query-block aggregate syntax must lower modifiers faithfully where supported.
- **Execution / interchange** — Prism and Substrait lowering must preserve filter, distinct, and ordering semantics or reject unsupported forms.
- **Documentation** — aggregate docs should prefer the modifier model and list compatibility helper aliases.

## Unresolved questions

- Should `count_if(null)` count zero rows or follow a stricter boolean-null diagnostic rule?
- Which aggregate functions must allow ordered input in the initial modifier contract, especially `listagg`, `percentile_cont`, and `percentile_disc`?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
