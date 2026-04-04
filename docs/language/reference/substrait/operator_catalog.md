# Substrait operator catalog (Reference)

This page is the **operational mapping reference** for InQL's Apache Substrait integration. The normative contract — including the Logical `Rel` alphabet, pinning policy, read-root boundary, and extension URI rules — lives in [InQL RFC 002][rfc-002]. This page provides the full capability → `Rel` catalog, profile tags, gap encoding requirements, and optional mutation profile detail that are too long-lived and versioned to remain inside the RFC text itself.

## Profile tags

Each entry in the catalog carries one of the following profile tags:

| Tag                   | Meaning                                                                                                                                                                                        |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **core**              | Maps to a standard logical `Rel` in the pinned Substrait revision; portable across conforming consumers without additional extension registration                                              |
| **extension**         | Requires a registered extension URI; portability depends on consumer support for that URI                                                                                                      |
| **gap**               | No stable logical `Rel` exists in current core Substrait; **must** use a documented non-core encoding (see [Gap profiles](#gap-profiles)); ad hoc or undocumented encodings are non-conforming |
| **optional-mutation** | Part of the optional mutation profile; not required for read/query analytical core; may be omitted by distributions that target read-only analytical use                                       |

The same status taxonomy is used in the [Substrait conformance corpus][ref-conformance-corpus]. Scenario contracts in that corpus are represented as typed InQL models with stable scenario IDs so CI and downstream implementations can consume a stable machine contract.

## Read/query analytical core profile

The following table maps InQL plan capabilities to Substrait logical relations and expression patterns for the read/query analytical core — the minimum required for InQL v0.1.

| InQL capability (conceptual)                                  | Substrait                                                                                                                                 | Profile |
| ------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| Logical table / registered name                               | `ReadRel` + `NamedTable`                                                                                                                  | core    |
| File or object scan as plan input                             | `ReadRel` + `LocalFiles` (format options in the pinned spec)                                                                              | core    |
| Literal or embedded rows                                      | `ReadRel` + `VirtualTable`                                                                                                                | core    |
| Predicate pushdown into scan                                  | `ReadRel` filter fields and/or separate `FilterRel` — producer policy; **must** be documented per implementation                          | core    |
| Row filter                                                    | `FilterRel`                                                                                                                               | core    |
| Add or replace computed columns                               | `ProjectRel`; current package code provides a boundary scaffold, while richer computed/window semantics remain deferred                         | core    |
| Inner join                                                    | `JoinRel` (inner variant)                                                                                                                 | core    |
| Left join                                                     | `JoinRel` (left outer variant)                                                                                                            | core    |
| Semi, anti, single, mark join variants                        | `JoinRel` (respective variant; optional `post_join_filter`)                                                                               | core    |
| Cross join                                                    | `CrossRel`                                                                                                                                | core    |
| Group by / aggregates                                         | `AggregateRel`; current package code provides a boundary scaffold, while richer grouping/measure semantics remain deferred                   | core    |
| Rollup / cube / grouping sets                                 | `AggregateRel` with multiple groupings                                                                                                    | core    |
| Distinct rows                                                 | `AggregateRel` with grouping keys and no measures                                                                                         | core    |
| Window / analytic functions                                   | `ProjectRel` with window expressions                                                                                                      | core    |
| Sort                                                          | `SortRel`                                                                                                                                 | core    |
| Limit / offset                                                | `FetchRel`                                                                                                                                | core    |
| Union, intersect, except                                      | `SetRel` with the appropriate set operation enum                                                                                          | core    |
| Reuse of an identical subplan                                 | `Plan` + `ReferenceRel`                                                                                                                   | core    |
| Unnest / explode                                              | Extension rel or documented expansion — **must** be pinned per implementation (see [Unnest / explode](#unnest--explode))                  | gap     |
| Pivot / unpivot                                               | `ExtensionSingleRel` or documented rewrite to join + aggregate + project (see [Pivot / unpivot](#pivot--unpivot))                         | gap     |
| Asof / interval join                                          | Gap or non-equi join expression only where consumer contract explicitly allows (see [Asof / interval joins](#asof--interval-joins))       | gap     |
| Streaming time semantics (watermarks, session windows, state) | Outside core Substrait unless via named extensions or a separate execution IR (see [Streaming time semantics](#streaming-time-semantics)) | gap     |

## Optional mutation profile

The following capabilities are part of the optional mutation profile. They are **not** required for InQL read/query analytical core (v0.1). An implementation that exposes any mutation-profile capability **must** document which relations are supported for its target backend and what portability guarantees (if any) apply.

| InQL capability                               | Substrait   | Profile           |
| --------------------------------------------- | ----------- | ----------------- |
| Write to a table / CTAS                       | `WriteRel`  | optional-mutation |
| Table update without a full child `Rel` input | `UpdateRel` | optional-mutation |
| DDL (create, drop, alter)                     | `DdlRel`    | optional-mutation |

Absence of these in a given distribution does not make InQL incomplete for read-only analytical use.

## Extension escape hatches

When no standard logical `Rel` covers a required operation and no gap encoding policy applies, implementations **may** use the following extension escape hatches. Any use **must** be declared in the public operator catalog for the toolchain version and assigned a stable, registered extension URI.

| Extension `Rel`      | When appropriate                                              |
| -------------------- | ------------------------------------------------------------- |
| `ExtensionLeafRel`   | Source/scan with no applicable standard `ReadRel` variant     |
| `ExtensionSingleRel` | Single-input transformation with no applicable standard `Rel` |
| `ExtensionMultiRel`  | Multiple-input relation with no applicable standard `Rel`     |

Using an extension escape hatch without a registered URI is non-conforming.

## Gap profiles

### Unnest / explode

Core Substrait does not define a portable unnest or explode `Rel` at the logical level. Until a stable logical `Rel` for unnest is adopted in the pinned Substrait revision and recognized by InQL:

- `EXPLODE`-style behavior **must** lower through a registered extension relation (`ExtensionSingleRel` or `ExtensionLeafRel`) with a declared extension URI in the toolchain's public catalog.
- Alternatively, a documented rewrite (for example, expanding a virtual table) **may** be used if the encoding is unambiguously specified in the public operator catalog for the toolchain version.
- Implementations **must not** present ad hoc or undocumented unnest encodings as portable core behavior.

Current package-level RFC 002 boundary registration:

- `https://inql.io/extensions/v0.1/unnest.yaml#explode`

### Pivot / unpivot

No core Substrait `Rel` covers pivot/unpivot directly:

- The canonical encoding is a documented rewrite to `JoinRel` + `AggregateRel` + `ProjectRel`.
- Alternatively, `ExtensionSingleRel` **may** be used with a registered URI.
- The chosen encoding **must** be documented in the public operator catalog for the toolchain version.

### Asof / interval joins

Asof and interval joins fall outside `JoinRel`'s standard join type enum:

- They **may** be expressed as a `JoinRel` with a complex non-equi join expression only where the consumer's documentation explicitly states it handles such expressions correctly.
- If expressed as an extension, a registered URI is required.
- The expected consumer behavior **must** be stated in the operator catalog entry for the toolchain version.

### Streaming time semantics

Watermarks, session windows, and stateful streaming operations are outside the scope of core Substrait's logical plan language:

- They **must** be expressed either through registered named extensions with explicitly documented semantics, or through a separate execution-level IR that is not the normative Substrait interchange.
- Any use of streaming extensions in the normative Substrait output **must** be documented as non-portable to non-streaming consumers.
- The operator catalog entry **must** state which consumers are known to handle the extension correctly.

<!-- Link references (single place for targets) -->

[rfc-002]: ../../rfcs/002_apache_substrait_integration.md
[ref-conformance-corpus]: ./conformance.md
