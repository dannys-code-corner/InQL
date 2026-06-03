# Filter builders (Reference)

Current filter authoring uses the shared scalar-expression builder model.

## Functions

| Builder        | Signature                                                                 | Meaning                                                                |
| -------------- | ------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `always_true`  | `def always_true() -> BoolLiteralExpr`                                    | Trivial boolean scalar expression; canonical rewrite can eliminate it. |
| `always_false` | `def always_false() -> BoolLiteralExpr`                                   | Boolean scalar expression that rejects every row.                      |
| `eq`           | `def eq(left: ScalarValueOrColumn, right: ScalarValueOrColumn) -> BoolColumnExpr` | Equality predicate scalar expression. |
| `gt`           | `def gt(left: ScalarValueOrColumn, right: ScalarValueOrColumn) -> BoolColumnExpr` | Greater-than predicate scalar expression. |
| `lit`          | `def lit(value: int \| float \| str \| bool) -> ColumnExpr`               | Canonical scalar literal helper.                                       |
| `int_lit`      | `def int_lit(value: int) -> IntLiteralExpr`                               | Typed integer literal helper.                                          |
| `str_lit`      | `def str_lit(value: str) -> StringLiteralExpr`                            | Typed string literal helper.                                           |
| `bool_lit`     | `def bool_lit(value: bool) -> BoolLiteralExpr`                            | Typed boolean literal helper.                                          |

## Example

```incan
from pub::inql.functions import col, eq, gt

filtered = (
    orders
        .filter(gt(col("amount"), 100))
        .filter(eq(col("status"), "open"))
)
```

## Notes

- Filter predicates are scalar expressions, not a separate predicate-only builder hierarchy.
- Primitive values are accepted where predicate helper signatures use value-or-column aliases. Use `lit(...)` or typed literal helpers when a broad `ColumnExpr` is required explicitly.
- Boolean composition belongs to the broader scalar-function surface.
