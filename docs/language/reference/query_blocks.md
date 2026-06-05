# Query blocks (Reference)

Query blocks are dependency-activated InQL expressions. Import `pub::inql` to make the vocabulary and helper surface
available in a downstream Incan package.

```incan
from pub::inql import DataFrame, count, desc, sum
from models import Order, OrderSummary

def summarize_orders(orders: DataFrame[Order]) -> DataFrame[OrderSummary]:
    return query {
        FROM orders
        GROUP BY .customer_id
        SELECT
            .customer_id as customer_id,
            sum(.amount) as total,
            count() as order_count,
        ORDER BY desc(.total)
        LIMIT 10
    }
```

The `OrderSummary` type parameter documents the intended output row model. The v0.1 implementation checks query
schema evolution and selected aliases through the carrier planning surface; full field/type compatibility validation
against annotated output models is tracked as schema-validation follow-up work.

InQL also accepts the colon spelling in expression position:

```incan
selected = query:
    FROM orders
    SELECT:
        .customer_id as customer_id
        .amount as amount
```

## Clauses

The implemented v0.1 query-block surface supports:

- `FROM <dataset>`
- `WHERE <predicate>` before or after `SELECT`
- `GROUP BY <expr>, ...`
- `SELECT <expr> as <alias>, ...`
- `SELECT DISTINCT <expr> as <alias>, ...`
- `ORDER BY <expr>, ...`
- `LIMIT <n>`
- `JOIN <dataset> ON <predicate>`
- `LEFT JOIN <dataset> ON <predicate>`
- `EXPLODE <expr> as <alias>`
- `WINDOW BY <alias> = <window expression>`

`ORDER BY` uses InQL ordering helpers such as `asc(...)` and `desc(...)`; postfix SQL spellings such as
`.amount DESC` are not part of the v0.1 query-block grammar.

## Expressions

Query-block expressions use Incan expression operators and desugar to the same InQL helper calls available in ordinary
method-chain code:

| Query expression | Helper equivalent |
| ---------------- | ----------------- |
| `.status == "paid"` | `eq(.status, "paid")` |
| `.status != "paid"` | `ne(.status, "paid")` |
| `.amount < 100` | `lt(.amount, 100)` |
| `.amount <= 100` | `lte(.amount, 100)` |
| `.amount > 100` | `gt(.amount, 100)` |
| `.amount >= 100` | `gte(.amount, 100)` |

The comparison helper names use `lte` and `gte` for inclusive bounds; `le` and `ge` are not public helper names.
Arithmetic operators lower the same way: `+` to `add`, `-` to `sub`, `*` to `mul`, `/` to `div`, and `%` to
`modulo`. Boolean and unary operators lower to their helper forms as well, such as `and_`, `or_`, `not_`, and `neg`.
Use `==` for equality; a single `=` remains assignment/binding syntax, not a query predicate.

## Resolution

- `.column` refers to the primary `FROM` relation or the current query schema after a projection boundary.
- `relation.column` refers to an explicitly joined relation.
- `SELECT` aliases become the output schema for later clauses.
- A `SELECT` alias may be reused by later expressions in the same `SELECT` list.

Query blocks desugar into the same carrier method calls available to ordinary InQL code before lowering through the
current carrier planning path. `LazyFrame` flows are Prism-backed; concrete `DataFrame` and `DataStream` flows still use
their documented carrier paths before converging at the Substrait boundary.
