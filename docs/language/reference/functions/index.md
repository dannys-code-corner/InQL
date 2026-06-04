# Functions (Reference)

This section is the landing page for InQL's registered function families.

Today the concrete shipped surfaces are documented here:

- [Filter builders](../builders/filters.md)
- [Aggregate builders](../builders/aggregates.md)
- [Projection builders](../builders/projections.md)
- [Generator and table-valued functions](generators.md)
- [Nested data functions](nested.md)
- [Window functions](windows.md)
- [Format functions](format.md)
- [Approximate functions](approximate.md)
- [Sketch functions](sketches.md)
- [Variant functions](variants.md)

The canonical scalar literal helper is `lit(...)`. Typed literal helpers construct the same scalar-expression representation. Public helper signatures use literal-or-column aliases such as `IntValueOrColumn`, `StrValueOrColumn`, `FloatValueOrColumn`, `BoolValueOrColumn`, and `ScalarValueOrColumn` when a parameter naturally accepts either a primitive literal or an existing scalar expression. These aliases normalize literals into the same scalar expression nodes, so typed helpers can be written as `make_date(2026, 5, 30)` or `substring(col("sku"), 1, 3)` without wrapping constants in `lit(...)`. Referenced columns are validated during Prism/Substrait lowering when the current query schema has concrete primitive kind facts.

The current registry-backed helper surface covers references, literals, casts, operators, predicates, conditionals, math, ordering, aggregates, generators, nested data, windows, format helpers, sketch helpers, and variant helpers. Each runtime entry exposes a stable function reference such as `inql.functions.col`, namespace, canonical name, typed lifecycle metadata (`since`, versioned changes, and optional deprecation), function policy category, function class, null behavior, alias policy, aggregate modifier policy, and Substrait mapping metadata. Checked public helpers provide the callable signature, source-level argument and return types, and, by default, the canonical name; metadata may override the canonical name only for source spelling constraints such as the reserved-word `mod` case.

The registry is the source for non-derivable machine facts such as policy, lifecycle, aliases, aggregate modifiers, and Substrait mappings. Public helper declarations remain the source for checked callable signatures, argument names, and typed expression return shape. Docstrings remain human-facing explanation, examples, and parameter intent. The `registry-metadata` check validates the checked API metadata projections produced from public facade aliases, registry decorators, and decorated callable signatures. Runtime registry entries are lazy and process-local: they support helper execution and lowering for loaded helpers, while the complete public catalog comes from checked metadata. This matters for generated docs, diagnostics, Prism lowering, and backend capability checks as the catalog grows.

Function policy category is separate from function class. Function class describes the semantic shape (`scalar`, `aggregate`, `ordering`, and later table-valued or partition-transform shapes). Policy category describes where the function belongs: portable core, explicitly namespaced extension-only, opt-in compatibility alias, engine-specific, or rejected compatibility request. Name-only registry lookup remains core-scoped; extension and engine-specific entries use namespace-qualified lookup so compatibility names cannot silently shadow portable core names. Rejected requests are documented as rejection metadata, not as lowerable registry entries or fake Substrait mappings.

The registered helper surface currently includes:

