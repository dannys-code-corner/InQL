# Window Functions (Reference)

Window helpers are relation-aware. A window function application produces one output value per input row while reading a partition of related rows. It is not an ordinary scalar expression and must be placed through a projection-like dataset method.

```incan
from pub::inql import LazyFrame
from pub::inql.functions import col, current_row, desc, lag, rank, sum, unbounded_preceding, window
from models import Order

def ranked_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    spec = window().partition_by([col("customer_id")]).order_by([desc(col("amount"))])
    return (
        orders
            .with_window_column("customer_rank", rank().over(spec))
            .with_window_column("previous_amount", lag(col("amount")).over(spec))
            .with_window_column(
                "running_amount",
                sum(col("amount")).over(spec.rows_between(unbounded_preceding(), current_row())),
            )
    )
```

The window helper surface includes:

| Function | Meaning | Placement |
| --- | --- | --- |
| `window()` | Build an empty window specification with a whole-partition row frame. | Refine with `.partition_by(...)`, `.order_by(...)`, `.rows_between(...)`, or `.range_between(...)`, then pass to `.over(...)`. |
| `unbounded_preceding()`, `preceding(n)`, `current_row()`, `following(n)`, `unbounded_following()` | Build frame bounds. | Use with `.rows_between(...)` or `.range_between(...)`. |
| `row_number()` | Assign a sequential row number inside the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `rank()` | Rank rows with gaps after ties inside the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `dense_rank()` | Rank rows without gaps after ties inside the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `percent_rank()` | Return relative rank within the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `cume_dist()` | Return cumulative distribution within the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `ntile(n)` | Split ordered rows into `n` buckets. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `lag(expr, offset=1, default_value=...)` | Read a prior row in the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `lead(expr, offset=1, default_value=...)` | Read a later row in the ordered window. | Use `.over(window().order_by(...))`, then `with_window_column(...)`. |
| `first_value(expr)`, `last_value(expr)`, `nth_value(expr, n)` | Read a value from the current frame. | Use `.over(window().order_by(...))`, then `with_window_column(...)`; value calls may use `.ignore_nulls()` or `.respect_nulls()` before `.over(...)`. |
| `sum(...)`, `count(...)`, `avg(...)`, `min(...)`, `max(...)` | Reuse aggregate helpers over a window frame. | Call `.over(window_spec)` on the aggregate measure, then `with_window_column(...)`. |

`WindowSpec.partition_by(...)` replaces the partition expressions. `WindowSpec.order_by(...)` replaces the ordering expressions. `WindowSpec.rows_between(...)` and `WindowSpec.range_between(...)` replace the frame. Ranking, distribution, offset, and value helpers require explicit ordering; missing ordering is rejected during logical lowering.

`with_window_column(name, application)` preserves input columns and adds or replaces `name` using add-or-replace projection semantics. Compatible adjacent window projections lower through Substrait `ConsistentPartitionWindowRel` with registry-backed function anchors, frame bounds, invocation metadata, null-treatment options, and output aliases. The DataFusion session backend executes the portable window helpers through the Substrait adapter boundary.
