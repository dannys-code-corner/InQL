# Projection builders (Reference)

Projection builders are the current semantic target for scalar expressions in computed columns and other row-level positions.

## Functions

| Builder      | Signature                                                                | Meaning                     |
| ------------ | ------------------------------------------------------------------------ | --------------------------- |
| `col`        | `def col(name: str) -> ColumnRefExpr`                                    | Named column reference.     |
| `lit`        | `def lit(value: int \| float \| str \| bool) -> ColumnExpr`              | Canonical scalar literal.   |
| `int_expr`   | `def int_expr(value: int) -> IntLiteralExpr`                             | Integer literal expression. |
| `float_expr` | `def float_expr(value: float) -> FloatLiteralExpr`                       | Float literal expression.   |
| `str_expr`   | `def str_expr(value: str) -> StringLiteralExpr`                          | String literal expression.  |
| `bool_expr`  | `def bool_expr(value: bool) -> BoolLiteralExpr`                          | Boolean literal expression. |
| `add`        | `def add(left: NumberValueOrColumn, right: NumberValueOrColumn) -> NumberColumnExpr` | Binary addition. |
| `mul`        | `def mul(left: NumberValueOrColumn, right: NumberValueOrColumn) -> NumberColumnExpr` | Binary multiplication. |
| `eq`         | `def eq(left: ScalarValueOrColumn, right: ScalarValueOrColumn) -> BoolColumnExpr` | Equality predicate. |
| `gt`         | `def gt(left: ScalarValueOrColumn, right: ScalarValueOrColumn) -> BoolColumnExpr` | Greater-than predicate. |

## Dataset entrypoint

```incan
def with_column(self, name: str, expr: ColumnExpr) -> Self
```

- missing name: append at end
- existing name: replace in place

## Example

```incan
from pub::inql.functions import add, col, mul

projected = (
    orders
        .with_column("amount_x2", mul(col("amount"), 2))
        .with_column("amount_plus_one", add(col("amount"), 1))
)
```

## Capability notes

- `with_column(...)` is the explicit computed-column entrypoint.
- Projection-list selection, query-block projection sugar, and alias-free symbolic surfaces lower to this scalar-expression model when exposed.
- Numeric, string, and boolean helpers accept primitive values where their public signatures use value-or-column aliases. Use `lit(...)` for broad scalar-expression positions that specifically require a `ColumnExpr`.
