# InQL RFC 023: Approximate and sketch functions

- **Status:** In Progress
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 016 (core aggregate functions)
  - InQL RFC 017 (aggregate modifiers)
  - InQL RFC 024 (function extension policy)
  - InQL RFC 025 (typed sketch logical values)
- **Issue:** [InQL #40](https://github.com/dannys-code-corner/InQL/issues/40)
- **RFC PR:** [InQL #50](https://github.com/dannys-code-corner/InQL/pull/50)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines the portable approximate aggregate boundary for InQL and records the sketch-state policy decision. InQL exposes explicit approximate aggregates for distinct counts and percentiles. It delegates sketch-state construction, merge, estimate, serialization, and deserialization helpers to InQL RFC 025 because those helpers require typed sketch logical values rather than ordinary string or binary payloads.

## Motivation

Spark exposes many approximate and sketch functions because large-scale analytics often trades exactness for bounded memory or faster execution. InQL should support the portable part of that direction, but sketch functions require more than names: they need accuracy parameters, merge semantics, serialization formats, determinism rules, and typed opaque state values.

If sketches are added as ordinary functions returning untyped bytes, InQL will not be able to reason about compatibility, aggregation state, or cross-backend behavior.

## Goals

- Define approximate aggregates as distinct from exact aggregates.
- Implement portable `approx_count_distinct` and `approx_percentile` aggregate helpers.
- Require accuracy/error parameters to be part of function signatures when a portable helper supports them.
- Preserve InQL-owned Substrait extension names and keep backend implementation names at the adapter boundary.
- Reserve sketch-state function names for InQL RFC 025, where sketch state is modeled as an explicit typed value family.

## Non-Goals

- Making approximate functions part of the exact core aggregate contract.
- Exposing sketch state as string or binary payloads instead of the typed sketch logical values defined by InQL RFC 025.
- Guaranteeing bit-for-bit compatibility with Spark or any other engine.
- Defining geospatial, cryptographic, or physical execution metadata functions.

## Guide-level explanation (how authors think about it)

Authors should see approximate functions as explicit approximate choices:

```incan
from pub::inql.functions import approx_count_distinct, approx_percentile, col

summary = (
    events
        .group_by([col("campaign_id")])
        .agg([
            approx_count_distinct(col("user_id")),
            approx_percentile(col("latency_ms"), 0.95, accuracy=10000),
        ])
)
```

The function names and arguments should make it clear that results are approximate and parameterized.

## Reference-level explanation (precise rules)

Approximate aggregate functions must be registered as approximate. Their registry entries must declare accuracy parameters, deterministic behavior for fixed inputs and parameters, mergeability, and result interpretation.

`approx_count_distinct(expr)` returns an approximate cardinality estimate over non-null expression values. It is a HyperLogLog-family aggregate. The portable helper intentionally has no relative-error parameter because the registered InQL Substrait extension mapping is unary; backend-specific precision controls must not be smuggled into the portable helper contract.

`approx_percentile(expr, percentile, accuracy=10000)` returns an approximate percentile estimate over numeric non-null expression values. `percentile` is a literal fraction in the inclusive range `[0.0, 1.0]`. `accuracy` is a positive integer approximation hint carried as a normal aggregate argument. The portable contract is an approximate percentile estimate, not a bit-for-bit promise of one backend's interpolation or sketch representation.

Sketch-construction functions are reserved for InQL RFC 025 and are not lowerable in this RFC. When they are introduced, they must return typed sketch values, not untyped binary blobs. Sketch values may have opaque runtime representation, but their logical type must identify the sketch family and value domain.

Sketch union, intersection, estimation, serialization, and deserialization functions are likewise reserved until InQL can accept only compatible sketch types and reject incompatible sketch-family or value-domain combinations before execution.

If serialized sketch formats are exposed, format versioning and cross-version compatibility must be specified.

## Design details

### Syntax

This RFC permits ordinary function-call syntax for approximate aggregate functions. Reserved sketch helpers will use the same call style if InQL RFC 025 admits them. RFC 023 does not require special query syntax.

### Semantics

Approximate functions must be opt-in by name or explicit option. InQL must not silently replace an exact aggregate with an approximate aggregate because a backend prefers it.

Sketch merge functions must define whether they are associative, commutative, idempotent, or order-sensitive.

### Interaction with other InQL surfaces

Approximate aggregates may appear anywhere aggregate measures are valid if their registry entry supports the position. Sketch scalar helpers are not exposed until sketch expressions have typed logical values.

### Compatibility / migration

This RFC is additive. Existing exact aggregates must not change semantics when approximate functions are introduced.

## Alternatives considered

- **Treat sketches as binary values.** Rejected because it loses type safety and merge compatibility.
- **Expose Spark sketch names directly as core functions.** Rejected because many sketch families are specialist extensions and require explicit state contracts.
- **Let backends choose approximate execution for exact aggregates.** Rejected because approximate results must be an author-visible choice.

## Drawbacks

- Sketch state types add complexity to the type system and serialization story.
- Cross-backend compatibility may be limited even when function names match.
- Accuracy parameters are difficult to explain without overpromising guarantees.

## Layers affected

- **InQL specification** — approximate and sketch functions must be separate from exact aggregate semantics.
- **InQL library package** — public helpers should expose approximate aggregate and sketch-state types only when contracts are explicit.
- **Incan compiler** — typechecking must validate sketch family compatibility and aggregate positions.
- **Execution / interchange** — Prism and Substrait lowering must preserve approximate parameters, sketch state types, and merge semantics or reject unsupported functions.
- **Documentation** — docs must label approximate functions clearly and explain accuracy parameters.

## Design Decisions

### Resolved

- `approx_count_distinct(expr)` is an aggregate measure, not a scalar expression, and its helper name makes approximate
  execution an explicit author choice.
- `approx_count_distinct` is registered as approximate metadata with HyperLogLog-family semantics, mergeability, and an
  approximate cardinality-result interpretation.
- `approx_count_distinct` follows InQL's registered unary Substrait extension mapping. It does not expose a
  user-tunable relative-error parameter because the portable mapping does not carry one.
- `approx_percentile(expr, percentile, accuracy=10000)` is an aggregate measure with t-digest-family approximation
  metadata. The helper validates literal percentile and accuracy arguments before building the measure.
- DataFusion's implementation is named `approx_distinct`; InQL keeps the InQL Substrait function name in emitted
  function metadata and rewrites only the DataFusion consumer declaration at the backend adapter boundary.
- DataFusion's approximate percentile implementation is named `approx_percentile_cont`; InQL uses the same adapter-only
  declaration rewrite and keeps `approx_percentile` as the portable Substrait extension name.
- `approx_count_distinct` allows aggregate-local filters and rejects an extra `distinct()` modifier because distinct
  estimation is already the helper's semantics.
- `approx_percentile` allows aggregate-local filters and rejects `distinct()` and ordered input because those modifiers
  are not part of the portable percentile aggregate contract.
- Sketch-state construction, merge, estimate, serialization, and deserialization helpers are delegated to InQL RFC 025.
  They are not exposed as lowerable RFC 023 functions because exposing those helpers as ordinary strings or binary values
  would violate the compatibility rules this RFC is meant to protect.

### Remaining

- InQL RFC 025 defines the follow-up design space for typed sketch state, portable serialization formats, and named
  merge/estimate helpers. That work must not retrofit RFC 023 by treating untyped binary payloads as sketch values.
- A future backend-capability layer may expose backend-specific approximation knobs as engine-specific functions or
  options when they cannot be represented by the portable helper signatures.

## Implementation Plan

1. Add registry approximation metadata with exact-helper defaults.
2. Add `approx_count_distinct(expr)` and `approx_percentile(expr, percentile, accuracy=10000)` under a logical approximate
   function family.
3. Add stable Substrait anchors and keep emitted function metadata on InQL extension names.
4. Add DataFusion adapter-local declaration rewrites to the first backend's implementation names.
5. Add focused helper, registry, Substrait lowering, Prism, and DataFusion-backed session tests with materialized output.
6. Add user-facing approximate-function docs, aggregate-builder docs, and release notes.
7. Record sketch-state helper names as reserved for InQL RFC 025.

## Progress Checklist

- [x] RFC 023 moved to In Progress with full portable approximate-aggregate design decisions.
- [x] Registry approximation metadata added for intentionally approximate functions.
- [x] `approx_count_distinct` helper added under the function catalog.
- [x] `approx_percentile` helper added under the function catalog.
- [x] Standard Substrait approximate aggregate extension metadata added.
- [x] DataFusion adapter-local approximate aggregate mappings added.
- [x] Focused helper, registry, Substrait lowering, Prism, and DataFusion-backed session tests added.
- [x] User-facing approximate-function docs, aggregate-builder docs, and release notes added.
- [x] Sketch-state logical types and sketch merge/estimate/serialization helpers delegated to InQL RFC 025 rather than
      exposed as untyped lowerable functions.
