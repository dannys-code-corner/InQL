# InQL RFC 018: Common scalar function catalog

- **Status:** In Progress
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 015 (core scalar functions and operators)
- **Issue:** [InQL #35](https://github.com/dannys-code-corner/InQL/issues/35)
- **RFC PR:** [InQL #44](https://github.com/dannys-code-corner/InQL/pull/44)
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines the common scalar function catalog beyond the core scalar slice: practical math, string, binary, regex, and date/time functions that authors expect in a dataframe system. The catalog is standards-led where possible, preserves compatibility aliases where semantics match, and leaves specialist or format-specific families to later RFCs.

## Motivation

After the core scalar vocabulary exists, authors still need everyday data cleaning and feature engineering functions. Spark, Snowflake, DataFusion, Arrow, dbt, and SQL systems all provide broad scalar coverage because real tabular work needs string normalization, date extraction, numeric transforms, regex predicates, and parsing helpers. InQL should add that breadth deliberately rather than through scattered helper additions.

The catalog should still avoid copying every backend-specific function. Functions that require nested types, JSON/CSV values, geospatial types, sketch state, encryption policy, or physical execution metadata belong elsewhere.

## Goals

- Define common math, string, binary, regex, and date/time scalar functions.
- Specify canonical names and compatibility aliases at the family level.
- Require type, null, and error behavior to be registry-visible.
- Keep format-specific, nested-data, sketch, geospatial, and engine-specific functions out of this RFC.

## Non-Goals

- Defining collection, map, struct, JSON, CSV, XML, URL, geospatial, crypto, sketch, or UDF functions.
- Defining aggregate or window functions.
- Defining a complete SQL standard conformance matrix.
- Defining locale and collation semantics beyond what listed functions require.

## Guide-level explanation (how authors think about it)

Authors should be able to clean and enrich ordinary scalar columns using familiar functions:

```incan
from pub::inql.functions import col, concat, date_trunc, lower, round, substring, trim

cleaned = (
    orders
        .with_column("email_norm", lower(trim(col("email"))))
        .with_column("order_month", date_trunc("month", col("created_at")))
        .with_column("amount_rounded", round(col("amount"), 2))
        .with_column("sku_prefix", substring(col("sku"), 1, 3))
)
```

Compatibility aliases are useful, but authors should see one canonical InQL name in docs for each semantic function.

## Reference-level explanation (precise rules)

InQL should define common math functions including `abs`, `ceil`, `floor`, `round`, `sqrt`, `power`, `exp`, `ln`, `log`, `log10`, `sign`, `least`, `greatest`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `degrees`, and `radians`.

InQL should define common string functions including `char_length`, `octet_length`, `upper`, `lower`, `trim`, `ltrim`, `rtrim`, `substring`, `position`, `overlay`, `concat`, `concat_ws`, `replace`, `translate`, `repeat`, `left`, `right`, `lpad`, and `rpad`.

InQL should define common binary and encoding functions including `encode`, `decode`, `base64`, `unbase64`, `hex`, and `unhex`, provided their character encoding and invalid-input behavior are specified.

InQL should define common regex functions including `regexp_like`, `regexp_replace`, and `regexp_extract` once the regex flavor and capture semantics are specified.

InQL should define common date/time functions including `current_date`, `current_time`, `current_timestamp`, `extract`, `date_part`, `date_trunc`, `time_trunc`, `date_add`, `date_sub`, `date_diff`, `timestamp_diff`, `to_date`, `to_time`, `to_timestamp`, `from_unixtime`, `unix_seconds`, `unix_millis`, `unix_micros`, `make_date`, `make_time`, and `make_timestamp`.

The catalog should explicitly account for dbt-style portability names such as `dateadd`, `datediff`, `safe_cast`, `split_part`, `last_day`, `hash`, and type-name helpers. These names may be canonical, aliases, or extension helpers, but their adapter-specific rendering requirements must be represented through the registry rather than undocumented backend conditionals.

Every function added by this RFC must be entered in the function registry with type rules, null behavior, determinism, and backend support. Current time functions must be marked stable within a query or explicitly nondeterministic; the registry must not leave that ambiguous.

## Design details

### Syntax

This RFC requires importable function names. Query syntax may expose SQL keywords such as `EXTRACT`, but those spellings must map to registry entries.

### Semantics

Canonical names should prefer SQL-standard or widely portable names where semantics match. Compatibility aliases may include Spark-style names such as `substr`, `ucase`, `lcase`, `dateadd`, or `datediff` where they are unambiguous.

String functions must define whether indices are one-based or zero-based. Date/time functions must define timezone and session-time behavior before they can reach Planned status.

### Interaction with other InQL surfaces

The same scalar catalog must be usable in filters, computed projections, grouping keys where allowed, aggregate arguments, and query-block expressions. Function availability must not depend on whether the author used dataframe methods or query blocks.

### Compatibility / migration

Existing arithmetic helpers should be treated as the start of this catalog but should remain compatible with the core scalar names. Broader aliases should be additive.

## Alternatives considered

- **Add all Spark scalar functions.** Rejected because many Spark functions are format-specific, engine-specific, physical-execution-specific, or require types InQL has not standardized.
- **Only expose DataFusion functions.** Rejected because backend availability should inform lowering, not define the portable InQL author contract.
- **Delay scalar breadth until every edge case is settled.** Rejected because practical data cleaning needs a common catalog, but unresolved families can remain Draft until semantics are precise.

## Drawbacks

- A broad scalar catalog increases documentation and test obligations.
- String indexing, regex flavor, timezone behavior, and numeric edge cases are easy places for backend drift.
- Compatibility aliases can make the surface look larger than it really is.

## Layers affected

- **InQL specification** — the common scalar catalog must extend the registry without contradicting core scalar semantics.
- **InQL library package** — public helpers should expose canonical names and selected aliases.
- **Incan compiler** — query-block operator and keyword forms should lower to registry entries where applicable.
- **Execution / interchange** — Prism and Substrait lowering must either preserve semantics, use registered extensions, or diagnose unsupported functions.
- **Documentation** — function reference docs should group scalar functions by family and show aliases.

## Implementation Plan

### Phase 1: Math scalar slice

- Add a small portable math family with honest Substrait and DataFusion behavior.
- Cover helper expression shape, registry metadata, Substrait lowering, and concrete session execution.

### Phase 2: Remaining math functions

- Add the remaining math functions once type and edge-case behavior is specified clearly enough to be portable.

### Phase 3: String functions

- Add string normalization and composition helpers after string position/indexing decisions are recorded.

### Phase 4: Binary, regex, and date/time functions

- Add binary/encoding helpers, regex helpers, and date/time helpers only after their invalid-input, regex flavor, timezone, and session-time behavior are settled.

## Progress Checklist

### Implemented in PR #44

- [x] Add `abs`, `ceil`, `floor`, and single-argument `round` helpers under `src/functions/math/`.
- [x] Add registry metadata and Substrait extension mappings for the math slice.
- [x] Add scalar lowering tests and DataFusion-backed session execution tests with concrete output values.
- [x] Publish the implemented math slice in the function reference docs.

### Remaining RFC 018 scope

- [ ] Add remaining math helpers such as `sqrt`, `power`, logarithms, trigonometry, `least`, and `greatest`.
- [ ] Add string helpers such as `upper`, `lower`, `trim`, `substring`, `position`, `concat`, and padding/replacement helpers.
- [ ] Add binary and encoding helpers with explicit invalid-input behavior.
- [ ] Add regex helpers after regex flavor and capture semantics are specified.
- [ ] Add date/time helpers after timezone and session-time behavior are specified.
- [ ] Add compatibility aliases where they are semantic aliases rather than backend spellings.

## Unresolved questions

- Should string positions be one-based for SQL compatibility or zero-based for host-language familiarity?
- Which regex engine and regex feature set should InQL standardize?
- What is the session timezone model for current timestamp and timestamp conversion functions?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
