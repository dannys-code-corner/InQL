# Dataset types (Reference)

This page documents the InQL dataset type hierarchy: the traits and concrete types that carry schema-parameterized tabular data through relational pipelines.

## Type hierarchy

```text
DataSet[T]                       (root trait — any tabular data)
├── BoundedDataSet[T]            (trait — finite extent)
│   ├── DataFrame[T]             (concrete — materialized/eager)
│   └── LazyFrame[T]             (concrete — deferred plan, bounded source)
└── UnboundedDataSet[T]          (trait — streaming/unbounded)
    └── DataStream[T]            (concrete — streaming)
```

### `DataSet[T]`

Root trait for any schema-parameterized tabular data whose row shape is an Incan `model` `T`.

All relational operations are defined on `DataSet[T]`. The compiler applies the **most restrictive** constraint set when the concrete kind is unknown at a call site (because the argument might be unbounded).

### `BoundedDataSet[T]`

Extends `DataSet[T]` — data with a finite, known extent. All relational operations are allowed.

### `UnboundedDataSet[T]`

Extends `DataSet[T]` — data from a streaming or unbounded source. Operations requiring unbounded state **must** be rejected at compile time.

### `DataFrame[T]`

Implements `BoundedDataSet[T]`. Materialized/eager result; always bounded. Conceptually the product of collecting or executing a `LazyFrame`.

### `LazyFrame[T]`

Implements `BoundedDataSet[T]`. Holds a logical plan (or equivalent) until an explicit execute, collect, or write boundary. Always bounded.

### `DataStream[T]`

Implements `UnboundedDataSet[T]`. Shares the `DataSet[T]` operation API but signals that its source is unbounded. The compiler applies static streaming constraints.

## Operation API

The following instance methods are defined on `DataSet[T]`:

| Method     | Signature                                       | Description                                                                                                                             |
| ---------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `filter`   | `def filter(self, predicate: bool) -> Self`     | Restrict rows by a boolean relational expression                                                                                        |
| `join`     | `def join(self, other: Self, on: bool) -> Self` | Combine with another relation on a join condition (`other: Self` at the trait level; see RFC 001 **Shipped trait signatures (`Self`)**) |
| `select`   | `def select(self) -> Self`                      | Project columns and expressions; logical output schema is tracked when lowering/typing (RFC 003)                                        |
| `group_by` | `def group_by(self) -> Self`                    | Define grouping keys for aggregation                                                                                                    |
| `agg`      | `def agg(self) -> Self`                         | Apply aggregate functions over groups; use imported helpers from `pub::inql.functions` (e.g. `total`, `count_rows`)                     |
| `order_by` | `def order_by(self) -> Self`                    | Define sort keys and directions                                                                                                         |
| `limit`    | `def limit(self, n: int) -> Self`               | Cap the number of rows (after sort when both apply)                                                                                     |
| `explode`  | `def explode(self) -> Self`                     | Expand a nested list column into rows                                                                                                   |

## Static capability gating

| Trait bound in signature | Allowed operations                                     | Constraint level                         |
| ------------------------ | ------------------------------------------------------ | ---------------------------------------- |
| `DataSet[T]`             | Intersection of bounded + unbounded capabilities       | Most restrictive (concrete kind unknown) |
| `BoundedDataSet[T]`      | All relational operations                              | Unrestricted                             |
| `UnboundedDataSet[T]`    | Relational operations minus unbounded-state operations | Streaming constraints enforced           |

## Usage

```incan
from pub::inql import LazyFrame, DataFrame, DataStream
from models import Order, Event

# Accept any carrier — generic utilities
def row_count[T](data: DataSet[T]) -> int:
    ...

# Batch only — Parquet writers, batch sinks
def write_parquet(data: BoundedDataSet[Order]) -> None:
    ...

# Streaming only — Kafka sinks, event processors
def write_to_kafka(events: UnboundedDataSet[Event]) -> None:
    ...

# Materialized data in hand
def inspect(data: DataFrame[Order]) -> None:
    ...

# Deferred plan — compose before execution
def build_pipeline(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    ...

# Streaming specifically
def process_stream(events: DataStream[Event]) -> DataStream[Event]:
    ...
```
