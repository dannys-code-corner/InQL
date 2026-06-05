# InQL RFC 026: Semi-structured variant logical values

- **Status:** Implemented
- **Created:** 2026-05-28
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 020 (nested data functions)
  - InQL RFC 022 (semi-structured and format functions)
  - InQL RFC 024 (function extension policy)
- **Issue:** [InQL #52](https://github.com/dannys-code-corner/InQL/issues/52)
- **RFC PR:** [InQL #56](https://github.com/dannys-code-corner/InQL/pull/56)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** v0.1

## Summary

This RFC defines semi-structured variant logical values for InQL. A variant value is distinct from ordinary `str` and `bytes` payloads: it carries a logical kind such as null, boolean, integer, floating point, string, timestamp, array, or object, and InQL predicates inspect that logical value rather than reparsing arbitrary JSON text.

## Core model

1. A semi-structured variant is a first-class logical value, even when a backend stores it in an opaque native representation.
2. Variant null and relation-level SQL null are distinct. Variant predicates must not erase that distinction.
3. Type predicates such as `typeof`, `is_array`, `is_object`, `is_integer`, `is_timestamp`, and `is_null_value` operate on variant expressions, not raw strings.
4. RFC 022 string-backed JSON and CSV payload helpers remain stable; this RFC may add variant-returning parse helpers, but it must not silently change existing helper return types.
5. Backend adapters may implement, emulate, or reject variant operations, but backend-native variant semantics do not define the portable InQL contract.

## Motivation

JSON and semi-structured payloads are common in data pipelines, but predicates such as `is_array(...)` and `typeof(...)` are ambiguous if InQL only has string payloads. `is_array(col("payload"))` could mean "parse this string as JSON and inspect the root value", or it could mean "inspect a typed semi-structured value that was already parsed by the logical plan." Those are different contracts with different null behavior, error behavior, and backend portability.

If InQL accepts variant predicate names before defining variant values, it either squats on better names with string-parser semantics or leaves backend adapters to decide meaning at execution time. This RFC records the missing logical value model so those predicates have a precise home.

## Goals

- Define a semi-structured variant logical value family.
- Define the predicate and inspection functions that operate on variant values.
- Distinguish relation-level SQL null from semi-structured null values.
- Define how variant-returning parse helpers interact with RFC 022 string-backed payload helpers.
- Keep variant semantics backend-neutral across Prism, Substrait, and backend adapters.

## Non-Goals

- Changing RFC 022 string-backed helpers such as `parse_json`, `from_json`, `to_json`, `from_csv`, or `to_csv`.
- Defining XML, geospatial, sketch, or binary-format functions.
- Inferring timestamps, decimals, or other rich scalar kinds from untyped JSON strings without an explicit schema or parse option.
- Requiring every backend to support native variant storage.
- Defining query-block syntax for variant destructuring.

## Guide-level explanation (how authors think about it)

Authors should parse payload text into a variant value before using variant predicates:

```incan
from pub::inql.functions import col, is_array, is_null_value, parse_variant_json, typeof, variant_get
from pub::inql.variants import variant_col

events_with_payload = events.with_column("payload_value", parse_variant_json(col("payload")))

projected = (
    events_with_payload
        .with_column("payload_kind", typeof(variant_col("payload_value")))
        .with_column("is_items_array", is_array(variant_get(variant_col("payload_value"), "$.items")))
        .with_column("deleted_was_json_null", is_null_value(variant_get(variant_col("payload_value"), "$.deleted_at")))
)
```

Authors who only need text validation or normalized JSON strings should keep using the RFC 022 helpers. Variant helpers are for plans that need logical semi-structured values and type-aware predicates.

## Reference-level explanation (precise rules)

InQL defines `VariantLogicalType` and `VariantExpr`. A variant logical type records a `VariantKind` and `VariantEncoding`. The implemented portable kind set is `any`, `null`, `boolean`, `integer`, `float`, `string`, `timestamp`, `array`, and `object`. The first implemented encoding is JSON.

Variant predicates must accept variant expressions. They must not accept ordinary `str` expressions as an implicit parse-and-inspect shortcut. Authors must use an explicit variant parse or cast helper when starting from JSON text.

`typeof(expr)` must return a stable lowercase kind name for a non-null variant value. It must distinguish at least `null`, `boolean`, `integer`, `float`, `string`, `timestamp`, `array`, and `object`. It must not report `timestamp` for a plain JSON string unless an explicit schema or parse option produced a typed timestamp variant.

`is_null_value(expr)`, `is_boolean(expr)`, `is_integer(expr)`, `is_float(expr)`, `is_string(expr)`, `is_timestamp(expr)`, `is_array(expr)`, and `is_object(expr)` must inspect the variant kind. `is_integer(...)` must be true only for integer variant values, not floating point values whose runtime value happens to have no fractional component. `is_null_value(...)` must be true only for semi-structured null values.

SQL null must remain distinct from variant null. If a predicate input is SQL null rather than a present variant value, the predicate result must follow InQL's scalar null behavior for missing inputs rather than returning true for `is_null_value(...)`.

`parse_variant_json(payload)` is the strict JSON-to-variant helper. `try_parse_variant_json(payload)` is the recoverable form. Their payload parameter accepts `StrValueOrColumn`, so authors may pass a string literal, a string column reference, or a string-producing expression without wrapping primitive values in `lit(...)`. Strict parse helpers must fail malformed payloads according to registry error metadata. Recoverable parse helpers must return SQL null or another explicitly documented recoverable result for malformed payloads. A JSON `null` payload must produce a present variant null, not SQL null.

`variant_get(expr, path)` accesses a variant path. Literal path strings are validated as beginning with `$`. String columns or string-producing expressions are accepted as dynamic paths and are validated at execution time by the implementation that evaluates the expression. The current path spelling matches the RFC 022 JSON path helper spelling so authors do not learn two root-marker conventions, but it remains a variant operation rather than a JSON-text extraction shortcut. Variant field/path access must preserve whether a missing path produced SQL null, variant null, or a present value. If a backend cannot preserve that distinction, the adapter must reject the operation or require an explicit compatibility mode.

Substrait lowering preserves variant logical type identity in registry metadata and scalar function options. A backend adapter may map variant values and predicates to native functions only when it preserves the InQL variant contract. The DataFusion adapter currently reports a backend planning diagnostic for typed variant execution because it has no variant runtime implementation.

## Design details

### Syntax

This RFC does not require new language syntax. Variant values and predicates may use ordinary helper calls and dataframe method chains. Future query-block syntax must lower to the same variant expression model.

### Semantics

Variant arrays and objects are semi-structured values, not InQL relation shapes. They may be projected, inspected, and passed to variant-aware helpers. They must not change relation cardinality unless used with a generator or table-valued function that explicitly defines such a change.

Variant ordering, equality, grouping, and serialization are not implicit. If InQL supports those operations for variants, the operation must define kind ordering, null behavior, and backend compatibility rather than inheriting an arbitrary backend default.

### Interaction with other InQL surfaces

RFC 020 nested data functions operate on typed array, map, and struct expressions. Variant arrays and objects may interoperate with those functions only through explicit conversion rules.

RFC 022 format helpers that return normalized JSON or CSV text remain string-backed helpers. This RFC may add variant-returning helpers with different names or explicit options, but existing RFC 022 helpers must not silently change return type.

Prism may use variant kind metadata for validation, projection pruning, and rewrite safety. Prism must not rewrite variant operations in ways that collapse SQL null and variant null, drop schema-directed typed scalar information, or turn variant predicates into text parser calls.

The Substrait boundary remains between InQL semantics and backend execution. DataFusion or any other backend may be an implementation target, but backend-native variant names, path syntaxes, and null rules do not define the portable InQL semantics.

### Compatibility / migration

This RFC is additive. Existing string payload columns remain strings. Existing RFC 022 functions keep their current string-backed behavior. Authors who want variant semantics must opt into variant-returning helpers or explicit casts.

## Implementation

The implemented public model is:

- `VariantKind`, `VariantEncoding`, `VariantParseMode`, `VariantLogicalType`, and `VariantExpr`.
- Metadata helpers: `variant_type(...)`, `variant_col(...)`, `variant_value(...)`, and `variant_types_compatible(...)`.
- Parse/access helpers: `parse_variant_json(...)`, `try_parse_variant_json(...)`, and `variant_get(...)`.
- Inspection helpers: `typeof(...)` returns `StringColumnExpr`; predicates such as `is_null_value(...)`,
  `is_boolean(...)`, `is_integer(...)`, `is_float(...)`, `is_string(...)`, `is_timestamp(...)`, `is_array(...)`, and
  `is_object(...)` return `BoolColumnExpr`.

Each public helper is registry-backed with explicit variant policy metadata. Variant helpers lower through InQL-owned Substrait extension mappings and carry variant kind, encoding, and parse mode as scalar function options where needed.

## Alternatives considered

- **Make `typeof` and `is_array` parse strings directly.** Rejected because it couples predicate semantics to JSON text parsing and makes typed variant values incompatible with the obvious function names.
- **Change RFC 022 JSON helpers to return variants.** Rejected because it would silently change the meaning of already-defined string-backed payload helpers and make simple normalization workflows depend on a richer value model.
- **Expose backend-native variant functions as compatibility aliases.** Rejected as the portable core because backend-native null behavior, path syntax, and type names differ.
- **Represent variants as ordinary structs or maps.** Rejected unless the value carries distinct variant logical type identity; ordinary nested values do not by themselves encode semi-structured null and kind semantics.

## Drawbacks

- Variant logical values add a new type family to expression planning and backend capability reporting.
- Cross-backend support will be uneven because engines differ in native semi-structured support.
- Keeping SQL null and variant null distinct requires careful documentation, tests, and adapter validation.
- Variant path access can become a large surface if not kept separate from parser and generator responsibilities.

## Layers affected

- **InQL specification** — variant values must be distinct from strings, bytes, typed nested values, and sketch values.
- **InQL library package** — public helpers must expose variant parse, path access, and predicate functions only with explicit registry metadata.
- **Incan compiler** — typechecking may need enough helper metadata to reject string expressions where variant expressions are required.
- **Execution / interchange** — Prism, Substrait lowering, and backend adapters must preserve variant type identity and SQL-null versus variant-null behavior or reject unsupported operations.
- **Documentation** — function references must present variant predicates as variant operations, not as JSON-text parser shortcuts.

## Design Decisions

- The public type spellings are `VariantLogicalType` and `VariantExpr`.
- Variant-returning JSON parsing uses new helper names, `parse_variant_json(...)` and `try_parse_variant_json(...)`, so
  RFC 022 string-backed JSON helpers remain stable.
- The shipped kind set is the JSON-compatible family plus timestamp: null, boolean, integer, float, string, timestamp,
  array, and object. Decimal, binary, date, and interval are not part of this RFC's public variant-kind contract.
- Variant path access uses `$`-rooted literal paths or string-producing dynamic path expressions through
  `variant_get(...)`; missing-path runtime behavior is an execution contract for adapters and must not collapse SQL null
  and variant null.
