# Approximate Functions (Reference)

Approximate helpers are explicit opt-in functions. InQL does not silently replace exact aggregates with approximate
execution because a backend can do so.

The current implemented slice is one aggregate:

| Function | Meaning |
| --- | --- |
| `approx_count_distinct(expr)` | Estimate the number of distinct non-null values produced by one expression. |

```incan
from pub::inql.functions import approx_count_distinct, col

summary = (
    events
        .group_by([col("campaign_id")])
        .agg([approx_count_distinct(col("user_id"))])
)
```

`approx_count_distinct` is registered as an approximate aggregate with HyperLogLog-family metadata. The portable author
contract is an approximate non-null distinct-count estimate; the first slice does not expose a user-tunable relative
error parameter because the standard Substrait mapping for this function is unary. Backend adapters must keep this
approximation visible in capability/error handling rather than redefining exact `count_distinct` semantics.

The helper lowers through the standard Substrait `approx_count_distinct` aggregate extension name. The DataFusion
adapter maps that declaration to DataFusion's `approx_distinct` implementation name at the backend boundary.

Approximate percentile functions, sketch-state values, sketch serialization, and sketch merge/estimate helpers remain
future slices until their accuracy parameters, logical sketch types, and compatibility rules are explicit.
