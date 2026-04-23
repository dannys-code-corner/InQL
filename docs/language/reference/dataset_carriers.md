# Dataset carriers (Reference)

This page documents the carrier hierarchy only. Method details live in [Dataset methods](dataset_methods.md) so this page stays small as the API grows.

## Type hierarchy

```text
DataSet[T]                       (root trait — any tabular data)
├── BoundedDataSet[T]            (trait — finite extent)
│   ├── DataFrame[T]             (concrete — materialized/eager)
│   └── LazyFrame[T]             (concrete — deferred plan, bounded source)
└── UnboundedDataSet[T]          (trait — streaming/unbounded)
    └── DataStream[T]            (concrete — streaming)
```

## Carrier roles

### `DataSet[T]`

Root trait for any schema-parameterized tabular data whose row shape is an Incan `model` `T`.

### `BoundedDataSet[T]`

Finite extent. All relational operations are allowed.

### `UnboundedDataSet[T]`

Streaming or unbounded extent. Operations requiring unbounded retained state must be rejected statically.

### `DataFrame[T]`

Materialized/eager result. Always bounded.

### `LazyFrame[T]`

Deferred logical pipeline. Always bounded.

### `DataStream[T]`

Streaming specialization. Shares the `DataSet[T]` API while carrying unbounded semantics.

## Related reference pages

- [Dataset methods](dataset_methods.md)
- [Filter builders](builders/filters.md)
- [Aggregate builders](builders/aggregates.md)
- [Projection builders](builders/projections.md)
- [Execution context](execution_context.md)

## Related explanation

- [Dataset carriers (Explanation)](../explanation/dataset_carriers.md)
