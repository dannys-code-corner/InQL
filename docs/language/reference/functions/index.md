# Functions (Reference)

This section is the landing page for broad function families as InQL grows beyond the current builder-first slice.

Today the concrete shipped surfaces are documented here:

- [Filter builders](../builders/filters.md)
- [Aggregate builders](../builders/aggregates.md)
- [Projection builders](../builders/projections.md)

The canonical scalar literal helper is `lit(...)`. Typed literal helpers construct the same scalar-expression representation.

The current registry-backed helper surface is registered in the package-owned function registry. Registry types live in `src/function_registry.incn`, the shared package registry lives in `src/functions/registry.incn`, and concrete public helper entries are produced by `function_registry.add(...)` decorators in individual `src/functions/<family>/<name>.incn` modules. The registry-backed families are references, literals, casts, operators, predicates, conditionals, math, ordering, and aggregates. Each runtime entry exposes a stable function reference such as `inql.functions.col`, namespace, canonical name, typed lifecycle metadata (`since`, versioned changes, and optional deprecation), InQL RFC 024 policy category, function class, null behavior, alias policy, and Substrait mapping metadata. Checked function signatures come from the public helper declaration, not from a second hand-written registry signature.

The registry is the source for non-derivable machine facts. Public helper declarations are the source for argument names, argument types, and return types. Docstrings remain human-facing explanation, examples, and parameter intent. The `registry-metadata` check validates the checked API metadata projections produced from public facade aliases, registry decorators, and decorated callable signatures. Runtime registry entries are lazy and process-local: they support helper execution and lowering for loaded helpers, while the complete public catalog comes from checked metadata. This matters for generated docs, diagnostics, Prism lowering, and backend capability checks as the catalog grows.

InQL RFC 024 policy category is separate from function class. Function class describes the semantic shape (`scalar`, `aggregate`, `ordering`, and later table-valued or partition-transform shapes). Policy category describes where the function belongs: portable core, explicitly namespaced extension-only, opt-in compatibility alias, engine-specific, or rejected compatibility request. Name-only registry lookup remains core-scoped; extension and engine-specific entries use namespace-qualified lookup so compatibility names cannot silently shadow portable core names. Rejected requests are documented as rejection metadata, not as lowerable registry entries or fake Substrait mappings.

The registered helper surface currently includes:

| Function | Registry class | Mapping |
| --- | --- | --- |
| `col(...)` | scalar | deterministic field-reference rewrite |
| `lit(...)`, `int_expr(...)`, `float_expr(...)`, `str_expr(...)`, `bool_expr(...)`, `int_lit(...)`, `str_lit(...)`, `bool_lit(...)` | scalar | deterministic literal rewrites |
| `always_true()`, `always_false()` | scalar | deterministic boolean-literal rewrites |
| `cast(...)`, `try_cast(...)` | scalar | built-in Substrait `Cast` Rex shapes; `try_cast` uses return-null failure behavior |
| `add(...)`, `sub(...)`, `mul(...)`, `div(...)`, `modulo(...)`, `neg(...)` | scalar | registered Substrait scalar mappings; `modulo(...)` registers canonical `mod` |
| `eq(...)`, `ne(...)`, `lt(...)`, `lte(...)`, `gt(...)`, `gte(...)`, `equal_null(...)` | scalar | registered Substrait scalar mappings; `equal_null(...)` lowers as null-safe equality |
| `and_(...)`, `or_(...)`, `not_(...)` | scalar | registered Substrait boolean mappings |
| `is_null(...)`, `is_not_null(...)`, `is_nan(...)`, `is_not_nan(...)` | scalar | registered predicate mappings; `is_not_nan(...)` lowers as `not(is_nan(...))` |
| `coalesce(...)`, `nullif(...)`, `case_when(...)` | scalar | registered Substrait mappings; `case_when(...)` lowers as built-in `IfThen` |
| `in_(...)`, `between(...)` | scalar | built-in membership/range lowering (`SingularOrList` and `between`) |
| `abs(...)`, `ceil(...)`, `floor(...)`, `round(...)` | scalar | registered Substrait math scalar mappings; `round(...)` is currently the single-argument form |
| `asc(...)`, `desc(...)`, `asc_nulls_first(...)`, `asc_nulls_last(...)`, `desc_nulls_first(...)`, `desc_nulls_last(...)` | ordering | structural sort-field helpers consumed by `order_by(...)` and lowered to Substrait `SortRel.sorts` |
| `sum(...)`, `count()`, `count_expr(...)`, `avg(...)`, `min(...)`, `max(...)` | aggregate | registered Substrait extension functions; `count_expr(...)` is a compatibility spelling for future `count(expr)` helper overloading |

Future ANSI-style families should grow under this section instead of bloating `dataset_types` or `dataset_methods`.
