# InQL RFC 025: Typed sketch logical values

- **Status:** Implemented
- **Created:** 2026-05-28
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 023 (approximate and sketch functions)
  - InQL RFC 024 (function extension policy)
- **Issue:** [InQL #51](https://github.com/dannys-code-corner/InQL/issues/51)
- **RFC PR:** [InQL #55](https://github.com/dannys-code-corner/InQL/pull/55)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** v0.1

## Summary

This RFC defines typed sketch logical values for InQL. Sketch helpers must not be modeled as ordinary strings or binary blobs; they must produce and consume logical sketch values that record sketch family, input value domain, parameterization, merge compatibility, and serialization format identity.

## Core model

1. A sketch value has a logical type, even if its runtime representation is opaque to InQL.
2. Sketch construction is aggregate-shaped when it summarizes many input rows into one sketch state.
3. Sketch merge is valid only for compatible sketch values.
4. Sketch estimate, quantile, serialization, and deserialization helpers must preserve sketch-family semantics instead of
   treating sketch payloads as generic bytes.
5. Backend adapters may implement, emulate, or reject sketch operations, but they must not redefine ordinary binary or
   string expressions as sketch states.

## Motivation

Approximate aggregates such as `approx_count_distinct(...)` and `approx_percentile(...)` are useful when authors only need a scalar result. Sketch state is different: authors may want to materialize a sketch, merge sketches across partitions or files, estimate from a stored sketch later, or serialize sketch state for transport. Those operations require compatibility rules that cannot be represented by `bytes` or `str` alone.

If InQL accepts untyped sketch blobs, it cannot reject invalid operations such as merging a HyperLogLog sketch with a KLL sketch, merging sketches over different value domains, or deserializing a payload with an incompatible format version. That would push semantic validation into backend-specific runtime failures and weaken the Substrait boundary.

## Goals

- Define sketch logical values as first-class typed values.
- Define the metadata required to compare sketch compatibility before execution.
- Define how sketch construction, merge, estimate, serialization, and deserialization helpers interact with the function
  registry.
- Keep sketch state backend-neutral in InQL semantics while allowing backend-specific execution support.
- Provide a design home for HyperLogLog, KLL, theta, count-min, and bitmap-style sketch families without forcing all
  families into the first implementation.

## Non-Goals

- Defining exact binary serialization formats for every sketch family.
- Guaranteeing bit-for-bit sketch compatibility with Spark, DataSketches, DataFusion, or any other runtime.
- Making sketch values ordinary scalar strings or binary columns.
- Replacing the scalar approximate aggregates defined by InQL RFC 023.
- Defining geospatial, cryptographic, or physical execution metadata functions.

## Guide-level explanation (how authors think about it)

Authors should think of a sketch as a typed summary value. It can be produced by an aggregate, stored as a column when the carrier supports it, merged with compatible sketches, and estimated later. RFC 025 ships the first concrete family: HyperLogLog.

```incan
from pub::inql.functions import col, hll_sketch
from pub::inql.sketches import hll_estimate, hll_merge, hll_type, sketch_col

daily = events.group_by([col("event_date")]).agg([
    hll_sketch(col("user_id"), precision=14),
])

monthly = daily.group_by([col("month")]).agg([
    hll_merge(sketch_col("hll_sketch_user_id", hll_type(precision=14))),
])

reported = monthly.with_column(
    "estimated_users",
    hll_estimate(sketch_col("hll_merge_hll_sketch_user_id", hll_type(precision=14))),
)
```

The important part is that `users_hll` is not a `str` or `bytes` column. It is a sketch logical value with a family, value domain, precision, and serialized format contract.

## Reference-level explanation (precise rules)

A sketch logical value must carry at least:

- sketch family identity, such as HyperLogLog, KLL, theta, count-min, or bitmap;
- input value domain, such as string identifiers, integer identifiers, numeric values, or categorical values;
- family parameters that affect merge compatibility, such as precision, accuracy, nominal entries, width/depth, seed, or
  ordering policy;
- format identity and version when the value can be serialized;
- nullability and ordinary column-position metadata needed by existing InQL expression and relation surfaces.

Sketch construction helpers that summarize rows must be aggregate measures. They must declare approximate-result metadata and sketch-output metadata in the function registry. They must not appear where row-level scalar expressions are required unless the helper is explicitly a scalar transformation over an existing sketch value.

Sketch merge helpers must validate sketch family compatibility before lowering. InQL must reject merges between different families, incompatible value domains, incompatible parameter sets, or incompatible format versions unless the specific family declares a safe coercion or union rule.

Sketch estimate helpers must declare the scalar result they produce. HyperLogLog-style estimates should return approximate cardinality results. KLL-style quantile helpers must require explicit percentile or rank arguments. Count-min-style lookup helpers must require an item expression whose domain is compatible with the sketch domain.

Sketch serialization helpers must be explicit. InQL must not implicitly coerce a sketch value to `str` or `bytes`. Deserialization must require enough type metadata to identify family, domain, parameters, and format version before the value can participate in merge or estimate operations.

Substrait lowering must preserve sketch logical type identity through extension type metadata or must reject the operation before execution. A backend adapter may map the sketch operation to a native implementation only when it can preserve the InQL sketch contract.

## Design details

### Syntax

This RFC does not require new language syntax. Sketch helpers may use ordinary function-call syntax and aggregate-builder syntax. Type annotations for deserialization may require a future surface if ordinary helper arguments cannot carry enough type information ergonomically.

### Semantics

Sketch values are opaque to ordinary scalar operators. Equality, ordering, string operations, binary operations, and arithmetic must not accept sketch values unless a later RFC defines a specific semantic rule. Sketch values may be grouped, projected, stored, merged, estimated, or serialized only through helpers that declare sketch-aware metadata.

Sketch families must define whether merge is associative, commutative, idempotent, or order-sensitive. Families must define which parameters are part of merge compatibility and which parameters are execution hints.

### Interaction with other InQL surfaces

Dataframe method chains and future query-block syntax must resolve to the same sketch logical value model. A sketch helper that is rejected in one authoring surface must not become valid in another.

Prism may preserve sketch type metadata and may use it for validation, projection pruning, and rewrite safety. Prism must not rewrite sketch operations in ways that drop family, domain, parameter, or serialization metadata.

The Substrait boundary must remain between InQL semantics and backend execution. DataFusion or any other backend may be the first implementation target, but backend-native sketch names and payload formats do not define the portable InQL type.

### Implementation

The implemented first family is HyperLogLog:

- `SketchLogicalType` carries `family`, `value_domain`, `precision`, and `format`.
- `SketchExpr` pairs a scalar expression with `SketchLogicalType`.
- `hll_type(...)`, `sketch_value(...)`, and `sketch_col(...)` build typed sketch metadata at the authoring boundary.
- `sketch_value(...)` accepts the standard scalar value-or-column input surface before attaching sketch metadata.
- `hll_sketch(...)` is an aggregate measure that produces typed HyperLogLog state from scalar values or expressions.
- `hll_merge(...)` is an aggregate measure over existing typed HyperLogLog state.
- `hll_estimate(...)`, `hll_serialize(...)`, and `hll_deserialize(...)` are scalar helpers over typed sketch state or
  explicit serialized payloads.
- `hll_deserialize(...)` accepts the standard string value-or-column input surface for explicit payloads.
- The public `SketchFamily` API exposes HyperLogLog in this implementation; additional families should add their own
  family-specific type builders, serialization formats, registry policies, and tests rather than sharing HLL metadata.
- Function registry entries expose typed sketch policy metadata and Substrait extension mappings.
- Substrait lowering carries sketch family, value domain, precision, and format in function options.
- The DataFusion adapter rejects typed sketch execution with a backend planning diagnostic. This is an adapter
  capability boundary, not an InQL semantic limitation.

### Compatibility / migration

This RFC is additive. RFC 023 approximate scalar-result aggregates remain valid. Existing string or binary columns are not retroactively treated as sketch values. If a backend or existing dataset stores sketch bytes, authors must use explicit deserialization with the required sketch type metadata.

## Alternatives considered

- **Treat sketches as bytes.** Rejected because it prevents typechecking merge compatibility and moves semantic errors
  into backend runtime failures.
- **Expose only scalar approximate aggregates.** Rejected as a complete long-term answer because stored and mergeable
  sketches are a legitimate analytics need, especially for pre-aggregated data.
- **Copy one backend's sketch catalog directly.** Rejected because InQL needs backend-neutral semantics and capability
  reporting.
- **Make sketch values ordinary structs.** Rejected unless the struct carries a distinct logical type; ordinary structs
  do not by themselves encode family-specific compatibility rules.

## Drawbacks

- Sketch logical values add a new kind of type metadata to expression and relation planning.
- Cross-backend support will be uneven because sketch algorithms and serialized formats vary.
- Documentation must be careful not to overpromise statistical guarantees that depend on algorithm parameters and backend
  implementations.
- Serialization compatibility can become a long-term maintenance burden if exposed too early.

## Layers affected

- **InQL specification** — sketch values must be distinguished from ordinary scalar, binary, string, map, and struct
  values.
- **InQL library package** — public sketch helpers must register family, domain, parameter, merge, estimate, and
  serialization metadata.
- **Incan compiler** — typechecking and diagnostics may need enough type information to represent sketch-valued
  expressions, reject invalid operations, and preserve metadata through public helper signatures.
- **Execution / interchange** — Substrait lowering and backend adapters must preserve sketch logical type identity or
  reject unsupported sketch operations before execution.
- **Documentation** — function references and RFCs must present sketch helpers as typed approximate state, not as
  backend-specific blobs.

## Design decisions

### Resolved

- The first public type spelling is explicit library metadata: `SketchLogicalType`, `SketchExpr`, `hll_type(...)`,
  `sketch_value(...)`, and `sketch_col(...)`.
- Public sketch helpers use the same typed value-or-column input conventions as the post-RFC018 scalar catalog: source
  values are accepted as primitive values or scalar expressions, while serialized sketch payloads use the string
  value-or-column surface.
- HyperLogLog is the first implemented sketch family because it cleanly extends the distinct-count approximation surface.
- HyperLogLog merge compatibility is defined by family, value domain, precision, and serialization format.
- Serialized sketch format identity is explicit and portable at the InQL logical layer. RFC 025 defines
  `inql_hll_v1` as the first format identity without promising bit-for-bit compatibility with every backend runtime.
- Sketch values may be represented in authoring expressions today through `SketchExpr`. Broader table-schema logical
  typing is left to RFC 026 and later schema work rather than hiding sketch state as strings or bytes.
