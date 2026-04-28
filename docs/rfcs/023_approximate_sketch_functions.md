# InQL RFC 023: Approximate and sketch functions

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 016 (core aggregate functions)
  - InQL RFC 017 (aggregate modifiers)
  - InQL RFC 024 (function extension policy)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines the design boundary for approximate aggregates and sketch functions in InQL, including approximate count distinct, approximate percentiles, HyperLogLog-like sketches, KLL-like sketches, theta sketches, count-min sketches, and bitmap aggregates. These functions must use explicit approximate-result and sketch-state semantics rather than pretending to be ordinary exact aggregates.

## Motivation

Spark exposes many approximate and sketch functions because large-scale analytics often trades exactness for bounded memory or faster execution. InQL should support that direction eventually, but sketch functions require more than names: they need accuracy parameters, merge semantics, serialization formats, determinism rules, and typed opaque state values.

If sketches are added as ordinary functions returning untyped bytes, InQL will not be able to reason about compatibility, aggregation state, or cross-backend behavior.

## Goals

- Define approximate aggregates as distinct from exact aggregates.
- Define sketch state as an explicit typed value family.
- Define merge, estimate, serialize, and deserialize semantics where applicable.
- Require accuracy/error parameters to be part of function signatures.
- Keep sketch functions out of the portable core until their state contracts are explicit.

## Non-Goals

- Making approximate functions part of the first core aggregate slice.
- Standardizing a specific sketch algorithm implementation in this Draft.
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
            approx_count_distinct(col("user_id"), relative_error=0.01),
            approx_percentile(col("latency_ms"), 0.95, accuracy=10000),
        ])
)
```

The function names and arguments should make it clear that results are approximate and parameterized.

## Reference-level explanation (precise rules)

Approximate aggregate functions must be registered as approximate. Their registry entries must declare accuracy parameters, deterministic behavior for fixed inputs and parameters, mergeability, and result interpretation.

`approx_count_distinct(expr, ...)` must return an approximate cardinality estimate. Its error parameter semantics must be documented before the function reaches Planned status.

`approx_percentile(expr, percentile, ...)` must return an approximate percentile estimate. Percentile domain, interpolation behavior, and accuracy parameter semantics must be documented before the function reaches Planned status.

Sketch-construction functions must return typed sketch values, not untyped binary blobs. Sketch values may have opaque runtime representation, but their logical type must identify the sketch family and value domain.

Sketch union, intersection, estimation, serialization, and deserialization functions must accept only compatible sketch types. InQL must reject incompatible sketch-family or value-domain combinations.

If serialized sketch formats are exposed, format versioning and cross-version compatibility must be specified.

## Design details

### Syntax

This RFC permits ordinary function-call syntax for approximate aggregate functions and sketch helpers. It does not require special query syntax.

### Semantics

Approximate functions must be opt-in by name or explicit option. InQL must not silently replace an exact aggregate with an approximate aggregate because a backend prefers it.

Sketch merge functions must define whether they are associative, commutative, idempotent, or order-sensitive.

### Interaction with other InQL surfaces

Approximate aggregates may appear anywhere aggregate measures are valid if their registry entry supports the position. Sketch scalar helpers may appear wherever scalar expressions of sketch type are valid.

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

## Unresolved questions

- Should InQL standardize one sketch family per use case or expose multiple named families?
- What serialization format, if any, should be portable across backends?
- How should accuracy guarantees be documented without implying backend-independent statistical promises that are not true?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
