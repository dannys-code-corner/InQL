# InQL RFC 031: Local inspection APIs and artifacts

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 029 (typed metadata attachments)
  - InQL RFC 030 (Prism lineage graph)
  - InQL RFC 040 (interoperability semantic profiles)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines local inspection APIs and deterministic evidence artifacts for InQL plans. The APIs expose plan structure, schema flow, lineage, metadata attachments, semantic profile evidence, ingress evidence, and diagnostics as typed records, while artifacts provide versioned serialized views suitable for CI, IDEs, agents, documentation, and downstream integrations.

## Motivation

Relational evidence is only useful if authors and tools can inspect it without scraping logs or formatted explanations. InQL needs local APIs and artifacts that work without a hosted service, without a catalog product, and without executing the plan. This keeps plan inspection open, reproducible, and testable.

## Goals

- Define local inspection APIs over InQL plans.
- Define deterministic artifact families for plan graph, lineage graph, schema flow, metadata attachments, semantic profiles, ingress mappings, client session context, and diagnostics.
- Require artifact versioning and unsupported-evidence markers.
- Keep human reports as projections from structured artifacts.

## Non-Goals

- Defining a UI, hosted artifact store, or managed catalog.
- Defining every external export mapping.
- Requiring plan execution before inspection.
- Making Markdown or console output the primary evidence format.

## Guide-level explanation (how authors think about it)

An author can inspect a lazy plan locally:

```incan
from pub::inql.inspect import inspect_plan

inspection = inspect_plan(summary)
inspection.output_schema()
inspection.lineage().field("total_amount")
```

The same inspection data can be written as artifacts for CI or downstream tools:

```incan
inspection.write_artifacts("target/inql")
```

The exact helper names are illustrative; the contract is that structured inspection exists before execution.

## Reference-level explanation (precise rules)

InQL must expose a local inspection capability for plans that returns typed records, not only formatted strings.

Inspection records must include semantic targets, output schema information, relation structure, lineage when available, metadata attachments when available, semantic profile records or assessments when available, ingress origin mappings, client session context, and frontend coverage when available, diagnostics, and evidence-version metadata.

InQL must define deterministic serialized artifacts for at least:

- plan graph
- lineage graph
- schema flow
- metadata attachments
- semantic profiles
- ingress mappings
- client session context
- diagnostics

Artifacts must include schema version, InQL version, relevant rule versions, target identifiers, and unsupported-evidence markers. An empty lineage graph must be distinguishable from lineage that was not computed or is not supported.

Human-readable reports may exist, but they must be generated from structured inspection records or artifacts.

Sensitive attachments must be redacted or omitted according to the visibility rules from InQL RFC 029.

## Design details

### Syntax

This RFC introduces no language syntax.

### Semantics

Inspection is read-only. It must not execute a plan, bind physical sources, mutate Prism-authored meaning, or make policy decisions.

### Interaction with other InQL surfaces

Method-chain, query-block, and future authoring surfaces should be inspectable through the same API once they lower to Prism.

### Compatibility / migration

Existing code remains valid. New tooling should prefer structured inspection over parsing `repr`, `debug`, or backend plan strings.

## Alternatives considered

- **Only expose formatted explanations.** Rejected because tools need structured data.
- **Only emit files, no API.** Rejected because IDEs and tests need in-memory inspection.
- **Wait for a higher-level catalog.** Rejected because local InQL users need inspection without external services.

## Drawbacks

- Artifact schemas create compatibility obligations.
- Deterministic output may constrain internal representation changes.
- Multiple artifact families require clear documentation.

## Layers affected

- **InQL specification** — local inspection becomes part of the relational evidence contract.
- **InQL library package** — inspection APIs and artifact writers must expose structured records.
- **Execution / interchange** — no execution is required, but artifacts may reference Substrait lowering status.
- **Documentation** — docs must present artifacts as primary evidence and reports as derived views.

## Unresolved questions

- Which artifact serialization format should be mandatory first?
- Should artifact writing be part of the core package or a separate tooling module?
- How stable must artifact ordering be for snapshot tests and CI diffs?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
