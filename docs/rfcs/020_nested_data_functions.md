# InQL RFC 020: Nested data functions

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (schema shapes and model-driven typing)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 021 (generator and table-valued functions)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines InQL functions for nested scalar values: arrays, maps, and structs. It covers construction, element access, cardinality, containment, sorting, set-like array operations, map entry access, and higher-order collection functions as a later extension point. Nested functions remain scalar when they produce one value per input row; cardinality-changing operations such as `explode` belong to a separate generator RFC.

## Motivation

Modern dataframe and warehouse systems routinely handle nested data. Spark has a large array/map/struct catalog, Snowflake has ARRAY/OBJECT/MAP and semi-structured VARIANT-oriented functions, Arrow and DataFusion support nested physical types, and Beam schemas support nested rows and collections. InQL needs nested data functions for realistic semi-structured data without confusing scalar nested values with relation-shaping generators.

The split matters. `array_contains(.items, "x")` is a row-level scalar predicate. `explode(.items)` changes the number of rows and must be modeled differently.

## Goals

- Define scalar functions for arrays, maps, and structs.
- Distinguish nested scalar operations from generators.
- Define element access and safe element access.
- Define collection size, containment, sorting, and set-like operations.
- Leave lambda-based higher-order functions as a later design decision unless the host language surface is ready.

## Non-Goals

- Defining `explode`, `posexplode`, `inline`, or other cardinality-changing functions.
- Defining JSON, CSV, XML, or variant value parsing.
- Defining geospatial or sketch collection types.
- Defining a full lambda syntax for collection transforms.

## Guide-level explanation (how authors think about it)

Authors should be able to inspect and manipulate nested values without changing relation cardinality:

```incan
from pub::inql.functions import array_contains, cardinality, col, element_at, map_keys

enriched = (
    events
        .filter(array_contains(col("tags"), "purchase"))
        .with_column("tag_count", cardinality(col("tags")))
        .with_column("first_item", element_at(col("items"), 1))
        .with_column("metadata_keys", map_keys(col("metadata")))
)
```

If an author wants one output row per item, that is a generator/table-valued operation rather than a nested scalar function.

## Reference-level explanation (precise rules)

InQL should define array construction with `array`, struct construction with `struct` or `named_struct`, and map construction with `create_map` or an equivalent canonical name.

InQL should define `cardinality` as the canonical size function for arrays and maps. Compatibility aliases such as `size`, `array_size`, and `array_length` may resolve to `cardinality` where semantics match.

InQL should define element access functions including `element_at`, `try_element_at`, and `get`. Strict element access must fail or diagnose according to its registry error policy when an index or key is invalid. `try_element_at` must produce the recoverable result defined by its registry entry.

InQL should define array predicates and transforms including `array_contains`, `array_position`, `array_sort`, `array_distinct`, `array_except`, `array_intersect`, `array_union`, `array_join`, `arrays_overlap`, `flatten`, `slice`, and `reverse` where type and null semantics are specified.

InQL should define map functions including `map_contains_key`, `map_entries`, `map_from_arrays`, `map_from_entries`, `map_keys`, and `map_values`.

InQL should account for object-style warehouse functions such as `object_construct`, `object_construct_keep_null`, `object_delete`, `object_insert`, `object_keys`, and `object_pick`. These should be modeled through typed object/map semantics where possible and through a variant/semi-structured family only when dynamic value semantics are required.

Higher-order functions such as `transform`, `filter`, `exists`, `forall`, `aggregate`, `reduce`, `zip_with`, `map_filter`, `transform_keys`, and `transform_values` must not reach Planned status until lambda or equivalent callback semantics are specified for InQL expressions.

## Design details

### Syntax

This RFC requires importable function forms. Literal syntax for arrays, maps, or structs may be defined by another RFC and must lower to these semantic concepts where applicable.

### Semantics

Nested scalar functions produce one value per input row. A function that changes row count, expands fields into multiple columns, or returns a relation belongs to InQL RFC 021.

Index origin, invalid-index behavior, null container behavior, null element behavior, and duplicate-map-key behavior must be specified before the corresponding functions reach Planned status.

### Interaction with other InQL surfaces

Nested functions may appear wherever scalar expressions of their result type are valid. Grouping by nested values may be restricted until equality and ordering semantics for nested values are fully specified.

### Compatibility / migration

No current InQL APIs are expected to break. Nested functions should be additive and gated by type support.

## Alternatives considered

- **Bundle nested functions with JSON or VARIANT functions.** Rejected because typed arrays/maps/structs are a core data model feature, while JSON parsing and dynamic variant values are format/value concerns.
- **Treat every collection operation as a generator.** Rejected because many collection operations are scalar row-level operations.
- **Add higher-order functions immediately.** Rejected until expression-lambda semantics are clear.

## Drawbacks

- Nested types require more precise type and nullability tracking.
- Backend support can differ sharply for nested operations.
- Index origin and invalid-access policy can surprise authors if aliases from different ecosystems are mixed carelessly.

## Layers affected

- **InQL specification** — nested scalar functions must fit the scalar expression model without changing relation cardinality.
- **InQL library package** — public helpers should expose nested constructors, accessors, and predicates.
- **Incan compiler** — typechecking must understand nested collection, map, and struct result types where functions are used.
- **Execution / interchange** — Prism and Substrait lowering must preserve nested value semantics or diagnose unsupported operations.
- **Documentation** — docs should separate nested scalar operations from generator functions.

## Unresolved questions

- Should element access use one-based indexing for SQL/Spark compatibility or zero-based indexing for host-language familiarity?
- What should strict `element_at` do on out-of-range indexes?
- Should grouping and ordering over arrays, maps, and structs be allowed initially?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
