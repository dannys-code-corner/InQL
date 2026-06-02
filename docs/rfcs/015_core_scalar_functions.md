# InQL RFC 015: Core scalar functions and operators

- **Status:** Implemented
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
- **Issue:** [InQL #32](https://github.com/dannys-code-corner/InQL/issues/32)
- **RFC PR:** [InQL #43](https://github.com/dannys-code-corner/InQL/pull/43)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** v0.1

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
from pub::inql.functions import add, and_, cast, col, coalesce, gt, is_not_null, lit, mul

enriched = (
    orders
        .filter(and_(is_not_null(col("customer_id")), gt(col("amount"), lit(0))))
        .with_column("amount_cents", mul(cast(col("amount"), "int"), lit(100)))
        .with_column("status_or_unknown", coalesce([col("status"), lit("unknown")]))
)
```

The exact surface may later gain operator sugar, but the semantic set should be available through importable functions first.

## Reference-level explanation (precise rules)

InQL must define canonical scalar entries for column reference, literal construction, cast, try-cast, equality, inequality, less-than, less-than-or-equal, greater-than, greater-than-or-equal, null-safe equality, boolean conjunction, boolean disjunction, boolean negation, null tests, NaN tests for floating types, addition, subtraction, multiplication, division, modulo, unary negation, `coalesce`, `nullif`, searched conditional expressions, membership predicates, range predicates, and ordering expressions.

Column references must resolve through the same schema rules used by InQL RFC 000 and InQL RFC 012. Literal construction must produce a scalar expression with a statically known literal type unless the literal type is intentionally dynamic.

Strict casts must fail when a value cannot be represented in the target type. `try_cast` must not raise the same value-conversion failure as strict `cast`; it must produce a null or other explicitly specified recoverable result.

Ordinary equality and comparison must follow SQL-style three-valued null behavior in relational predicates unless another RFC explicitly chooses a different policy. Null-safe equality must compare nulls as values according to its own registry entry.

Boolean operators must define three-valued logic for nullable boolean operands. If InQL exposes host-language operator sugar later, that sugar must preserve the same truth table.

Arithmetic functions must define numeric type promotion and overflow behavior. If exact overflow behavior remains backend-dependent in Draft, implementations must reject ambiguous cases rather than silently changing semantics.

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

### Surface cleanup

Typed literal helpers such as `int_expr`, `float_expr`, `str_expr`, `bool_expr`, `int_lit`, `str_lit`, and `bool_lit` should route through `lit` or the same scalar-literal representation. Existing `add`, `mul`, `eq`, and `gt` should become registered core scalar entries.

## Alternatives considered

- **Wait for the broad function catalog.** Rejected because authors need a stable scalar core before broad catalog work can be coherent.
- **Keep separate filter and projection helper families.** Rejected by InQL RFC 012 because row-level scalar meaning should be unified.
- **Use backend SQL expression strings for all scalar expressions.** Rejected because it avoids typed expression checking.

## Drawbacks

- Defining null, cast, and numeric failure behavior exposes hard choices earlier.
- Typed helper entrypoints may increase the number of public helper names.
- Operator sugar and function helper names can drift unless tooling treats registry entries as canonical.

## Layers affected

- **InQL specification** — the core scalar vocabulary must remain consistent with InQL RFC 012 and InQL RFC 014.
- **InQL library package** — public helpers should expose the core scalar entries and selected typed helper entrypoints.
- **Incan compiler** — query syntax and any future operator sugar must lower to the same scalar function semantics.
- **Execution / interchange** — Prism and Substrait lowering must preserve casts, null-safe equality, boolean logic, ordering null placement, and `try_` behavior.
- **Documentation** — scalar function reference docs should distinguish canonical names from typed helper entrypoints.

## Implementation Plan

### Phase 1: Registry-backed scalar application model

- Keep structural scalar nodes for column references and typed literals.
- Replace bespoke scalar function/operator expression variants with one registry-backed scalar function application node.
- Ensure public function kind and mapping metadata come from registry entries rather than a parallel function-kind switchboard.

### Phase 2: Public core scalar helpers and registry metadata

- Add the RFC 015 public helper names in one-helper-per-module `src/functions/<family>/<name>.incn` files using declaration-side `@register_function(...)` decorators.
- Keep non-derivable machine metadata in decorator specs; derive names and signatures from checked helper declarations.
- Preserve typed literal helper compatibility while routing through the canonical literal representation.
- Use Substrait mapping metadata only for real IR lowering: extension functions, built-in Rex shapes, deterministic rewrites, or structural relation contexts such as sort fields.

### Phase 3: Lowering and diagnostics

- Resolve scalar function application lowering through registry mapping metadata.
- Preserve existing correct lowerings for the scalar functions already represented by current Substrait extension mappings.
- Add registry-driven extension mappings, built-in Rex lowerings, and structural sort-field lowerings for the rest of the core scalar slice.
- Return clear invalid-context diagnostics when structural helpers such as `asc(...)` are used outside their valid query context.

### Phase 4: Tests and docs

- Add tests for helper imports, expression shape, registry metadata, scalar lowering, structural ordering lowering, invalid-context diagnostics, and literal helper cleanup.
- Update user-facing function docs and release notes.
- Run the registry metadata check because it protects the RFC 014 declaration-side registry contract.

## Progress Checklist

### Spec / lifecycle

- [x] RFC 014 registry baseline is merged and available as the implementation baseline.
- [x] RFC 015 issue exists and is linked.
- [x] RFC 015 moved to In Progress with implementation plan and checklist.
- [x] Design Decisions record the current implementation-slice answers.

### Expression model

- [x] Keep structural scalar nodes for `ColumnRefExpr` and literals.
- [x] Replace bespoke function/operator expression variants with registry-backed scalar function application.
- [x] Make public expression kind/function metadata derive from the registry-backed application.
- [x] Preserve typed literal helper compatibility through the canonical literal representation.

### Public helpers / registry

- [x] Register `cast` and `try_cast`.
- [x] Register comparisons: `eq`, `ne`, `lt`, `lte`, `gt`, `gte`, `equal_null`.
- [x] Register boolean logic: `and_`, `or_`, `not_`.
- [x] Register null and NaN predicates: `is_null`, `is_not_null`, `is_nan`, `is_not_nan`.
- [x] Register arithmetic: `add`, `sub`, `mul`, `div`, `mod`, `neg`.
- [x] Register conditionals: `coalesce`, `nullif`, `case_when`.
- [x] Register predicates: `in_`, `between`.
- [x] Register ordering helpers: `asc`, `desc`, `asc_nulls_first`, `asc_nulls_last`, `desc_nulls_first`, `desc_nulls_last`.

### Lowering / interchange

- [x] Resolve supported scalar function lowering through registry mapping metadata.
- [x] Keep existing supported Substrait extension lowering for `add`, `mul`, `eq`, and `gt`.
- [x] Add honest current lowerings for casts, comparisons, boolean logic, null/NaN predicates, arithmetic, conditionals, membership predicates, and range predicates.
- [x] Emit invalid-context diagnostics for ordering helpers used as standalone scalar expressions.
- [x] Lower ordering helpers into Substrait `SortRel.sorts` when used through `order_by(...)`.

### Tests

- [x] Registry tests cover all RFC 015 helpers and mapping categories.
- [x] Scalar expression tests prove helper calls share one function application node.
- [x] Lowering tests cover scalar helpers, structural ordering helpers, and invalid ordering contexts.
- [x] Regression tests prove adding one scalar function does not require separate kind/lowering switchboards.

### Docs / release

- [x] Update function reference docs for the core scalar helper surface.
- [x] Update release notes.
- [x] Confirm no package version bump is required for the unreleased v0.1 package line.

### Verification

- [x] `make test-style`
- [x] focused scalar/function registry/Substrait tests
- [x] `make fmt-check`
- [x] `make registry-metadata`
- [x] `make build`
- [x] `make test`
- [x] `make smoke-consumer`

## Design Decisions

- **Boolean helper names:** the canonical boolean helper names are `and_`, `or_`, and `not_`; the trailing underscore avoids host-language keyword collisions while keeping SQL-familiar names recognizable.
- **`try_cast` failure result:** `try_cast` uses null-on-conversion-failure semantics for this RFC. Typed recoverable error values remain a future extension point rather than part of the core scalar slice.
- **Numeric promotion boundary:** the current v0.3-era InQL package records numeric helper intent and checked helper signatures but does not introduce a package-local numeric promotion table. Lowering must only use mappings that are currently represented honestly; ambiguous numeric behavior must remain explicit instead of silently choosing backend-dependent behavior.
- **`in_` scope:** `in_` accepts literal or expression lists in this RFC. Relation-valued subquery membership is a future query-surface feature and is out of scope for this core scalar package slice.
- **Registry-backed applications:** structural scalar nodes remain appropriate for `ColumnRefExpr` and typed literals, but function/operator calls such as `add`, `mul`, `eq`, `gt`, `and_`, `or_`, and `cast` are represented as registry-backed scalar function applications rather than one bespoke model per function forever.
- **Current lowering boundary:** the implemented RFC 015 slice lowers through registered Substrait extension mappings, built-in Substrait Rex shapes (`Cast`, `SingularOrList`, and `IfThen`), and structural `SortRel.sorts` lowering for ordering helpers. DataFusion is the first execution adapter that consumes the emitted Substrait plan; it does not define the portable helper semantics.
- **Module layout:** each public helper lives in its own `src/functions/<family>/<name>.incn` module with helper-local docs, decorator metadata, and inline tests. Family directories group references, literals, casts, operators, predicates, conditionals, ordering, aggregates, and formatting helpers for readable source ownership and future generated docs. `src/functions/mod.incn` remains the public import facade; all-surface catalog validation uses checked API metadata projections rather than runtime loader hooks.
- **Modulo spelling:** `mod` remains the canonical registry function name, but the Incan public helper is `modulo(...)` because `mod` is reserved by the language.
