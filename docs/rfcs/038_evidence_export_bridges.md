# InQL RFC 038: Evidence export bridges

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 029 (typed metadata attachments)
  - InQL RFC 030 (Prism lineage graph)
  - InQL RFC 031 (local inspection APIs and artifacts)
  - InQL RFC 032 (execution observations)
  - InQL RFC 036 (governed plan bundle)
  - InQL RFC 040 (interoperability semantic profiles)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines evidence export bridges from InQL's internal evidence model to external and adjacent formats. Export bridges map InQL plan, lineage, schema-flow, execution, quality, coverage, semantic profile, and bundle records into downstream views such as OpenLineage events, telemetry signals, semantic inspection fragments, and catalog/governance integration artifacts. Exports are projections from InQL evidence, not the internal source of truth.

## Motivation

InQL evidence should be useful outside InQL. CI systems, lineage tools, telemetry pipelines, catalogs, notebooks, and agents may all consume different formats. If each integration reconstructs evidence independently, semantics will drift. InQL should provide export bridges that preserve its local evidence model while acknowledging that external formats may be less expressive.

## Goals

- Define export bridges as downstream projections from InQL evidence.
- Preserve semantic target references and evidence versions where possible.
- Allow lossy external mappings only when loss is explicit.
- Keep provider configuration and hosted ingestion outside InQL core.
- Support local export without requiring a specific external service.

## Non-Goals

- Making any external format the internal InQL evidence model.
- Defining hosted ingestion, storage, dashboards, or managed governance behavior.
- Defining a telemetry provider, collector, exporter, or sampling policy.
- Guaranteeing that every external tool can represent every InQL evidence feature.

## Guide-level explanation (how authors think about it)

An author or CI job can export evidence from local artifacts:

```incan
bundle = governed_plan_bundle(summary)
bundle.export_openlineage("target/inql/openlineage.json")
bundle.export_telemetry("target/inql/telemetry.json")
```

The names are illustrative. The key contract is that exports are generated from InQL evidence artifacts, not from backend logs or reconstructed SQL strings.

## Reference-level explanation (precise rules)

An export bridge must declare its source evidence schema versions, target format, target format version when available, mapping coverage, unsupported fields, redaction behavior, and diagnostics.

Export bridges must preserve semantic target identifiers when the target format can carry them. When the target format cannot carry them directly, the bridge should preserve them in an extension, custom facet, attribute, or sidecar artifact when safe.

Lossy mappings must be explicit. If an external lineage format cannot distinguish value, control, grouping, join, and sort lineage, the export must either preserve the distinction through an extension or report the loss.

Sensitive attachments must follow visibility rules. Export bridges must not leak sensitive payloads merely because a target format lacks redaction semantics.

Provider configuration, authentication, network transport, sampling, hosted ingestion, and storage are outside this RFC.

## Design details

### Syntax

This RFC introduces no authoring syntax.

### Semantics

Exports are projections. They must not become the authoritative source of InQL plan, lineage, quality, or execution semantics.

### Interaction with other InQL surfaces

Export bridges depend on inspection artifacts, execution observations, quality observations, adapter coverage, and governed plan bundles. They should map from those records rather than from backend-specific plans.

### Compatibility / migration

Export bridges must version their mappings. Adding a new internal evidence field should not silently change external semantics without a mapping version change or documented behavior.

## Alternatives considered

- **Adopt one external lineage model internally.** Rejected because InQL needs evidence that many external tools cannot represent directly.
- **Leave all exports to downstream systems.** Rejected because independent reconstruction causes drift.
- **Require hosted ingestion.** Rejected because local export must work in open InQL.

## Drawbacks

- Export bridges require maintenance as external formats evolve.
- Some mappings will be lossy or require extensions.
- Redaction rules can make exports harder to debug.

## Layers affected

- **InQL specification** — export bridge responsibilities and loss reporting become normative.
- **InQL library package** — export APIs may live in core or optional modules.
- **Execution / interchange** — exports may include Substrait references, telemetry-shaped observations, and lineage events.
- **Documentation** — docs must identify external exports as projections, not internal truth.

## Unresolved questions

- Which export bridge should be implemented first?
- Should export bridges live in the core package or optional integration packages?
- What sidecar format should preserve InQL-specific evidence when an external target is lossy?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
