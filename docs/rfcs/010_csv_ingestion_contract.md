# InQL RFC 010: CSV dialect and interpretation contract

- **Status:** Draft
- **Created:** 2026-04-19
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 001 (dataset types and carrier schema surfaces)
  - InQL RFC 004 (execution context and session read boundaries)
  - InQL RFC 009 (session format handler registry)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2-rc5
- **Shipped in:** —

## Summary

This RFC defines InQL's north-star CSV dialect and interpretation contract. It standardizes how authors describe CSV dialect, header policy, malformed-row behavior, and schema or type inference so that the `csv` format behaves predictably across execution backends and future format-handler implementations. The core claim is that CSV parsing semantics must be expressed as stable structured InQL configuration and validated as part of the InQL contract, rather than being left to whatever parser quirks a specific backend happens to expose.

## Core model

1. CSV ingestion configuration is structured data, not ad-hoc booleans or backend-specific string flags.
2. CSV **dialect**, **schema or type interpretation**, and **malformed-row policy** are distinct concerns and must remain separate in the public API.
3. CSV reads participate in InQL's schema layering: declared schema, planned schema, and resolved schema are related but not interchangeable.
4. Execution backends and format handlers must either honor the requested CSV contract or reject unsupported configuration before query execution begins.
5. `Session.read_csv(...)` is convenience sugar over the session's format-dispatch surface for the `csv` format key.

## Motivation

CSV is deceptively simple. In practice, a read surface that says "read CSV" without a precise contract leaves critical behavior undefined: quoting, embedded delimiters, multiline fields, header handling, null tokens, numeric grouping, timestamp recognition, and malformed-row policy. That creates two problems for InQL.

First, authors cannot reason about portability. A workflow that succeeds with one execution backend may silently change meaning under another backend or after an internal parser change. Second, InQL's typed carrier model becomes weaker if CSV ingestion can quietly reinterpret columns or infer different shapes based on parser quirks rather than an explicit contract.

InQL should not inherit the accidental complexity of backend-specific CSV readers as its user-facing semantics. It needs its own contract: explicit enough that authors can rely on it, and precise enough that conformance tests can verify it.

## Goals

- Define a stable, backend-portable CSV dialect and interpretation contract for `Session.read_csv(...)`.
- Separate CSV dialect configuration from schema or type inference and malformed-row handling.
- Specify the default CSV behavior authors get when they do not override options.
- Define how CSV reads interact with declared, planned, and resolved schema layers.
- Require early rejection when a backend or format handler cannot satisfy the requested CSV contract.
- Make conformance testing possible without binding InQL semantics to one specific parser implementation.

## Non-Goals

- Achieving full behavioral parity with pandas, Spark, DuckDB, or any other existing CSV reader.
- Standardizing every possible locale-specific number, date, or timestamp convention in this RFC.
- Defining CSV write semantics; this RFC is about ingestion.
- Defining directory, prefix, archive, compression, or other source-discovery semantics for where CSV parse units come from.
- Turning malformed-row recovery into a general data-quality framework.
- Prescribing one specific internal parser architecture or one specific backend implementation strategy.

## Guide-level explanation (how authors think about it)

Authors should think of CSV reads as a contract between their program and the runtime, not as a best-effort convenience helper. The contract has three parts:

- how the file is tokenized and decoded
- how tokens are interpreted as typed values
- what should happen when the input is malformed or ambiguous

The public surface should therefore expose one structured options value for CSV reads.

```incan
from pub::inql import Session
from pub::inql.formats import (
    CsvDialect,
    CsvHeaderMode,
    CsvInference,
    CsvReadOptions,
    MalformedRowPolicy,
    NumericGroupingPolicy,
)
from models import Order

session = Session.default()

orders = session.read_csv[Order](
    "orders",
    "s3://warehouse/orders.csv",
    CsvReadOptions(
        dialect=CsvDialect(
            delimiter=",",
            quote="\"",
            header=CsvHeaderMode.Present,
            multiline_fields=true,
        ),
        inference=CsvInference(
            null_tokens=["", "NULL"],
            numeric_grouping=NumericGroupingPolicy.AllowCommonSeparators,
        ),
        malformed_rows=MalformedRowPolicy.Error,
    ),
)
```

