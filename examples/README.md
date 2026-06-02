# InQL examples

Examples demonstrating InQL model-shaped dataset types, scalar-expression builders, and Session execution patterns.

## Overview

The examples are split between compile-safe API shape examples and executable Session flows. Together they show how model
types, typed carriers, scalar expressions, grouped aggregates, reads, writes, collection, and display fit together in
ordinary Incan code.

## Example structure

- `dataset_api.incn` — Demonstrates typed DataSet[T] pipeline helper shapes
- `trait_hierarchy.incn` — Demonstrates trait hierarchy usage
- `bounded_vs_unbounded.incn` — Demonstrates bounded vs unbounded type signatures
- `session_read_transform_write_csv.incn` — Demonstrates `Session.read_csv(...) -> LazyFrame[OrderId] -> transform -> write/display`
- `session_read_transform_write_order_lines_csv.incn` — Same flow with a realistic multi-column `OrderLine` model and fixture
- `session_grouped_aggregate_csv.incn` — Grouped aggregate over `LazyFrame[AggregateOrder]` using `col(...)`, `sum(...)`, and `count()`
- `session_with_column_csv.incn` — Derived-column example over `LazyFrame[AggregateOrder]` using `with_column(...)`, `mul(...)`, and `lit(...)`
- `models.incn` — Shared `@derive(Clone)` row models for examples

## Running examples

```bash
incan run examples/dataset_api.incn
incan run examples/trait_hierarchy.incn
incan run examples/bounded_vs_unbounded.incn
incan run examples/session_read_transform_write_csv.incn
incan run examples/session_read_transform_write_order_lines_csv.incn
incan run examples/session_grouped_aggregate_csv.incn
incan run examples/session_with_column_csv.incn
```

> Note: Session examples expect repo fixtures in `tests/fixtures/` and write output files to `tests/target/`.

## What these examples show

These examples document the API patterns for the InQL dataset and Session surface:

1. Model contracts are visible at source boundaries as `LazyFrame[OrderId]`, `LazyFrame[OrderLine]`, and `LazyFrame[AggregateOrder]`
2. Carrier transformations remain typed Incan functions rather than stringly runtime scripts
3. Builder-based aggregation runs through `col(...)`, `sum(...)`, and `count()`
4. Builder-based scalar expressions run through `col(...)`, `lit(...)`, `eq(...)`, `gt(...)`, `add(...)`, and `mul(...)`
5. Session execution provides `collect`, `display`, and write sinks over DataFusion

They serve three purposes:

- **Regression tests** — verifying the patterns remain valid
- **Documentation** — showing users how to use the API
- **Examples** — providing starting points for real code

## Incan status

- **RFC 041** (First-Class Rust Interop Authoring): Implemented in Incan v0.2
- **RFC 042** (Traits Are Always Abstract): Implemented in Incan v0.2

These RFCs provide the trait and interop foundation InQL builds on.

## Scope boundaries

- **Materialized row access** — Session collection exposes typed `DataFrame[T]` materialization metadata and preview text; row iteration/accessor design belongs to the DataFrame API work.
- **Output row retargeting** — `select[U](...)` preserves the carrier kind while allowing the projected row model to change.
- **Convenience authoring** — these examples use the explicit builder surface that concise query and operator surfaces lower into.
