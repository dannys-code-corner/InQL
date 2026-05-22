# Projection builders (Reference)

Projection builders are the current semantic target for scalar expressions in computed columns and other row-level positions.

## Functions

| Builder      | Signature                                                    | Meaning                     |
| ------------ | ------------------------------------------------------------ | --------------------------- |
| `col`        | `def col(name: str) -> ColumnExpr`                           | Named column reference.     |
| `lit`        | `def lit(value: int \| float \| str \| bool) -> ColumnExpr`  | Canonical scalar literal.   |
| `int_expr`   | `def int_expr(value: int) -> ColumnExpr`                     | Integer literal expression. |
| `float_expr` | `def float_expr(value: float) -> ColumnExpr`                 | Float literal expression.   |
| `str_expr`   | `def str_expr(value: str) -> ColumnExpr`                     | String literal expression.  |
| `bool_expr`  | `def bool_expr(value: bool) -> ColumnExpr`                   | Boolean literal expression. |
| `add`        | `def add(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Binary addition.            |
| `mul`        | `def mul(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Binary multiplication.      |
| `eq`         | `def eq(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr`  | Equality predicate.         |
| `gt`         | `def gt(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr`  | Greater-than predicate.     |

## Dataset entrypoint

```incan
def with_column(self, name: str, expr: ColumnExpr) -> Self
```

- missing name: append at end
- existing name: replace in place

## Example

```incan
from pub::inql.functions import add, col, lit, mul

projected = (
    orders
        .with_column("amount_x2", mul(col("amount"), lit(2)))
        .with_column("amount_plus_one", add(col("amount"), lit(1)))
)
```

## Capability notes

- `with_column(...)` is the explicit computed-column entrypoint.
- Projection-list selection, query-block projection sugar, and alias-free symbolic surfaces lower to this scalar-expression model when exposed.
- The typed literal helpers construct the same scalar-literal representation as `lit(...)`.
