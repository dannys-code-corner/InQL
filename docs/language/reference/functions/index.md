# Functions (Reference)

This section is the landing page for broad function families as InQL grows beyond the current builder-first slice.

Today the concrete shipped surfaces are documented here:

- [Filter builders](../builders/filters.md)
- [Aggregate builders](../builders/aggregates.md)
- [Projection builders](../builders/projections.md)

The canonical scalar literal helper is `lit(...)`. Typed literal helpers construct the same scalar-expression representation.

The current public helper surface is also registered in the package-owned function registry. Registry types live in `src/function_registry.incn`, while the concrete public helper entries are produced by `FUNCTION_REGISTRY.add(...)` decorators in `src/functions.incn`. Each entry exposes a stable function reference such as `inql.functions.col`, canonical name, typed lifecycle metadata (`since`, versioned changes, and optional deprecation), signature facts, function class, null behavior, alias policy, and Substrait mapping metadata.

The registry is the source for machine-readable function facts. Docstrings remain human-facing explanation, while argument names, type rules, lifecycle facts, and Substrait mappings come from typed registry metadata and public helper signatures. The `registry-metadata` check validates that runtime registry entries produced by decorators still agree with checked API metadata for decorator canonical names, argument names, argument types, and return types. This matters for generated docs, diagnostics, Prism lowering, and backend capability checks as the catalog grows.

The first registered helpers are:

| Function | Registry class | Mapping |
| --- | --- | --- |
| `col(...)` | scalar | deterministic field-reference rewrite |
| `lit(...)`, `int_expr(...)`, `float_expr(...)`, `str_expr(...)`, `bool_expr(...)`, `int_lit(...)`, `str_lit(...)`, `bool_lit(...)` | scalar | deterministic literal rewrites |
| `add(...)`, `mul(...)`, `eq(...)`, `gt(...)` | scalar | registered Substrait extension functions |
| `always_true()`, `always_false()` | scalar | deterministic boolean-literal rewrites |
| `sum(...)`, `count()` | aggregate | registered Substrait extension functions |

Future ANSI-style families should grow under this section instead of bloating `dataset_types` or `dataset_methods`.
