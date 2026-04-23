# Filter builders (Reference)

Current filter authoring is explicit and builder-based.

## Functions

| Builder        | Signature                                                               | Meaning                                                |
| -------------- | ----------------------------------------------------------------------- | ------------------------------------------------------ |
| `always_true`  | `def always_true() -> FilterPredicate`                                  | Trivial predicate; canonical rewrite can eliminate it. |
| `always_false` | `def always_false() -> FilterPredicate`                                 | Predicate that rejects every row.                      |
| `eq`           | `def eq(column: ColumnExpr, literal: FilterLiteral) -> FilterPredicate` | Equality predicate.                                    |
| `gt`           | `def gt(column: ColumnExpr, literal: FilterLiteral) -> FilterPredicate` | Greater-than predicate.                                |
| `int_lit`      | `def int_lit(value: int) -> FilterLiteral`                              | Integer literal for filter predicates.                 |
| `str_lit`      | `def str_lit(value: str) -> FilterLiteral`                              | String literal for filter predicates.                  |
| `bool_lit`     | `def bool_lit(value: bool) -> FilterLiteral`                            | Boolean literal for filter predicates.                 |

## Example

```incan
from pub::inql.functions import col, eq, gt, int_lit, str_lit

filtered = (
    orders
        .filter(gt(col("amount"), int_lit(100)))
        .filter(eq(col("status"), str_lit("open")))
)
```

## Notes

- Filter predicates currently operate on one explicit column builder plus one explicit literal.
- Rich boolean composition is follow-up work.
