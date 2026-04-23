# Dataset methods (Reference)

This page documents the current `DataSet[T]` method surface. Builder-function details live under `reference/builders/`.

The Substrait helper surface behind these methods is split by semantic role:

- `src/substrait/relations.incn` builds concrete `Rel` nodes
- `src/substrait/plans.incn` assembles `Plan` envelopes
- `src/substrait/inspect.incn` owns relation/plan inspection and output-column inference
- `src/substrait/schema_registry.incn` owns named-table schema binding

## Shared method surface

| Method        | Signature                                                    | Meaning                                                                                       |
| ------------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| `filter`      | `def filter(self, predicate: FilterPredicate) -> Self`       | Restrict rows by an explicit builder-backed predicate.                                        |
| `join`        | `def join(self, other: Self, on: bool) -> Self`              | Combine with another same-carrier relation on the current placeholder join condition surface. |
| `select`      | `def select(self) -> Self`                                   | Identity projection today; full argument-bearing `select(...)` is follow-up work.             |
| `with_column` | `def with_column(self, name: str, expr: ColumnExpr) -> Self` | Add or replace one projected column using an explicit projection builder expression.          |
| `group_by`    | `def group_by(self, columns: list[ColumnExpr]) -> Self`      | Define grouping keys using explicit column builders.                                          |
| `agg`         | `def agg(self, measures: list[AggregateMeasure]) -> Self`    | Apply aggregate measures over the current relation or current grouping.                       |
| `order_by`    | `def order_by(self) -> Self`                                 | Placeholder sort entrypoint until richer order expressions land.                              |
| `limit`       | `def limit(self, n: int) -> Self`                            | Cap row count.                                                                                |
| `explode`     | `def explode(self) -> Self`                                  | Expand a nested list column into rows.                                                        |

## `with_column`

### Signature

```incan
def with_column(self, name: str, expr: ColumnExpr) -> Self
```

### Semantics

- If `name` does not already exist, the new projected column is appended at the end.
- If `name` already exists, that slot is replaced in place.
- Replacement preserves ordinal position.
- The current first-slice expression surface is:
  - `col(name)`
  - `int_expr(...)`
  - `float_expr(...)`
  - `str_expr(...)`
  - `bool_expr(...)`
  - `add(left, right)`
  - `mul(left, right)`

### Example

```incan
from pub::inql import LazyFrame
from pub::inql.functions import add, col, int_expr, mul
from models import Order

def enrich(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return (
        orders
            .with_column("amount_x2", mul(col("amount"), int_expr(2)))
            .with_column("amount_plus_one", add(col("amount"), int_expr(1)))
    )
```

## Current limits

- `join(...)` is still structurally temporary: same-carrier input type and placeholder join condition.
- `select(...)` does not yet accept explicit projection arguments.
- `DataFrame[T]` still exposes materialized metadata and preview text, not a full public row API.
- Query-block sugar and scoped DSL symbol surfaces are later compiler work. The current builder APIs are the semantic target for that future sugar.

## Related builder references

- [Filter builders](builders/filters.md)
- [Aggregate builders](builders/aggregates.md)
- [Projection builders](builders/projections.md)