The same contract should be reachable through the generic format-dispatch path. `read_csv` is convenience API surface, not a separate semantic contract.

```incan
orders = session.read_format[Order](
    logical_name="orders",
    source="s3://warehouse/orders.csv",
    format="csv",
    options=CsvReadOptions(
        dialect=CsvDialect(
            delimiter=",",
            quote="\"",
            header=CsvHeaderMode.Present,
            multiline_fields=true,
        ),
        inference=CsvInference(
            null_tokens=["", "NULL"],
            numeric_grouping=NumericGroupingPolicy.AllowCommonSeparators,
        ),
        malformed_rows=MalformedRowPolicy.Error,
    ),
)
```

If an author does not provide options, InQL still has a defined default contract rather than an implementation accident.

```incan
orders = session.read_csv[Order]("orders", "s3://warehouse/orders.csv")
```

That default should mean "standard CSV with headers, quoted-field support, strict malformed-row handling, and deterministic type interpretation rules", not "whatever the current backend happened to do this week."

Headerless CSV is still allowed, but it should be explicit because it changes how schema is established.

```incan
from pub::inql.formats import CsvDialect, CsvHeaderMode, CsvReadOptions

rows = session.read_csv[Order](
    "orders",
    "file:///tmp/orders_no_header.csv",
    CsvReadOptions(
        dialect=CsvDialect(header=CsvHeaderMode.Absent),
    ),
)
```

## Reference-level explanation (precise rules)

### Public configuration model

InQL must expose one stable structured CSV configuration surface for read operations. The exact namespace may evolve, but the public API must include the equivalent of:

- `CsvReadOptions`
- `CsvDialect`
- `CsvInference`
- `MalformedRowPolicy`
- `CsvHeaderMode`
- `NumericGroupingPolicy`

`Session.read_csv(...)` must accept a `CsvReadOptions` value, either explicitly or through an equivalent structured builder surface. Anonymous positional booleans or backend-specific flag bags must not be the normative API.

The normative contract is owned by the `csv` format entry in the session's format-dispatch surface. `Session.read_csv(...)` should remain convenience sugar over that same dispatch path rather than becoming a separate backend-specific semantic entry point.

### Dialect contract

`CsvDialect` must define at least:

- `delimiter`: one field separator character
- `quote`: one quote character used for quoted fields
- `header`: whether the first logical record is a header row
- `multiline_fields`: whether quoted fields may span multiple physical lines

InQL's default dialect must behave as follows:

- delimiter is `,`
- quote is `"`
- header mode is `Present`
- doubled quote characters inside a quoted field represent one literal quote
- quoted fields may contain delimiters and line breaks
- malformed rows are not silently ignored

The default contract should align with standard CSV expectations, but InQL owns the contract text. It must not outsource semantics to a backend documentation page.

### Schema and type interpretation

CSV ingestion must distinguish three schema layers:

- **declared schema**: the author- or model-level schema the program is written against
- **planned schema**: the schema InQL can establish before execution from configuration, headers, and compatible schema analysis
- **resolved schema**: the schema observed from materialized output after execution

When `T` is supplied in `read_csv[T](...)`, `T` is the declared schema contract. CSV ingestion must validate compatibility between the file and `T` according to the configured header and inference policy. A backend must not silently reorder columns or reinterpret field names in a way that breaks the declared schema.

When CSV options require token interpretation beyond raw strings, `CsvInference` must control that behavior. At minimum it must define:

- null-token recognition
- boolean lexical forms
- integer and decimal lexical forms
- timestamp or temporal lexical forms
- numeric grouping policy

If numeric grouping separators are allowed, the contract must define which separators are permitted and where they may appear. For example, `1_000.00` and `1,234.567` must not be accepted or rejected by accident; acceptance must follow the configured grouping policy.

### Malformed-row policy

InQL must expose an explicit malformed-row policy. At minimum the contract must support an `Error` mode that fails the read. Additional policies may exist, but they must be explicit and portable.

