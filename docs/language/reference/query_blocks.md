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

## Resolution

- `.column` refers to the primary `FROM` relation or the current query schema after a projection boundary.
- `relation.column` refers to an explicitly joined relation.
- `SELECT` aliases become the output schema for later clauses.
- A `SELECT` alias may be reused by later expressions in the same `SELECT` list.

Query blocks lower into the same Dataset, Prism, Substrait, and Session adapter path as equivalent method-chain code.
