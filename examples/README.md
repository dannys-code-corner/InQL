# InQL examples

Examples demonstrating InQL model-shaped dataset types, scalar-expression builders, and Session execution patterns.

## Overview

The examples are split between compile-safe API shape examples and executable Session flows. Together they show how model types, typed carriers, scalar expressions, grouped aggregates, reads, writes, collection, and display fit together in ordinary Incan code.

## Example structure

- `dataset_api.incn` — Demonstrates typed DataSet[T] pipeline helper shapes
- `trait_hierarchy.incn` — Demonstrates trait hierarchy usage
- `bounded_vs_unbounded.incn` — Demonstrates bounded vs unbounded type signatures
- `session_read_transform_write_csv.incn` — Demonstrates `Session.read_csv(...) -> LazyFrame[OrderId] -> transform -> write/display`
- `session_read_transform_write_order_lines_csv.incn` — Same flow with a realistic multi-column `OrderLine` model and fixture
- `session_grouped_aggregate_csv.incn` — Grouped aggregate over `LazyFrame[AggregateOrder]` using `col(...)`, `sum(...)`, and `count()`
- `session_with_column_csv.incn` — Derived-column example over `LazyFrame[AggregateOrder]` using `with_column(...)`, `mul(...)`, and `lit(...)`
- `advanced_retail_analytics.incn` — Larger 100-row retail method-chain spike covering scalar functions, JSON, URL parsing, hashing, aggregates, windows, and generators
- `advanced_retail_query_blocks/` — Dependency-consumer query-block version of the retail spike, covering the
  query-block vocabulary over the same 100-row fixture
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
incan run examples/advanced_retail_analytics.incn
(cd examples/advanced_retail_query_blocks && incan lock && incan run src/main.incn)
```

> Note: Session examples expect repo fixtures in `tests/fixtures/` and write output files to `tests/target/`.

## Advanced spike

`advanced_retail_analytics.incn` reads `tests/fixtures/advanced_retail_orders.csv`, a 100-row CSV fixture with quoted JSON event payloads. It materializes three outputs:

- an enriched high-value order view with string cleanup, math, date extraction, JSON validation/extraction, URL query
  extraction, hashing, regex, and nested array helpers
- a grouped paid-order rollup using `sum`, `avg`, `min`, `max`, `count`, and `count_distinct`
- a generated tag view that composes window ranking with `explode(...)`

`advanced_retail_query_blocks/` is the same fixture exercised from a standalone dependency consumer. It imports `pub::inql` and runs query blocks for the high-value projection, grouped rollup, and generated-tag window view:

```incan
high_value = query {
    FROM paid
    SELECT
        .order_id as order_id,
        .customer_id as customer_id,
        .region_norm as region_norm,
        .net_amount as net_amount,
        .campaign as campaign,
        .channel as channel,
    WHERE .net_amount > 100
    ORDER BY desc(.net_amount)
    LIMIT 8
}

rollup = query {
    FROM enriched
    WHERE eq(.status_norm, "paid")
    GROUP BY .region_norm, .channel
    SELECT
        .region_norm as region_norm,
        .channel as channel,
        sum(.net_amount) as total_net_amount,
        avg(.net_amount) as avg_net_amount,
        count() as order_count
    ORDER BY desc(.total_net_amount)
}
```

## What these examples show

These examples document the API patterns for the InQL dataset and Session surface:

1. Model contracts are visible at source boundaries as `LazyFrame[OrderId]`, `LazyFrame[OrderLine]`, and `LazyFrame[AggregateOrder]`
2. Carrier transformations remain typed Incan functions rather than stringly runtime scripts
3. Builder-based aggregation runs through `col(...)`, `sum(...)`, and `count()`
4. Builder-based scalar expressions run through `col(...)`, `lit(...)`, `eq(...)`, `gt(...)`, `add(...)`, and `mul(...)`
5. Query blocks activate through `pub::inql` in dependency consumers, desugar into carrier calls, and meet the rest of
   InQL at the Substrait boundary
6. Session execution provides `collect`, `display`, and write sinks over DataFusion

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
