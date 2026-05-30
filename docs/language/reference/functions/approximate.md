# Approximate Functions (Reference)

Approximate helpers are explicit opt-in functions. InQL does not silently replace exact aggregates with approximate
execution because a backend can do so.

The portable RFC 023 aggregate surface is:

| Function | Meaning |
| --- | --- |
| `approx_count_distinct(expr)` | Estimate the number of distinct non-null values produced by one expression. |
| `approx_percentile(expr, percentile, accuracy=10000)` | Estimate one percentile over numeric non-null values. |

```incan
from pub::inql.functions import approx_count_distinct, approx_percentile, col

summary = (
    events
        .group_by([col("campaign_id")])
        .agg([
            approx_count_distinct(col("user_id")),
            approx_percentile(col("latency_ms"), 0.95),
        ])
)
```

`approx_count_distinct` is registered as an approximate aggregate with HyperLogLog-family metadata. The portable author
contract is an approximate non-null distinct-count estimate. It does not expose a user-tunable relative-error parameter
because the registered InQL Substrait extension mapping for this function is unary. Backend adapters must keep this
approximation visible in capability/error handling rather than redefining exact `count_distinct` semantics.

`approx_percentile` is registered as an approximate aggregate with t-digest-family metadata. `percentile` must be between
`0.0` and `1.0` inclusive. `accuracy` must be positive and is carried as an explicit aggregate argument so backend
capability handling can accept, emulate, or reject the requested approximation instead of silently changing semantics.
Generated aggregate output names include the percentile and accuracy arguments.

Both helpers lower through registered InQL Substrait aggregate extension names. The DataFusion adapter maps
`approx_count_distinct` to DataFusion's `approx_distinct` implementation and maps `approx_percentile` to
`approx_percentile_cont` at the backend boundary.

Sketch-state construction, merge, estimate, serialization, and deserialization are implemented by
[Sketch functions](sketches.md). Those helpers use typed sketch logical values with sketch family, value domain, merge
compatibility, and serialized format identity. Exposing sketch state as strings or binary payloads would violate the RFC
023 type-safety requirement.
