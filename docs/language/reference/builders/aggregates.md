# Aggregate builders (Reference)

Current aggregate authoring is explicit and scalar-expression-based.

## Functions

| Builder | Signature                                                   | Meaning                                                                |
| ------- | ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| `col`   | `def col(name: str) -> ColumnExpr`                          | Column reference builder used by aggregates, filters, and projections. |
| `lit`   | `def lit(value: int \| float \| str \| bool) -> ColumnExpr` | Canonical scalar literal helper.                                       |
| `sum`   | `def sum(expr: ColumnExpr) -> AggregateMeasure`             | Sum one scalar expression.                                             |
| `count` | `def count() -> AggregateMeasure`                           | Count rows in the current relation or group.                           |

## Example

```incan
from pub::inql.functions import add, col, count, lit, sum

grouped = orders.group_by([col("customer_id")]).agg([sum(add(col("amount"), lit(5))), count()])
```

## Notes

- Aggregate inputs use the same scalar-expression model as filters, projections, and grouping keys.
- Future `.column` sugar and scoped aggregate symbols should lower to this same surface rather than replacing its semantics.
