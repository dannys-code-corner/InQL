# InQL RFC 015: Core scalar functions and operators

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines the first required scalar function and operator slice for InQL: column references, literals, casts, comparisons, boolean logic, null checks, basic arithmetic, conditionals, membership predicates, range predicates, and ordering expressions. These functions are the minimum scalar vocabulary needed for credible typed dataframe and query-block authoring before broader math, string, and date/time catalogs are added.

## Motivation

InQL currently has a split helper surface for filters and projections. That is enough for early examples but too narrow for real dataframe work. Authors need one predictable core scalar vocabulary that can express common predicates, computed columns, grouping keys, aggregate arguments, and sort keys across all InQL authoring surfaces.

The core slice should be intentionally small. Functions such as advanced trigonometry, regex extraction, JSON parsing, and collection transforms are useful, but they should not delay the basic relational expression contract.

## Goals

- Define the required core scalar function set.
- Require consistent behavior across query blocks and dataframe method chains.
- Define strict and `try_` forms for casts and basic numeric failure modes.
- Define null-safe and ordinary comparison behavior.
- Establish ordering expression helpers with explicit null placement.

## Non-Goals

- Defining the full math, string, date/time, regex, or nested-data catalog.
- Defining aggregate functions.
- Defining window functions.
- Defining SQL string parsing.
- Replacing InQL RFC 012's canonical scalar expression model.

## Guide-level explanation (how authors think about it)

Authors should be able to write everyday filters and computed columns without switching helper families:

```incan
from pub::inql.functions import add, and_, cast, col, coalesce, gt, is_not_null, lit, lower, mul

enriched = (
    orders
        .filter(and_(is_not_null(col("customer_id")), gt(col("amount"), lit(0))))
        .with_column("amount_cents", mul(cast(col("amount"), "int"), lit(100)))
        .with_column("normalized_status", coalesce(lower(col("status")), lit("unknown")))
)
```

The exact surface may later gain operator sugar, but the semantic set should be available through importable functions first.

## Reference-level explanation (precise rules)

InQL must define canonical scalar entries for column reference, literal construction, cast, try-cast, equality, inequality, less-than, less-than-or-equal, greater-than, greater-than-or-equal, null-safe equality, boolean conjunction, boolean disjunction, boolean negation, null tests, NaN tests for floating types, addition, subtraction, multiplication, division, modulo, unary negation, `coalesce`, `nullif`, searched conditional expressions, membership predicates, range predicates, and ordering expressions.

Column references must resolve through the same schema rules used by InQL RFC 000 and InQL RFC 012. Literal construction must produce a scalar expression with a statically known literal type unless the literal type is intentionally dynamic.

Strict casts must fail when a value cannot be represented in the target type. `try_cast` must not raise the same value-conversion failure as strict `cast`; it must produce a null or other explicitly specified recoverable result.

Ordinary equality and comparison must follow SQL-style three-valued null behavior in relational predicates unless another RFC explicitly chooses a different policy. Null-safe equality must compare nulls as values according to its own registry entry.

Boolean operators must define three-valued logic for nullable boolean operands. If InQL exposes host-language operator sugar later, that sugar must preserve the same truth table.

Arithmetic functions must define numeric type promotion and overflow behavior. If exact overflow behavior remains backend-dependent in Draft, implementations must reject unsupported or ambiguous cases rather than silently changing semantics.

Ordering expressions must include ascending, descending, ascending-null-first, ascending-null-last, descending-null-first, and descending-null-last forms. Default null placement must be documented and must not vary silently by backend.

## Design details

### Syntax

This RFC requires importable function forms. Operator syntax may be added or mapped by existing query syntax, but the function registry entries are the semantic target.

### Semantics

The minimum core scalar catalog is:

- references and literals: `col`, `lit`
- casts: `cast`, `try_cast`
- comparisons: `eq`, `ne`, `lt`, `lte`, `gt`, `gte`, `equal_null`
- boolean logic: `and_`, `or_`, `not_`
- null and NaN predicates: `is_null`, `is_not_null`, `is_nan`, `is_not_nan`
- arithmetic: `add`, `sub`, `mul`, `div`, `mod`, `neg`
- conditionals: `coalesce`, `nullif`, `case_when`
- predicates: `in_`, `between`
- ordering: `asc`, `desc`, `asc_nulls_first`, `asc_nulls_last`, `desc_nulls_first`, `desc_nulls_last`

### Interaction with other InQL surfaces

`query {}` syntax may present SQL-familiar operators such as `==`, `>`, `AND`, `OR`, and `IS NULL`, but those operators must lower to the same scalar function semantics defined here. Dataframe methods must consume the same scalar expression model.

### Compatibility / migration

Existing helpers such as `int_expr`, `float_expr`, `str_expr`, `bool_expr`, `int_lit`, `str_lit`, and `bool_lit` may remain temporarily as compatibility shims to `lit`. Existing `add`, `mul`, `eq`, and `gt` should become registered core scalar entries.

## Alternatives considered

- **Wait for the broad function catalog.** Rejected because authors need a stable scalar core before broad catalog work can be coherent.
- **Keep separate filter and projection helper families.** Rejected by InQL RFC 012 because row-level scalar meaning should be unified.
- **Use backend SQL expression strings for all scalar expressions.** Rejected because it avoids typed expression checking.

## Drawbacks

- Defining null, cast, and numeric failure behavior exposes hard choices earlier.
- Compatibility shims may temporarily increase the number of public helper names.
- Operator sugar and function helper names can drift unless tooling treats registry entries as canonical.

## Layers affected

- **InQL specification** — the core scalar vocabulary must remain consistent with InQL RFC 012 and InQL RFC 014.
- **InQL library package** — public helpers should expose the core scalar entries and compatibility shims.
- **Incan compiler** — query syntax and any future operator sugar must lower to the same scalar function semantics.
- **Execution / interchange** — Prism and Substrait lowering must preserve casts, null-safe equality, boolean logic, ordering null placement, and `try_` behavior.
- **Documentation** — scalar function reference docs should distinguish canonical names from compatibility helpers.

## Unresolved questions

- Should the canonical boolean helper names be `and_`, `or_`, and `not_`, or should InQL expose different names because these collide with host-language keywords?
- Should `try_cast` return null on conversion failure, or should InQL eventually support a typed recoverable error result?
- What exact numeric promotion table should InQL use for mixed integer, decimal, and floating arithmetic?
- Should `in_` require literal lists initially, or should it also accept relation-valued subqueries in this RFC?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
