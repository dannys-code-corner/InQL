# InQL RFC 022: Semi-structured and format functions

- **Status:** In Progress
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 009 (session format handler registry)
  - InQL RFC 010 (CSV dialect and interpretation contract)
  - InQL RFC 011 (source discovery and parse-unit expansion)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 020 (nested data functions)
- **Issue:** [InQL #39](https://github.com/dannys-code-corner/InQL/issues/39)
- **RFC PR:** [InQL #49](https://github.com/dannys-code-corner/InQL/pull/49)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines InQL's semi-structured and format-oriented function families: JSON value functions, CSV value functions, schema inference helpers, type predicates for dynamic values, URL helpers, and hashing functions. These functions are practical data-engineering tools, but they should live in explicit format families rather than the core scalar catalog.

## Motivation

Real data pipelines frequently parse JSON strings, emit JSON values, inspect CSV-shaped payloads, hash identifiers, and normalize URLs. Spark and Snowflake expose many of these as functions, while InQL already has separate source-format RFCs. The design should preserve that boundary: reading a CSV file is an I/O concern, but parsing a CSV-encoded scalar value is a scalar format function.

Without a separate RFC, format helpers risk leaking ingestion policy into the scalar catalog or duplicating schema inference semantics from the session/source-discovery layer.

## Goals

- Define JSON scalar and schema helper functions.
- Define CSV scalar and schema helper functions.
- Define dynamic-value type predicates where InQL supports variant-like values.
- Define URL parse/encode/decode helpers.
- Define deterministic hash functions for data engineering.
- Keep format functions separate from source reading and writing contracts.

## Non-Goals

- Defining source discovery, file scanning, or format handler registration.
- Defining XML, variant, geospatial, crypto, or sketch functions.
- Defining nested array/map/struct functions except as return values of parsing functions.
- Defining physical input-file metadata functions.

## Guide-level explanation (how authors think about it)

Authors should be able to parse and produce semi-structured scalar values inside relational transformations:

```incan
from pub::inql.functions import col, from_json, get_json_object, sha2, to_json

events = (
    raw_events
        .with_column("payload_obj", from_json(col("payload"), EventPayload))
        .with_column("event_type", get_json_object(col("payload"), "$.type"))
        .with_column("user_hash", sha2(col("user_id"), 256))
        .with_column("payload_out", to_json(col("payload_obj")))
)
```

Reading a JSON file into a dataset remains a session/source operation, not a scalar function call.

## Reference-level explanation (precise rules)

InQL should define JSON functions including `from_json`, `to_json`, `get_json_object`, `json_array_length`, `json_object_keys`, `schema_of_json`, `parse_json`, `check_json`, `json_extract_path_text`, and `try_from_json` where recoverable parse behavior is desired.

InQL should define CSV value functions including `from_csv`, `to_csv`, and `schema_of_csv` only insofar as they operate on scalar values or schema descriptions. These functions must not replace the session CSV read/write contract.

InQL should define URL functions including `parse_url`, `url_encode`, `url_decode`, and `try_url_decode`, with exact invalid-input behavior recorded in the registry.

InQL should define hash functions including `crc32`, `md5`, `sha1`, `sha2`, and `xxhash64`, with input encoding and output representation specified.

Where InQL supports variant-like dynamic values, it should define type inspection and predicate functions such as `typeof`, `is_array`, `is_object`, `is_integer`, `is_timestamp`, and `is_null_value`. These functions must not be accepted before the value model they inspect is defined.

This RFC does not introduce a dynamic variant value model. Parser helpers that pass semi-structured payloads through the current scalar expression model return validated, normalized payload text. A later variant/value-model RFC may add typed semi-structured return values, but it must not silently change the meaning of the RFC 022 string-backed payload helpers.

Schema inference helper functions must be deterministic for the same input values and options. They must not inspect external files or session state unless explicitly defined as source-discovery functions outside this RFC.

## Design details

### Syntax

This RFC requires importable function forms. Format options may be positional, named, or option-record values, but their type and defaults must be specified before functions reach Planned status.

### Semantics

Strict parsing functions must fail on invalid input according to their registry error behavior. `try_` parsing functions must return null or another explicitly specified recoverable result on invalid input.

Hash functions must define whether they operate on UTF-8 string bytes, binary bytes, or typed value encodings. A hash over a typed value must not silently change encoding by backend.

### Interaction with other InQL surfaces

Format scalar functions may be used anywhere scalar expressions of their result type are valid. Source reading and writing remain governed by the session and format handler RFCs.

### Compatibility / migration

This RFC is additive. It should not change existing CSV ingestion behavior.

## Alternatives considered

- **Place all format helpers in the common scalar catalog.** Rejected because format parsing has option, schema, and I/O-adjacent concerns that deserve a separate boundary.
- **Make JSON and CSV functions source-only.** Rejected because scalar payload parsing is common inside already-loaded datasets.
- **Add full XML and variant support in the same RFC.** Rejected because those need their own type and compatibility discussion, even though this RFC may reserve JSON parsing and dynamic-value predicate names.

## Drawbacks

- Format functions can blur the line between scalar transformation and ingestion if docs are weak.
- Schema inference helpers can be expensive or misleading on small samples.
- Hash output compatibility requires careful encoding rules.

## Layers affected

- **InQL specification** — format functions must stay distinct from source and sink contracts.
- **InQL library package** — public helpers should expose JSON, CSV scalar, URL, and hash functions with explicit scalar argument contracts.
- **Incan compiler** — typechecking must validate current scalar argument shapes; no new compiler syntax is required by this RFC.
- **Execution / interchange** — Prism and Substrait lowering must preserve parser options, hash encodings, and scalar payload contracts or diagnose unsupported functions.
- **Documentation** — docs should distinguish scalar format functions from session read/write APIs.

## Design Decisions

### Resolved

- Hash helpers operate on UTF-8 string bytes and return lowercase hexadecimal strings.
- Portable concrete hash helpers are `md5`, `sha1`, `sha224`, `sha256`, `sha384`, `sha512`, `crc32`, and `xxhash64`, each with an honest Substrait extension mapping and DataFusion-backed execution coverage.
- `sha2(expr, bit_length)` is a compatibility helper, not a separate backend mapping. It rewrites to `sha224`, `sha256`, `sha384`, or `sha512` for supported literal bit lengths and rejects unsupported values.
- URL helpers accept scalar URL or component strings. `url_decode(...)` is strict and fails malformed percent escapes; `try_url_decode(...)` returns null for malformed percent escapes.
- JSON helpers accept scalar JSON payload strings. Strict helpers fail invalid JSON; `try_from_json(...)` returns null for invalid JSON; schema and path helpers are deterministic over the provided payload and literal schema/path arguments.
- CSV helpers accept scalar row strings and explicit schema-description strings. They operate on payload values only and do not replace the session CSV read/write contract.
- Dynamic-value predicate names such as `typeof`, `is_array`, `is_object`, `is_integer`, `is_timestamp`, and `is_null_value` are reserved by this RFC but intentionally not accepted until InQL defines a variant-like value model. This follows the reference rule above rather than creating fake string predicates.

## Implementation Plan

1. Add registry-backed hashing helpers under a logical function family.
2. Add registry-backed URL, JSON, and CSV scalar payload helpers under logical function families.
3. Add stable Substrait extension anchors for concrete helpers.
4. Keep `sha2(...)` as a compatibility rewrite over concrete helpers rather than a second mapping.
5. Add focused helper, registry, Substrait lowering, and DataFusion session tests with concrete output values.
6. Add user-facing format-function docs and release notes.
7. Reserve dynamic-value predicate names until a variant-like value model exists.

## Progress Checklist

- [x] RFC 022 moved to In Progress with full scalar format helper design decisions recorded.
- [x] `md5`, `sha1`, `sha224`, `sha256`, `sha384`, `sha512`, `sha2`, `crc32`, and `xxhash64` helpers added under the function catalog.
- [x] JSON, CSV, and URL scalar payload helpers added under the function catalog.
- [x] Concrete helpers registered with Substrait extension metadata.
- [x] `sha2(...)` implemented as a literal-bit-length rewrite with invalid-input diagnostics.
- [x] Focused helper, registry, Substrait lowering, and DataFusion-backed session tests added.
- [x] User-facing format-function docs and release notes added.
- [x] Dynamic-value predicate names reserved and rejected until a variant-like value model exists.
