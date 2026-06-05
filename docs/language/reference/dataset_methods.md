# Dataset methods (Reference)

This page documents the current carrier method surface. Builder-function details live under `reference/builders/`.

The Substrait helper surface behind these methods is split by semantic role:

- `src/substrait/relations.incn` builds concrete `Rel` nodes
- `src/substrait/plans.incn` assembles `Plan` envelopes
- `src/substrait/inspect.incn` owns relation/plan inspection and output-column inference
- `src/schema_registry.incn` owns logical named-table schema binding

## Carrier method surface

| Method        | Signature                                                    | Meaning                                                                                        |
| ------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| `filter`      | `def filter(self, predicate: ColumnExpr) -> Self`            | Restrict rows by a boolean scalar expression.                                                  |
| `join`        | `def join(self, other: Self, on: ColumnExpr) -> Self`        | Combine with another same-carrier relation using the package's scalar predicate surface.       |
| `select`      | `def select[U](self, assignments: list[ProjectionAssignment] = []) -> SameCarrier[U]` | Project an output row shape while preserving the carrier kind. |
| `with_column` | `def with_column(self, name: str, expr: ColumnExpr) -> Self` | Add or replace one projected column using a scalar expression.                                 |
| `group_by`    | `def group_by(self, columns: list[ColumnExpr]) -> Self`      | Define grouping keys using scalar expressions.                                                 |
| `agg`         | `def agg(self, measures: list[AggregateMeasure]) -> Self`    | Apply aggregate measures over the current relation or current grouping.                        |
| `generate`    | `def generate(self, generator: GeneratorApplication) -> Self` | Apply a relation-shaping generator such as `explode(...)` with explicit output aliases.        |
| `with_window_column` | `def with_window_column(self, name: str, application: WindowFunctionApplication) -> Self` | Add or replace one projected column using a named window function. |
| `order_by`    | `def order_by(self, columns: list[ColumnExpr]) -> Self`      | Sort rows by scalar expressions or ordering helpers such as `asc(...)` and `desc(...)`.        |
| `limit`       | `def limit(self, n: int) -> Self`                            | Cap row count.                                                                                 |

`SameCarrier[U]` means `DataFrame[U]` for `DataFrame[T]`, `LazyFrame[U]` for `LazyFrame[T]`, and `DataStream[U]` for `DataStream[T]`. The root `DataSet[T]` trait remains the common plan/schema contract; schema-changing projection is expressed on concrete carriers until Incan grows native trait type-family support.

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
from pub::inql.functions import add, col, mul
from models import Order

def enrich(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return (
        orders
            .with_column("amount_x2", mul(col("amount"), 2))
            .with_column("amount_plus_one", add(col("amount"), 1))
    )
```

## Capability notes

- `join(...)` is constrained to same-carrier inputs and the `ColumnExpr` predicate surface shown in the signature.
- `select(...)` is the schema-changing projection boundary used by query blocks. Identity `select()` preserves the current row model through its surrounding expected type, while explicit assignments can retarget to a new row model.
- `generate(...)` preserves all input columns and appends generated output aliases for `explode`, `explode_outer`, `posexplode`, `posexplode_outer`, `inline`, `inline_outer`, `flatten`, and `stack` generator applications. Alias collisions are rejected during planning/lowering.
- `with_window_column(...)` supports placed ranking, distribution, offset, value, and aggregate-over-window helpers over explicit window specs. Portable helpers lower through Substrait window relations and execute through the DataFusion session adapter.
- `DataFrame[T]` exposes materialized metadata and preview text; row-level accessors belong to the materialized DataFrame API surface.
- Query-block and scoped DSL surfaces lower into these builder APIs rather than defining separate method semantics.

## Related builder references

- [Filter builders](builders/filters.md)
- [Aggregate builders](builders/aggregates.md)
- [Projection builders](builders/projections.md)
- [Window functions](functions/windows.md)
