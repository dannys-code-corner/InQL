# Aggregate builders (Reference)

Current aggregate authoring is explicit and scalar-expression-based.

## Functions

| Builder | Signature                                                   | Meaning                                                                |
| ------- | ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| `col`   | `def col(name: str) -> ColumnExpr`                          | Column reference builder used by aggregates, filters, and projections. |
| `lit`   | `def lit(value: int \| float \| str \| bool) -> ColumnExpr` | Canonical scalar literal helper.                                       |
| `sum`   | `def sum(expr: ColumnExpr) -> AggregateMeasure`             | Sum one scalar expression.                                             |
| `count` | `def count() -> AggregateMeasure` | Count rows. |
| `count_expr` | `def count_expr(expr: ColumnExpr) -> AggregateMeasure` | Count non-null expression values; compatibility spelling for the future `count(expr)` form. |
| `avg`   | `def avg(expr: ColumnExpr) -> AggregateMeasure`             | Average one numeric scalar expression.                                 |
| `min`   | `def min(expr: ColumnExpr) -> AggregateMeasure`             | Return the minimum non-null value for one orderable scalar expression.  |
| `max`   | `def max(expr: ColumnExpr) -> AggregateMeasure`             | Return the maximum non-null value for one orderable scalar expression.  |

## Example

```incan
from pub::inql.functions import add, avg, col, count, count_expr, lit, max, min, sum

grouped = orders.group_by([col("customer_id")]).agg([
    sum(add(col("amount"), lit(5))),
    count(),
    count_expr(col("discount_code")),
    avg(col("amount")),
    min(col("created_at")),
    max(col("created_at")),
])
```

## Notes

- Aggregate inputs use the same scalar-expression model as filters, projections, and grouping keys.
- `count()` counts rows. `count_expr(expr)` counts non-null values produced by the expression and lowers to the same
  canonical `count` Substrait extension function.
- `sum`, `avg`, `min`, and `max` skip null values. They return backend-null results when no non-null input value exists.
- Future `.column` sugar and scoped aggregate symbols should lower to this same surface rather than replacing its semantics.
