# InQL RFC 025: Typed sketch logical values

- **Status:** Draft
- **Created:** 2026-05-28
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 023 (approximate and sketch functions)
  - InQL RFC 024 (function extension policy)
- **Issue:** —
- **RFC PR:** [InQL #50](https://github.com/dannys-code-corner/InQL/pull/50)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines typed sketch logical values for InQL. Sketch helpers must not be modeled as ordinary strings or binary blobs; they must produce and consume logical sketch values that record sketch family, input value domain, parameterization, merge compatibility, and serialization format identity.

## Core model

1. A sketch value has a logical type, even if its runtime representation is opaque to InQL.
2. Sketch construction is aggregate-shaped when it summarizes many input rows into one sketch state.
3. Sketch merge is valid only for compatible sketch values.
4. Sketch estimate, quantile, serialization, and deserialization helpers must preserve sketch-family semantics instead of treating sketch payloads as generic bytes.
5. Backend adapters may implement, emulate, or reject sketch operations, but they must not redefine ordinary binary or string expressions as sketch states.

## Motivation

Approximate aggregates such as `approx_count_distinct(...)` and `approx_percentile(...)` are useful when authors only need a scalar result. Sketch state is different: authors may want to materialize a sketch, merge sketches across partitions or files, estimate from a stored sketch later, or serialize sketch state for transport. Those operations require compatibility rules that cannot be represented by `bytes` or `str` alone.

If InQL accepts untyped sketch blobs, it cannot reject invalid operations such as merging a HyperLogLog sketch with a KLL sketch, merging sketches over different value domains, or deserializing a payload with an incompatible format version. That would push semantic validation into backend-specific runtime failures and weaken the Substrait boundary.

## Goals

- Define sketch logical values as first-class typed values.
- Define the metadata required to compare sketch compatibility before execution.
- Define how sketch construction, merge, estimate, serialization, and deserialization helpers interact with the function registry.
- Keep sketch state backend-neutral in InQL semantics while allowing backend-specific execution support.
- Provide a design home for HyperLogLog, KLL, theta, count-min, and bitmap-style sketch families without forcing all families into the first implementation.

## Non-Goals

- Defining exact binary serialization formats for every sketch family.
- Guaranteeing bit-for-bit sketch compatibility with Spark, DataSketches, DataFusion, or any other runtime.
- Making sketch values ordinary scalar strings or binary columns.
- Replacing the scalar approximate aggregates defined by InQL RFC 023.
- Defining geospatial, cryptographic, or physical execution metadata functions.

## Guide-level explanation (how authors think about it)

Authors should think of a sketch as a typed summary value. It can be produced by an aggregate, stored as a column when the carrier supports it, merged with compatible sketches, and estimated later. The helper names below are illustrative; the exact public names remain an unresolved question for this Draft.

```incan
from pub::inql.functions import col
from pub::inql.sketches import hll_estimate, hll_merge, hll_sketch

daily = events.group_by([col("event_date")]).agg([
    hll_sketch(col("user_id"), precision=14),
])

monthly = daily.group_by([col("month")]).agg([
    hll_merge(col("hll_sketch_user_id")),
])

reported = monthly.with_column("estimated_users", hll_estimate(col("hll_merge_hll_sketch_user_id")))
```

The important part is not the exact helper names in this Draft. The important part is that `users_hll` is not a `str` or `bytes` column. It is a sketch logical value with a family, value domain, and parameter contract.

## Reference-level explanation (precise rules)

A sketch logical value must carry at least:

- sketch family identity, such as HyperLogLog, KLL, theta, count-min, or bitmap;
- input value domain, such as string identifiers, integer identifiers, numeric values, or categorical values;
- family parameters that affect merge compatibility, such as precision, accuracy, nominal entries, width/depth, seed, or ordering policy;
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

### Compatibility / migration

This RFC is additive. RFC 023 approximate scalar-result aggregates remain valid. Existing string or binary columns must not be retroactively treated as sketch values. If a backend or existing dataset stores sketch bytes, authors must use explicit deserialization with the required sketch type metadata.

## Alternatives considered

- **Treat sketches as bytes.** Rejected because it prevents typechecking merge compatibility and moves semantic errors into backend runtime failures.
- **Expose only scalar approximate aggregates.** Rejected as a complete long-term answer because stored and mergeable sketches are a legitimate analytics need, especially for pre-aggregated data.
- **Copy one backend's sketch catalog directly.** Rejected because InQL needs backend-neutral semantics and capability reporting.
- **Make sketch values ordinary structs.** Rejected unless the struct carries a distinct logical type; ordinary structs do not by themselves encode family-specific compatibility rules.

## Drawbacks

- Sketch logical values add a new kind of type metadata to expression and relation planning.
- Cross-backend support will be uneven because sketch algorithms and serialized formats vary.
- Documentation must be careful not to overpromise statistical guarantees that depend on algorithm parameters and backend implementations.
- Serialization compatibility can become a long-term maintenance burden if exposed too early.

## Layers affected

- **InQL specification** — sketch values must be distinguished from ordinary scalar, binary, string, map, and struct values.
- **InQL library package** — public sketch helpers must register family, domain, parameter, merge, estimate, and serialization metadata.
- **Incan compiler** — typechecking and diagnostics may need enough type information to represent sketch-valued expressions, reject invalid operations, and preserve metadata through public helper signatures.
- **Execution / interchange** — Substrait lowering and backend adapters must preserve sketch logical type identity or reject unsupported sketch operations before execution.
- **Documentation** — function references and RFCs must present sketch helpers as typed approximate state, not as backend-specific blobs.

## Unresolved questions

- What is the minimal public type spelling for sketch values, especially for deserialization where the type cannot be inferred from a payload alone?
- Which sketch family should be the first implementation target: HyperLogLog, KLL, theta, count-min, or bitmap?
- Which family parameters are part of merge compatibility, and which are backend execution hints?
- Should serialized sketch format identity be portable across backends, or should serialization be extension-specific by default?
- Should sketch values be legal in table schemas before InQL has a broader logical type model beyond primitive row columns?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
