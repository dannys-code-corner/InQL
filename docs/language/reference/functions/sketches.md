# Sketch Functions (Reference)

Sketch helpers model approximate state as typed logical values, not as ordinary strings or binary payloads. The first portable family is HyperLogLog.

| Function | Meaning |
| --- | --- |
| `hll_type(value_domain=SketchValueDomain.StringIdentifier, precision=14)` | Build HyperLogLog logical type metadata. |
| `sketch_col(name, sketch_type)` | Reference a column as typed sketch state. |
| `sketch_value(expr, sketch_type)` | Attach sketch logical metadata to an existing scalar expression. |
| `hll_sketch(expr, value_domain=SketchValueDomain.StringIdentifier, precision=14)` | Aggregate source values into typed HyperLogLog state. `expr` accepts primitive values or scalar expressions. |
| `hll_merge(sketch)` | Aggregate compatible HyperLogLog sketches into one sketch. |
| `hll_estimate(sketch)` | Estimate approximate cardinality from typed HyperLogLog state. |
| `hll_serialize(sketch)` | Serialize typed HyperLogLog state explicitly. |
| `hll_deserialize(payload, value_domain=SketchValueDomain.StringIdentifier, precision=14)` | Decode an explicit string payload value or scalar expression into typed HyperLogLog state. |

```incan
from pub::inql.functions import col, hll_sketch
from pub::inql.sketches import hll_estimate, hll_merge, hll_type, sketch_col

daily = events.group_by([col("event_date")]).agg([
    hll_sketch(col("user_id"), precision=14),
])

literal_seed = events.group_by([col("event_date")]).agg([
    hll_sketch("anonymous-user", precision=14),
])

monthly = daily.group_by([col("month")]).agg([
    hll_merge(sketch_col("hll_sketch_user_id", hll_type(precision=14))),
])

reported = monthly.with_column(
    "estimated_users",
    hll_estimate(sketch_col("hll_merge_hll_sketch_user_id", hll_type(precision=14))),
)
```

Sketch compatibility is structural. HyperLogLog sketches can merge only when family, value domain, precision, and serialization format match. `hll_deserialize(...)` requires those facts because they cannot be inferred from a payload alone.

The public helper surface follows the typed value-or-column conventions used by the rest of the function catalog: `hll_sketch(...)` accepts primitive values or scalar expressions, while `hll_deserialize(...)` accepts string payload values or scalar expressions.

RFC 025 helpers lower through InQL-owned Substrait extension mappings and carry sketch metadata in function options. The DataFusion adapter reports a backend planning diagnostic for typed sketch execution because it has no sketch runtime implementation. That rejection is an adapter capability boundary; the InQL plan remains typed and backend-neutral.
