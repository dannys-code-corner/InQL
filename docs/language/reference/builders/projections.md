# Projection builders (Reference)

Projection builders are the current semantic target for computed columns.

## Functions

| Builder      | Signature                                                    | Meaning                     |
| ------------ | ------------------------------------------------------------ | --------------------------- |
| `col`        | `def col(name: str) -> ColumnExpr`                           | Named column reference.     |
| `int_expr`   | `def int_expr(value: int) -> ColumnExpr`                     | Integer literal expression. |
| `float_expr` | `def float_expr(value: float) -> ColumnExpr`                 | Float literal expression.   |
| `str_expr`   | `def str_expr(value: str) -> ColumnExpr`                     | String literal expression.  |
| `bool_expr`  | `def bool_expr(value: bool) -> ColumnExpr`                   | Boolean literal expression. |
| `add`        | `def add(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Binary addition.            |
| `mul`        | `def mul(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Binary multiplication.      |

## Dataset entrypoint

```incan
def with_column(self, name: str, expr: ColumnExpr) -> Self
```

- missing name: append at end
- existing name: replace in place

## Example

```incan
from pub::inql.functions import add, col, int_expr, mul

projected = (
    orders
        .with_column("amount_x2", mul(col("amount"), int_expr(2)))
        .with_column("amount_plus_one", add(col("amount"), int_expr(1)))
)
```

## Current limits

- No argument-bearing `select(...)` yet.
- No query-block projection sugar yet.
- No alias-free symbolic surface like `.amount * 2` yet.
