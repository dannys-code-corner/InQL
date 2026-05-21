# Filter builders (Reference)

Current filter authoring uses the shared scalar-expression builder model.

## Functions

| Builder        | Signature                                                   | Meaning                                                                |
| -------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| `always_true`  | `def always_true() -> ColumnExpr`                           | Trivial boolean scalar expression; canonical rewrite can eliminate it. |
| `always_false` | `def always_false() -> ColumnExpr`                          | Boolean scalar expression that rejects every row.                      |
| `eq`           | `def eq(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Equality predicate scalar expression.                                  |
| `gt`           | `def gt(left: ColumnExpr, right: ColumnExpr) -> ColumnExpr` | Greater-than predicate scalar expression.                              |
| `lit`          | `def lit(value: int \| float \| str \| bool) -> ColumnExpr` | Canonical scalar literal helper.                                       |
| `int_lit`      | `def int_lit(value: int) -> ColumnExpr`                     | Typed integer literal helper.                                          |
| `str_lit`      | `def str_lit(value: str) -> ColumnExpr`                     | Typed string literal helper.                                           |
| `bool_lit`     | `def bool_lit(value: bool) -> ColumnExpr`                   | Typed boolean literal helper.                                          |

## Example

```incan
from pub::inql.functions import col, eq, gt, lit

filtered = (
    orders
        .filter(gt(col("amount"), lit(100)))
        .filter(eq(col("status"), lit("open")))
)
```

## Notes

- Filter predicates are scalar expressions, not a separate predicate-only builder hierarchy.
- The typed `*_lit(...)` helpers construct the same scalar-literal representation as `lit(...)`.
- Boolean composition belongs to the broader scalar-function surface.
