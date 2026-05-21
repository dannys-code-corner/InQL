# InQL examples

Examples demonstrating InQL dataset types, scalar-expression builders, and Session execution patterns.

## Overview

The examples are split between compile-safe API shape examples and executable Session flows. Together they show how typed carriers, scalar expressions, grouped aggregates, reads, writes, collection, and display fit together in ordinary Incan code.

## Example structure

- `dataset_api.incn` — Demonstrates the DataSet[T] operation API
- `trait_hierarchy.incn` — Demonstrates trait hierarchy usage
- `bounded_vs_unbounded.incn` — Demonstrates bounded vs unbounded type signatures
- `session_read_transform_write_csv.incn` — Demonstrates `Session.read_csv(name, uri) -> LazyFrame transform -> Session.write_csv(...) -> session.activate() -> display(...)`
- `session_read_transform_write_order_lines_csv.incn` — Same flow with a realistic multi-column `OrderLine` model and fixture
- `session_grouped_aggregate_csv.incn` — Real grouped aggregate example over CSV using `col(...)`, `sum(...)`, and `count()`
- `session_with_column_csv.incn` — Real derived-column example over CSV using `with_column(...)`, `mul(...)`, and `lit(...)`
- `models.incn` — Shared row models for examples

## Running examples

```bash
incan run examples/dataset_api.incn
incan run examples/session_read_transform_write_csv.incn
incan run examples/session_read_transform_write_order_lines_csv.incn
incan run examples/session_grouped_aggregate_csv.incn
incan run examples/session_with_column_csv.incn
```

> Note: Session examples expect repo fixtures in `tests/fixtures/` and write output files to `tests/target/`.

## What these examples show

These examples document the API patterns for the InQL dataset and Session surface:

1. **RFC 001** contracts are represented as compile-safe signatures and trait assignments
2. Builder-based aggregation runs through `col(...)`, `sum(...)`, and `count()`
3. Builder-based scalar expressions run through `col(...)`, `lit(...)`, `eq(...)`, `gt(...)`, `add(...)`, and `mul(...)`
4. **RFC 004** provides execution behavior (`execute`, `collect`, and write sinks over DataFusion)

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
- **Convenience authoring** — these examples use the explicit builder surface that concise query and operator surfaces lower into.
