# Dataset types (Explanation)

This page explains how to think about and use InQL's dataset types.

## Why dataset types?

Typed pipelines need a first-class carrier for columnar data indexed by `T`. Without `DataSet[T]`, relational authoring surfaces would lack a stable primary relation and schema flow for `FROM`-style entry points.

The **bounded/unbounded** distinction — inspired by Spark Structured Streaming's principle that a stream is an unbounded table — must be expressed at the **type level** so the compiler can enforce streaming constraints statically rather than at runtime.

## The core idea

A `DataSet[T]` is a **schema-parameterized tabular carrier**:

- `T` is an Incan `model` — the row schema
- The carrier holds tabular data with that schema
- Operations like `filter`, `join`, `select` transform the carrier

## Bounded vs unbounded

The key insight is that **a stream is an unbounded table**. Rather than defining separate operation APIs for batch and streaming, `DataSet[T]` provides one relational operation surface. The bounded/unbounded property is expressed through the type system:

- **`BoundedDataSet[T]`** — finite extent, all operations allowed
- **`UnboundedDataSet[T]`** — streaming/unbounded, unbounded-state operations rejected at compile time

This enables **static capability gating**: operations that require unbounded state are rejected at compile time when the target is unbounded, without requiring a separate streaming API.

## When to use which type

### `DataFrame[T]` — materialized/eager

Use `DataFrame[T]` when you have data in hand and want to inspect or manipulate it directly:

```incan
from pub::inql import DataFrame
from models import Order

def inspect_orders(orders: DataFrame[Order]) -> None:
    # Work with materialized data
    pass
```

`DataFrame[T]` is always bounded — it's the product of collecting or executing a `LazyFrame`.

### `LazyFrame[T]` — deferred plan

Use `LazyFrame[T]` when you want to compose operations before execution:

```incan
from pub::inql import LazyFrame
from models import Order

def high_value_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    # Intended when query syntax is available: return orders.filter(.amount > 100)
    return orders
```

### `DataStream[T]` — streaming

Use `DataStream[T]` for streaming/unbounded data:

```incan
from pub::inql import DataStream
from models import Event

def important_events(events: DataStream[Event]) -> DataStream[Event]:
    # Intended when query syntax is available: return events.filter(.severity == "critical")
    return events
```

`DataStream[T]` shares the same operation API as batch carriers, but signals that its source is unbounded. Static streaming constraints are specified in RFC 001 and enforced as the compiler gains analysis for `UnboundedDataSet[T]`.

## Type signatures

The trait hierarchy gives you three levels of specificity:

```incan
from pub::inql import DataSet, BoundedDataSet, UnboundedDataSet
from models import Order, Event

# Accepts any carrier — generic utilities
def row_count[T](data: DataSet[T]) -> int:
    ...

# Batch only — Parquet writers, batch sinks
def write_parquet(data: BoundedDataSet[Order]) -> None:
    ...

# Streaming only — Kafka sinks, event processors
def write_to_kafka(events: UnboundedDataSet[Event]) -> None:
    ...
```

And two levels of concrete-type specificity:

```incan
from pub::inql import DataFrame, LazyFrame, DataStream
from models import Order, Event

# Materialized data in hand
def inspect(data: DataFrame[Order]) -> None:
    ...

def build_pipeline(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    ...

def process_stream(events: DataStream[Event]) -> DataStream[Event]:
    ...
```

## Aggregate helpers

`.agg(...)` uses **imported** symbols from `pub::inql.functions` (for example `total`, `count_rows`).

<!-- FIXME: remove this note once Incan RFC 045 is implemented -->
> Note: ambient `sum` / `count` builtins will be possible once Incan's RFC 045 is implemented since those names clash with Incan/stdlib in ordinary expression positions.

## What's next?

- **Execution context**: How `DataSet` operations actually run (RFC 004)
- **Query DSL**: `query {}` blocks that produce plans (RFC 003)
- **Substrait**: Portable logical plans (RFC 002)
