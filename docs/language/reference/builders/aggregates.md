# Aggregate builders (Reference)

Current aggregate authoring is explicit and builder-based.

## Functions

| Builder | Signature                                       | Meaning                                                                |
| ------- | ----------------------------------------------- | ---------------------------------------------------------------------- |
| `col`   | `def col(name: str) -> ColumnExpr`              | Column reference builder used by aggregates, filters, and projections. |
| `sum`   | `def sum(expr: ColumnExpr) -> AggregateMeasure` | Sum one selected numeric column.                                       |
| `count` | `def count() -> AggregateMeasure`               | Count rows in the current relation or group.                           |

## Example

```incan
from pub::inql.functions import col, count, sum

grouped = orders.group_by([col("customer_id")]).agg([sum(col("amount")), count()])
```

## Notes

- The current package slice requires explicit `col(...)` builders.
- Future `.column` sugar and scoped aggregate symbols should lower to this same surface rather than replacing its semantics.