| Function | Registry class | Mapping |
| --- | --- | --- |
| `col(...)` | scalar | deterministic field-reference rewrite |
| `lit(...)`, `int_expr(...)`, `float_expr(...)`, `str_expr(...)`, `bool_expr(...)`, `int_lit(...)`, `str_lit(...)`, `bool_lit(...)` | scalar | deterministic literal rewrites |
| `always_true()`, `always_false()` | scalar | deterministic boolean-literal rewrites |
| `cast(...)`, `try_cast(...)` | scalar | built-in Substrait `Cast` Rex shapes; primitive targets use type tokens such as `int`, `float`, `str`, and `bool`; explicit string target spellings remain available for compatibility aliases such as `int64` and `float64`; `try_cast` uses return-null failure behavior |
| `add(...)`, `sub(...)`, `mul(...)`, `div(...)`, `modulo(...)`, `neg(...)` | scalar | registered Substrait scalar mappings; `modulo(...)` registers canonical `mod` |
| `eq(...)`, `ne(...)`, `lt(...)`, `lte(...)`, `gt(...)`, `gte(...)`, `equal_null(...)` | scalar | registered Substrait scalar mappings; `equal_null(...)` lowers as null-safe equality |
| `and_(...)`, `or_(...)`, `not_(...)` | scalar | registered Substrait boolean mappings |
| `is_null(...)`, `is_not_null(...)`, `is_nan(...)`, `is_not_nan(...)` | scalar | registered predicate mappings; `is_not_nan(...)` lowers as `not(is_nan(...))` |
| `coalesce(...)`, `nullif(...)`, `case_when(...)` | scalar | registered Substrait mappings; `case_when(...)` lowers as built-in `IfThen` |
| `in_(...)`, `between(...)` | scalar | built-in membership/range lowering (`SingularOrList` and `between`) |
| `abs(...)`, `ceil(...)`, `floor(...)`, `round(...)`, `sqrt(...)`, `power(...)`, `exp(...)`, `ln(...)`, `log(...)`, `log10(...)`, `sign(...)`, `least(...)`, `greatest(...)`, `sin(...)`, `cos(...)`, `tan(...)`, `asin(...)`, `acos(...)`, `atan(...)`, `atan2(...)`, `degrees(...)`, `radians(...)` | scalar | registered RFC 018 math scalar mappings; `round(...)` accepts an optional precision expression |
| `char_length(...)`, `octet_length(...)`, `upper(...)`, `lower(...)`, `trim(...)`, `ltrim(...)`, `rtrim(...)`, `substring(...)`, `position(...)`, `overlay(...)`, `concat(...)`, `concat_ws(...)`, `replace(...)`, `translate(...)`, `repeat(...)`, `left(...)`, `right(...)`, `lpad(...)`, `rpad(...)`, `split_part(...)` | scalar | registered RFC 018 string scalar mappings with SQL-compatible one-based positions; string and integer literal parameters use `StrValueOrColumn` and `IntValueOrColumn` aliases |
| `substr(...)`, `ucase(...)`, `lcase(...)` | scalar | RFC 018 compatibility alias spellings for canonical `substring(...)`, `upper(...)`, and `lower(...)` |
| `encode(...)`, `decode(...)`, `base64(...)`, `unbase64(...)`, `hex(...)`, `unhex(...)` | scalar | registered RFC 018 text encoding helpers; convenience helpers rewrite to `encode(...)` or `decode(...)` with an explicit `StrValueOrColumn` format literal |
| `regexp_like(...)`, `regexp_replace(...)`, `regexp_extract(...)` | scalar | registered RFC 018 regex helpers using Rust-regex-compatible semantics; pattern, replacement, flags, and group-index parameters accept literal-or-column aliases |
| `current_date()`, `current_time()`, `current_timestamp()`, `date_part(...)`, `date_trunc(...)`, `time_trunc(...)`, `date_add(...)`, `date_sub(...)`, `date_diff(...)`, `timestamp_diff(...)`, `to_date(...)`, `to_time(...)`, `to_timestamp(...)`, `from_unixtime(...)`, `unix_seconds(...)`, `unix_millis(...)`, `unix_micros(...)`, `make_date(...)`, `make_time(...)`, `make_timestamp(...)`, `last_day(...)` | scalar | registered RFC 018 date/time helpers; current date/time helpers are nondeterministic registry entries, and part selectors, date amounts, epoch seconds, and constructors use `StrValueOrColumn`/`IntValueOrColumn` aliases |
| `extract(...)`, `dateadd(...)`, `datediff(...)`, `safe_cast(...)` | scalar | RFC 018 compatibility alias spellings for canonical `date_part(...)`, `date_add(...)`, `date_diff(...)`, and `try_cast(...)` |
| `array(...)`, `cardinality(...)`, `array_contains(...)`, `arrays_overlap(...)`, `array_position(...)`, `array_range(...)`, `element_at(...)`, `array_sort(...)`, `array_distinct(...)`, `array_except(...)`, `array_intersect(...)`, `array_union(...)`, `array_join(...)`, `array_slice(...)`, `array_reverse(...)`, `array_flatten(...)`, `map_from_arrays(...)`, `map_extract(...)`, `map_contains_key(...)`, `map_keys(...)`, `map_values(...)`, `map_entries(...)`, `named_struct(...)` | scalar | registered nested scalar helpers backed by Substrait extension mappings; `array_range(...)` registers canonical `range` for positional generator lowering and `map_contains_key(...)` lowers as a documented predicate rewrite |
| `explode(...)`, `explode_outer(...)`, `posexplode(...)`, `posexplode_outer(...)`, `inline(...)`, `inline_outer(...)`, `flatten(...)`, `stack(...)` | generator | relation-extension mappings consumed by `generate(...)`; positional forms use zero-based positions |
| `window()`, `unbounded_preceding()`, `preceding(...)`, `current_row()`, `following(...)`, `unbounded_following()`, `row_number()`, `rank()`, `dense_rank()`, `percent_rank()`, `cume_dist()`, `ntile(...)`, `lag(...)`, `lead(...)`, `first_value(...)`, `last_value(...)`, `nth_value(...)` | window | `window()` and bound helpers build structural window-spec metadata; placed ranking, distribution, offset, value, and aggregate-over-window helpers lower through `ConsistentPartitionWindowRel` and execute through the DataFusion session adapter |
| `md5(...)`, `sha1(...)`, `sha224(...)`, `sha256(...)`, `sha384(...)`, `sha512(...)`, `sha2(...)`, `crc32(...)`, `xxhash64(...)`, JSON helpers, CSV helpers, URL helpers | scalar | registered RFC 022 format helpers; concrete helpers lower through Substrait extension mappings, while `sha2(...)` rewrites to a supported concrete SHA-2 helper |
| `asc(...)`, `desc(...)`, `asc_nulls_first(...)`, `asc_nulls_last(...)`, `desc_nulls_first(...)`, `desc_nulls_last(...)` | ordering | structural sort-field helpers consumed by `order_by(...)` and lowered to Substrait `SortRel.sorts` |
| `sum(...)`, `count(...)`, `count_expr(...)`, `count_distinct(...)`, `count_if(...)`, `avg(...)`, `min(...)`, `max(...)` | aggregate | registered Substrait extension functions for core aggregates; `count_expr(...)` is an alias spelling for `count(expr)`, while `count_distinct(...)` and `count_if(...)` are compatibility rewrites; core aggregates allow `DISTINCT` and aggregate-local `FILTER` where the aggregate shape is valid |
| `approx_count_distinct(...)`, `approx_percentile(...)` | aggregate | registered approximate aggregate extension functions; both are explicit approximate choices and keep DataFusion implementation-name rewrites inside the backend adapter |
| `hll_sketch(...)`, `hll_merge(...)`, `hll_estimate(...)`, `hll_serialize(...)`, `hll_deserialize(...)` | aggregate/scalar | RFC 025 typed HyperLogLog sketch helpers; construction and merge are aggregate measures, estimate/serialization/deserialization are scalar helpers, and all carry explicit sketch metadata through registry and Substrait options |
| `parse_variant_json(...)`, `try_parse_variant_json(...)`, `variant_get(...)`, `typeof(...)`, `is_null_value(...)`, `is_boolean(...)`, `is_integer(...)`, `is_float(...)`, `is_string(...)`, `is_timestamp(...)`, `is_array(...)`, `is_object(...)` | scalar | RFC 026 typed variant helpers; parse helpers use `StrValueOrColumn`, access helpers return `VariantExpr` values, `typeof(...)` returns `StringColumnExpr`, predicates return `BoolColumnExpr`, and all carry explicit variant metadata through registry and Substrait options |

New function families should grow under this section instead of bloating dataset carrier or dataset method modules.
