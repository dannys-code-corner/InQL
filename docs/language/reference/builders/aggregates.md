# Aggregate builders (Reference)

Current aggregate authoring is explicit and scalar-expression-based.

## Functions

| Builder | Signature                                                   | Meaning                                                                |
| ------- | ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| `col`   | `def col(name: str) -> ColumnExpr`                          | Column reference builder used by aggregates, filters, and projections. |
| `lit`   | `def lit(value: int \| float \| str \| bool) -> ColumnExpr` | Canonical scalar literal helper.                                       |
| `sum`   | `def sum(expr: ColumnExpr) -> AggregateMeasure`             | Sum one scalar expression.                                             |
| `count` | `def count() -> AggregateMeasure`; `def count(expr: ColumnExpr) -> AggregateMeasure` | Count rows with no argument, or count non-null expression values with one argument. |
| `count_expr` | `def count_expr(expr: ColumnExpr) -> AggregateMeasure` | Compatibility spelling for `count(expr)`. |
| `count_distinct` | `def count_distinct(expr: ColumnExpr) -> AggregateMeasure` | Count distinct non-null expression values. |
| `count_if` | `def count_if(predicate: ColumnExpr) -> AggregateMeasure` | Count rows where the predicate is true. |
| `avg`   | `def avg(expr: ColumnExpr) -> AggregateMeasure`             | Average one numeric scalar expression.                                 |
| `min`   | `def min(expr: ColumnExpr) -> AggregateMeasure`             | Return the minimum non-null value for one orderable scalar expression.  |
| `max`   | `def max(expr: ColumnExpr) -> AggregateMeasure`             | Return the maximum non-null value for one orderable scalar expression.  |
| `approx_count_distinct` | `def approx_count_distinct(expr: ColumnExpr) -> AggregateMeasure` | Estimate distinct non-null expression values. |
| `approx_percentile` | `def approx_percentile(expr: ColumnExpr, percentile: float, accuracy: int = 10000) -> AggregateMeasure` | Estimate one percentile over numeric non-null values. |
| `hll_sketch` | `def hll_sketch(expr: ColumnExpr, value_domain: SketchValueDomain = SketchValueDomain.StringIdentifier, precision: int = 14) -> AggregateMeasure` | Aggregate source values into typed HyperLogLog sketch state. |
| `hll_merge` | `def hll_merge(sketch: SketchExpr) -> AggregateMeasure` | Merge compatible typed HyperLogLog sketch values. |

## Modifiers

Aggregate measures support method-style modifiers:

| Modifier | Signature | Meaning |
| --- | --- | --- |
| `distinct` | `measure.distinct() -> AggregateMeasure` | Apply SQL-style `DISTINCT` to aggregate input values. |
| `filter` | `measure.filter(predicate: ColumnExpr) -> AggregateMeasure` | Apply an aggregate-local boolean predicate before aggregation. |
| `order_by` | `measure.order_by(ordering: list[ColumnExpr]) -> AggregateMeasure` | Record ordered aggregate input. Core aggregates reject ordered input until an order-sensitive aggregate lands. |

## Example

```incan
from pub::inql.functions import add, approx_count_distinct, approx_percentile, avg, col, count, count_distinct, count_if, eq, hll_sketch, lit, max, min, str_lit, sum

grouped = orders.group_by([col("customer_id")]).agg([
    sum(add(col("amount"), lit(5))),
    count(),
    count(col("discount_code")),
    count_distinct(col("product_id")),
    count_if(eq(col("status"), str_lit("paid"))),
    sum(col("amount")).filter(eq(col("status"), str_lit("paid"))),
    avg(col("amount")),
    min(col("created_at")),
    max(col("created_at")),
    approx_count_distinct(col("user_id")),
    approx_percentile(col("latency_ms"), 0.95),
    hll_sketch(col("user_id"), precision=14),
])
```

## Notes

- Aggregate inputs use the same scalar-expression model as filters, projections, and grouping keys.
- `count()` counts rows. `count(expr)` counts non-null values produced by the expression.
- `count(...)` accepts zero or one expression; passing multiple expressions is an error.
- `count_expr(expr)` is a compatibility spelling for `count(expr)`.
- `count_distinct(expr)` is compatibility sugar for `count(expr).distinct()`.
- `count_if(predicate)` is compatibility sugar for `count().filter(predicate)`. Rows where the predicate is false or
  null do not contribute to the aggregate.
- `sum`, `avg`, `min`, and `max` skip null values. They return backend-null results when no non-null input value exists.
- `approx_count_distinct` and `approx_percentile` are approximate aggregate choices. They allow aggregate-local filters
  but reject extra `DISTINCT` and ordered input in the portable contract.
- `approx_percentile` output names include percentile and accuracy parameters so two percentile estimates over the same
  expression do not collapse into the same output column name.
- `hll_sketch` and `hll_merge` are aggregate-shaped typed sketch helpers. They produce typed sketch state and preserve
  sketch family, value domain, precision, and format metadata through the registry and Substrait boundary.
- Unsupported aggregate modifiers fail at lowering or backend planning; they are not ignored.
- Future `.column` sugar and scoped aggregate symbols should lower to this same surface rather than replacing its semantics.
