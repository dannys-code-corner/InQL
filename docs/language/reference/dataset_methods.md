# Dataset methods (Reference)

This page documents the current `DataSet[T]` method surface. Builder-function details live under `reference/builders/`.

The Substrait helper surface behind these methods is split by semantic role:

- `src/substrait/relations.incn` builds concrete `Rel` nodes
- `src/substrait/plans.incn` assembles `Plan` envelopes
- `src/substrait/inspect.incn` owns relation/plan inspection and output-column inference
- `src/substrait/schema_registry.incn` owns named-table schema binding

## Shared method surface

| Method        | Signature                                                    | Meaning                                                                                        |
| ------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `filter`      | `def filter(self, predicate: ColumnExpr) -> Self`            | Restrict rows by a boolean scalar expression.                                                  |
| `join`        | `def join(self, other: Self, on: bool) -> Self`              | Combine with another same-carrier relation using the package's boolean join predicate surface. |
| `select`      | `def select(self) -> Self`                                   | Preserve the current projection shape as an identity projection.                               |
| `with_column` | `def with_column(self, name: str, expr: ColumnExpr) -> Self` | Add or replace one projected column using a scalar expression.                                 |
| `group_by`    | `def group_by(self, columns: list[ColumnExpr]) -> Self`      | Define grouping keys using scalar expressions.                                                 |
| `agg`         | `def agg(self, measures: list[AggregateMeasure]) -> Self`    | Apply aggregate measures over the current relation or current grouping.                        |
| `order_by`    | `def order_by(self, columns: list[ColumnExpr]) -> Self`      | Sort rows by scalar expressions or ordering helpers such as `asc(...)` and `desc(...)`.        |
| `limit`       | `def limit(self, n: int) -> Self`                            | Cap row count.                                                                                 |
| `explode`     | `def explode(self) -> Self`                                  | Expand a nested list column into rows.                                                         |

## `with_column`

### Signature

```incan
def with_column(self, name: str, expr: ColumnExpr) -> Self
```

### Semantics

- If `name` does not already exist, the new projected column is appended at the end.
- If `name` already exists, that slot is replaced in place.
- Replacement preserves ordinal position.
- The scalar-expression surface is:
  - `col(name)`
  - `lit(value)`
  - `int_expr(...)`
  - `float_expr(...)`
  - `str_expr(...)`
  - `bool_expr(...)`
  - `add(left, right)`
  - `mul(left, right)`
  - `eq(left, right)`
  - `gt(left, right)`

### Example

```incan
from pub::inql import LazyFrame
from pub::inql.functions import add, col, lit, mul
from models import Order

def enrich(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return (
        orders
            .with_column("amount_x2", mul(col("amount"), lit(2)))
            .with_column("amount_plus_one", add(col("amount"), lit(1)))
    )
```

## Capability notes

- `join(...)` is constrained to same-carrier inputs and the boolean join predicate surface shown in the signature.
- `select(...)` preserves projection shape; explicit projection lists are represented today through `with_column(...)` and scalar-expression builders.
- `DataFrame[T]` exposes materialized metadata and preview text; row-level accessors belong to the materialized DataFrame API surface.
- Query-block and scoped DSL surfaces lower into these builder APIs rather than defining separate method semantics.

## Related builder references

- [Filter builders](builders/filters.md)
- [Aggregate builders](builders/aggregates.md)
- [Projection builders](builders/projections.md)
