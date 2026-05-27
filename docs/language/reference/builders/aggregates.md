# Aggregate builders (Reference)

Current aggregate authoring is explicit and scalar-expression-based.

## Functions

| Builder | Signature                                                   | Meaning                                                                |
| ------- | ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| `col`   | `def col(name: str) -> ColumnExpr`                          | Column reference builder used by aggregates, filters, and projections. |
| `lit`   | `def lit(value: int \| float \| str \| bool) -> ColumnExpr` | Canonical scalar literal helper.                                       |
| `sum`   | `def sum(expr: ColumnExpr) -> AggregateMeasure`             | Sum one scalar expression.                                             |
| `count` | `def count(*exprs: ColumnExpr) -> AggregateMeasure` | Count rows with no argument, or count non-null expression values with one argument. |
| `count_expr` | `def count_expr(expr: ColumnExpr) -> AggregateMeasure` | Compatibility spelling for `count(expr)`. |
| `count_distinct` | `def count_distinct(expr: ColumnExpr) -> AggregateMeasure` | Count distinct non-null expression values. |
| `count_if` | `def count_if(predicate: ColumnExpr) -> AggregateMeasure` | Count rows where the predicate is true. |
| `avg`   | `def avg(expr: ColumnExpr) -> AggregateMeasure`             | Average one numeric scalar expression.                                 |
| `min`   | `def min(expr: ColumnExpr) -> AggregateMeasure`             | Return the minimum non-null value for one orderable scalar expression.  |
| `max`   | `def max(expr: ColumnExpr) -> AggregateMeasure`             | Return the maximum non-null value for one orderable scalar expression.  |

## Modifiers

Aggregate measures support method-style modifiers:

| Modifier | Signature | Meaning |
| --- | --- | --- |
| `distinct` | `measure.distinct() -> AggregateMeasure` | Apply SQL-style `DISTINCT` to aggregate input values. |
| `filter` | `measure.filter(predicate: ColumnExpr) -> AggregateMeasure` | Apply an aggregate-local boolean predicate before aggregation. |
| `order_by` | `measure.order_by(ordering: list[ColumnExpr]) -> AggregateMeasure` | Record ordered aggregate input. Core aggregates reject ordered input until an order-sensitive aggregate lands. |

## Example

```incan
from pub::inql.functions import add, avg, col, count, count_distinct, count_if, eq, lit, max, min, str_lit, sum

grouped = orders.group_by([col("customer_id")]).agg([
    sum(add(col("amount"), lit(5))),
    count(),
    count(col("discount_code")),
    count_distinct(col("product_id")),
    count_if(eq(col("status"), str_lit("paid"))),
    sum(col("amount")).filter(eq(col("status"), str_lit("paid"))),
    avg(col("amount")),
    min(col("created_at")),
    max(col("created_at")),
])
```

## Notes

- Aggregate inputs use the same scalar-expression model as filters, projections, and grouping keys.
- `count()` counts rows. `count(expr)` counts non-null values produced by the expression.
- `count(...)` accepts zero or one expression; passing multiple expressions is an error.
- `count_expr(expr)` is a compatibility spelling for `count(expr)`.
- `count_distinct(expr)` is compatibility sugar for `count(expr).distinct()`.
- `count_if(predicate)` is compatibility sugar for `count().filter(predicate)`. Rows where the predicate is false or
  null do not contribute to the aggregate.
- `sum`, `avg`, `min`, and `max` skip null values. They return backend-null results when no non-null input value exists.
- Unsupported aggregate modifiers fail at lowering or backend planning; they are not ignored.
- Future `.column` sugar and scoped aggregate symbols should lower to this same surface rather than replacing its semantics.
