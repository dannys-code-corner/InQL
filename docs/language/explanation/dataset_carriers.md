# Dataset carriers (Explanation)

This page explains how to think about and use InQL's dataset carriers. It is intentionally conceptual. Exact method and builder signatures live in the reference pages.

## Why dataset carriers?

Typed pipelines need a first-class carrier for columnar data indexed by type `T`. Without `DataSet[T]`, relational authoring surfaces would lack a stable primary relation and schema flow for `FROM`-style entry points.

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

Current collection and materialization flows are documented in [Execution context](execution_context.md).

Collected `DataFrame[T]` values currently expose structured materialization metadata rather than reparsing rendered output:

- resolved columns
- row count
- preview text for display/debugging

### `LazyFrame[T]` — deferred plan

Use `LazyFrame[T]` when you want to compose operations before execution:

```incan
from pub::inql import LazyFrame
from pub::inql.functions import col, gt, int_lit
from models import Order

def high_value_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return orders.filter(gt(col("amount"), int_lit(100)))
```

### `DataStream[T]` — streaming

Use `DataStream[T]` for streaming/unbounded data:

```incan
from pub::inql import DataStream
from pub::inql.functions import col, eq, str_lit
from models import Event

def important_events(events: DataStream[Event]) -> DataStream[Event]:
    return events.filter(eq(col("severity"), str_lit("critical")))
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

## Builder surfaces

Current relational authoring is explicit and builder-based. That is deliberate: these builders are the semantic target for later compiler sugar rather than throwaway stopgaps.

Today there are three concrete builder families:

- filters: `eq(...)`, `gt(...)`, `int_lit(...)`, `str_lit(...)`
- aggregates: `col(...)`, `sum(...)`, `count()`
- projections: `with_column(...)`, `add(...)`, `mul(...)`, `int_expr(...)`, `str_expr(...)`, `bool_expr(...)`

### Aggregate helpers

`.agg(...)` uses **imported** symbols from `pub::inql.functions` through explicit builders such as `col(...)`, `sum(...)`, and `count()`.

Concrete builder example:

```incan
from pub::inql.functions import col, count, sum
from pub::inql import LazyFrame
from models import Order

def orders_by_customer(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return orders.group_by([col("customer_id")]).agg([sum(col("amount")), count()])
```

That is the current semantic target for future sugar such as `.customer_id` or `query {}` aggregate expressions.

### Projection helpers

Computed columns now have one real entrypoint: `with_column(name, expr)`.

```incan
from pub::inql.functions import add, col, int_expr, mul
from pub::inql import LazyFrame
from models import Order

def enrich_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return (
        orders
            .with_column("amount_x2", mul(col("amount"), int_expr(2)))
            .with_column("amount_plus_one", add(col("amount"), int_expr(1)))
    )
```

The rule is simple:

- new name: append at the end
- existing name: replace in place

## Concrete examples

The most useful way to read the current surface is to separate:

- **compile-safe method-chain examples** for the API shape
- **runnable Session examples** for end-to-end execution

### Concrete method-chain example

This is real current InQL, not aspirational pseudocode:

```incan
from pub::inql.functions import add, col, count, int_expr, sum
from pub::inql import LazyFrame
from models import Order

def summarize_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    grouped = (
        orders
            .with_column("amount_plus_one", add(col("amount"), int_expr(1)))
            .group_by([col("customer_id")])
            .agg([sum(col("amount")), count()])
    )
    return grouped.limit(10)
```

### Runnable Session example

The runnable example at [session_grouped_aggregate_csv.incn](../../../examples/session_grouped_aggregate_csv.incn) uses the real fixture in `tests/fixtures/aggregate_orders.csv`:

```text
customer_id,amount
A,10
A,15
B,7
```

and executes this grouped aggregate:

```incan
grouped = orders.group_by([col("customer_id")]).agg([sum(col("amount")), count()])
```

So the expected grouped result is effectively:

- `A -> sum(amount)=25, count()=2`
- `B -> sum(amount)=7, count()=1`

Run it from the repository root:

```bash
incan run examples/session_grouped_aggregate_csv.incn
```

It will:

1. read the fixture through `Session.read_csv(...)`
2. build the grouped aggregate plan
3. display the collected result
4. write the grouped output to `tests/target/session_grouped_aggregate_out.csv`

## What's next?

- **Execution context**: how deferred carriers bind, execute, collect, and write in practice: [Execution context](execution_context.md)
- **Method reference**: exact `DataSet[T]` method signatures and semantics: [Dataset methods](../reference/dataset_methods.md)
- **Builder references**:
  - [Filter builders](../reference/builders/filters.md)
  - [Aggregate builders](../reference/builders/aggregates.md)
  - [Projection builders](../reference/builders/projections.md)
- **Query DSL**: `query {}` blocks that produce plans (RFC 003)
- **Substrait**: Portable logical plans (RFC 002)