Malformed input includes at least:

- unmatched quotes
- row-width mismatches relative to the effective schema contract
- invalid escapes under the configured dialect
- values that violate required strict coercions when coercion is part of the declared contract

If a backend cannot implement a requested malformed-row policy faithfully, it must reject the read before execution rather than silently degrading behavior.

### Early validation and capability rejection

CSV option validation must happen before query execution starts. InQL must reject invalid option combinations and unsupported backend capabilities at the session or planning boundary.

Backends and format handlers may support more behavior internally than the portable contract defines, but that additional behavior must not leak into InQL's normative semantics unless a future RFC standardizes it.

### Diagnostics

CSV read failures must report typed diagnostics that distinguish at least:

- invalid CSV option configuration
- schema or header incompatibility
- malformed input
- backend capability mismatch
- I/O failure

Diagnostics should name the logical source and, when known, the relevant row or field boundary. They should not require authors to know backend internals in order to understand what failed.

## Design details

### Syntax

This RFC does not introduce new language grammar. It standardizes library-surface configuration for CSV ingestion.

### Semantics

`Session.read_csv(...)` is not merely "open a file and guess." It establishes a portable parsing and interpretation contract that downstream planning and execution must honor. CSV reads remain lazy from a data-execution perspective, but option validation and schema compatibility checks should occur as early as possible.

The default path must be strict enough to be predictable. Convenience cannot come from ambiguity. A permissive parser mode may exist, but only as an explicit policy choice.

### Interaction with other InQL surfaces

CSV ingestion must compose with carrier-schema layering from InQL RFC 001. `planned_columns` may be established from declared schema, header contract, or compatible pre-execution analysis, while `resolved_columns` remain a runtime fact on materialized output.

CSV ingestion must compose with the execution boundary from InQL RFC 004. Session-owned read configuration is part of the execution contract, not an implementation detail hidden in a backend adapter.

CSV-specific configuration should also remain compatible with the format-handler model in InQL RFC 009. The `csv` format entry owns the contract; `read_csv` is convenience sugar over that dispatch path. A CSV handler may implement the contract, but it does not get to redefine the contract.

### Compatibility / migration

This RFC is additive in intent, but existing code paths that depend on underspecified CSV behavior may observe stricter validation once the contract is implemented. That is acceptable: ambiguous ingestion behavior is technical debt, not a compatibility guarantee.

Implementations should provide a clear migration path for currently implicit defaults by documenting which old behavior maps to explicit `CsvReadOptions` values.

## Alternatives considered

- Leaving CSV semantics to the execution backend: rejected, because it makes InQL's read surface non-portable and weakens the typed carrier contract.
- Defining only a minimal RFC 4180 subset and deferring all inference semantics: rejected, because authors still need stable rules for nulls, numerics, booleans, and timestamps when reading typed data.
- Copying pandas or Spark behavior wholesale: rejected, because those systems optimize for their own ecosystems and backward-compatibility constraints. InQL should learn from them, not inherit them uncritically.
- Exposing only backend-specific option maps: rejected, because it would bypass InQL's role as the contract owner.

## Drawbacks

- The public API becomes larger and more explicit.
- Backends and format handlers take on a real conformance burden instead of "best effort" parsing freedom.
- Default choices around grouping separators, whitespace, and permissiveness will be contentious and must be defended clearly.

## Layers affected

- **InQL specification** must define CSV ingestion as a portable contract rather than a backend-specific convenience.
- **InQL library package** must expose stable structured CSV read options and typed diagnostics consistent with this RFC.
- **Execution / interchange** must validate backend or handler capability against the requested CSV contract before execution begins.
- **Documentation** must explain the default CSV contract and its explicit override model without relying on backend documentation as the source of truth.

## Unresolved questions

- Should headerless CSV be supported only when a declared schema is present, or may InQL synthesize stable placeholder column names as part of the portable contract?
- Should the default numeric-grouping policy remain strict and reject separators such as `_` and `,`, or should common grouping separators be allowed by default?
- Should unquoted leading and trailing whitespace be preserved exactly by default, or normalized as part of the portable CSV contract?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
