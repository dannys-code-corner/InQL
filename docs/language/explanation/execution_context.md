# Execution context (Explanation)

This page explains how to think about InQL's execution model as it works today.

## The mental model

There are two distinct phases:

1. Build deferred relational work in `LazyFrame[T]`
2. Ask a `Session` to bind and run that work

`LazyFrame[T]` is not local data in hand. It is deferred relational intent. `DataFrame[T]` is local materialized data.

## Why `Session` exists

`Session` owns the parts that are not just logical plan shape:

- source registration
- logical-name to physical-source binding
- backend execution
- materialization
- writing to sinks

That keeps the carrier model clean. A `LazyFrame[T]` describes work. A `Session` makes that work run.

## `execute` vs `collect`

These two APIs are related but not interchangeable.

### `session.execute(...)`

Use `execute(...)` when you want an execution checkpoint:

- the plan binds successfully
- lowering succeeds
- the backend can run it

It returns `LazyFrame[T]` again because the point is validation and execution success, not local materialization.

### `session.collect(...)`

Use `collect(...)` when you want local data:

- it runs the same backend path
- it materializes a `DataFrame[T]`

This is the boundary where deferred relational work becomes local data in hand.

## Why active session exists

Some convenience APIs are nicer when they do not force the session parameter through every call site. `lazy.collect()` is one of those cases.

That convenience still needs a real execution context underneath, so it resolves through the active session at call time.

- `session.activate()` sets the current active session
- `lazy.collect()` uses that active session

If there is no active session, the convenience API fails clearly instead of pretending execution context can be ambient without definition.

## Writing is still Session-owned

`session.write_csv(...)` and `session.write_parquet(...)` remain explicit Session methods because writing is not just a carrier concern. It requires binding, execution, and sink ownership.

So the current ergonomic split is:

- convenience materialization: `lazy.collect()`
- explicit writes: `session.write_csv(...)`, `session.write_parquet(...)`

This is a current package ergonomics choice, not a statement that all future convenience APIs must keep the same shape.

## Typical flow

```incan
from pub::inql import Session
from models import Order

session = Session.default()

orders = session.read_csv[Order]("orders", "orders.csv")?
filtered = orders.filter(true).limit(10)

session.activate()
preview = filtered.collect()?
session.write_csv(filtered, "orders_out.csv")?
```

This pattern is intentionally simple:

- read returns deferred work
- transforms stay deferred
- collect materializes when needed
- writes remain explicit on the session

## Current limitation

`DataFrame[T]` is already the materialized carrier, but its row-level user API is still intentionally narrow. The important current semantic distinction is already in place:

- `LazyFrame[T]` = deferred
- `DataFrame[T]` = local materialized

For exact API shape, see [Execution context (Reference)](../reference/execution_context.md).
