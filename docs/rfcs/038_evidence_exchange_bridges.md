# InQL RFC 038: Evidence exchange bridges

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
- **Issue:** [InQL #72](https://github.com/dannys-code-corner/InQL/issues/72)
- **RFC PR:** [InQL #60](https://github.com/dannys-code-corner/InQL/pull/60)
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines evidence exchange bridges between InQL's internal evidence model and external or adjacent formats. Exchange bridges map InQL plan, lineage, schema-flow, execution, quality, coverage, semantic profile, and bundle records into downstream views such as OpenLineage events, telemetry signals, semantic inspection fragments, transformation-project artifacts, and catalog/governance integration artifacts. They may also ingest external evidence artifacts such as transformation manifests, source catalogs, schema catalogs, run results, and orchestration metadata. Representative artifact families include dbt manifests and run results, Glue Data Catalog or Hive Metastore snapshots, Airflow or MWAA DAG metadata, Dagster assets, Prefect deployment metadata, OpenLineage events, DataHub or OpenMetadata catalog records, and Great Expectations-style quality results. Inbound artifacts and outbound projections are evidence exchange records, not the internal source of truth.

## Motivation

InQL evidence should be useful outside InQL, and external project artifacts should be usable as evidence inputs when they are explicit about their source and scope. CI systems, lineage tools, telemetry pipelines, catalogs, notebooks, transformation frameworks, orchestrators, and agents may all consume or produce different formats. Systems such as dbt, Airflow, MWAA, Dagster, Prefect, Glue Data Catalog, Hive Metastore, DataHub, OpenMetadata, OpenLineage, and Great Expectations are useful ecosystem examples, but none of them should become InQL's internal evidence model. If each integration reconstructs evidence independently, semantics will drift. InQL should provide exchange bridges that preserve its local evidence model while acknowledging that external formats may be less expressive or may represent facts at a different semantic layer.

## Goals

- Define exchange bridges as inbound and outbound mappings around InQL evidence.
- Preserve semantic target references and evidence versions where possible.
- Allow lossy external mappings only when loss is explicit.
- Allow external artifacts to seed metadata, lineage hints, quality observations, run observations, and target mappings without becoming authoritative InQL semantics.
- Support transformation-framework artifacts such as manifests, catalogs, run results, source definitions, model metadata, tests, tags, and documentation scaffolds, including dbt-shaped artifacts where a bridge supports that profile.
- Keep provider configuration and hosted ingestion outside InQL core.
- Support local exchange without requiring a specific external service.

## Non-Goals

- Making any external format the internal InQL evidence model.
- Defining hosted ingestion, storage, dashboards, or managed governance behavior.
- Defining a telemetry provider, collector, exporter, or sampling policy.
- Guaranteeing that every external tool can represent every InQL evidence feature.
- Guaranteeing that imported transformation, catalog, or orchestration artifacts are complete or semantically authoritative.
- Defining a full migration product, transformation runtime, or orchestration engine.

## Guide-level explanation (how authors think about it)

An author or CI job can exchange evidence with local artifacts:

```incan
bundle = governed_plan_bundle(summary)
bundle.export_openlineage("target/inql/openlineage.json")
bundle.export_telemetry("target/inql/telemetry.json")
```

The names are illustrative. The key contract is that outbound exports are generated from InQL evidence artifacts, not from backend logs or reconstructed SQL strings.

For transformation-project workflows, an exchange bridge can also ingest project artifacts and emit reviewable suggestions:

```incan
project = transformation_project_artifacts("analytics_project/")
bundle = governed_plan_bundle(summary, evidence=[project])

sources = bundle.export_transformation_sources()
tests = bundle.export_transformation_quality_suggestions()
```

The bridge may read common artifacts such as manifests, catalogs, run results, source definitions, tests, tags, metadata, and documentation fragments. In a dbt-shaped bridge, for example, those inputs may include `manifest.json`, `catalog.json`, `run_results.json`, source YAML, model YAML, tags, exposures, tests, and documentation blocks. It may emit suggested source declarations, model metadata, test definitions, tags, exposures, or documentation scaffolds. Those suggestions remain projections from InQL evidence and imported artifact evidence; they do not make the transformation framework the semantic owner of the plan.

## Reference-level explanation (precise rules)

An exchange bridge must declare its direction, source evidence schema versions, target format, target format version when available, mapping coverage, unsupported fields, redaction behavior, and diagnostics.

Outbound exchange bridges must preserve semantic target identifiers when the target format can carry them. When the target format cannot carry them directly, the bridge should preserve them in an extension, custom facet, attribute, or sidecar artifact when safe.

Inbound exchange bridges must preserve external artifact identity, source location, artifact version, and confidence. Imported records may attach metadata, origin hints, observed run facts, quality observations, or candidate mappings to InQL semantic targets. They must not create InQL lineage, policy decisions, quality pass/fail states, or adapter coverage unless the corresponding InQL evidence contract can represent and validate that evidence.

Lossy mappings must be explicit. If an external lineage format cannot distinguish value, control, grouping, join, and sort lineage, the bridge must either preserve the distinction through an extension or report the loss. If an imported artifact collapses source relation, model, test, and run-result semantics into one node vocabulary, the bridge must report that limitation instead of pretending the artifact has InQL target precision.

Sensitive attachments must follow visibility rules. Exchange bridges must not leak sensitive payloads merely because a target format lacks redaction semantics.

Provider configuration, authentication, network transport, sampling, hosted ingestion, and storage are outside this RFC.

## Design details

### Syntax

This RFC introduces no authoring syntax.

### Semantics

Outbound exports are projections. Inbound artifacts are evidence inputs. Neither direction may become the authoritative source of InQL plan, lineage, quality, or execution semantics.

### Interaction with other InQL surfaces

Exchange bridges depend on inspection artifacts, execution observations, quality observations, adapter coverage, interoperability profiles, and governed plan bundles. They should map from or into those records rather than from backend-specific plans.

Transformation-framework bridges are a first-class example. A bridge may ingest manifest, catalog, run-result, source, model, test, tag, exposure, metadata, and documentation artifacts from systems such as dbt, Airflow, MWAA, Dagster, or Prefect when the bridge profile supports them. It may export suggested source definitions, model metadata, quality tests, documentation scaffolds, exposures, tags, or run validation summaries. The bridge must keep imported project semantics distinct from Prism-authored semantics and must identify any profile assumptions used to compare source and target environments.

### Compatibility / migration

Exchange bridges must version their mappings. Adding a new internal evidence field should not silently change external semantics without a mapping version change or documented behavior. Imported artifact schemas must be versioned or fingerprinted where possible so stale or incompatible artifacts can be diagnosed.

## Alternatives considered

- **Adopt one external lineage model internally.** Rejected because InQL needs evidence that many external tools cannot represent directly.
- **Leave all exchange to downstream systems.** Rejected because independent reconstruction causes drift.
- **Require hosted ingestion.** Rejected because local export must work in open InQL.
- **Treat transformation project artifacts as authoritative semantics.** Rejected because those artifacts are valuable evidence, but they are not Prism's analyzed relational model.

## Drawbacks

- Exchange bridges require maintenance as external formats evolve.
- Some mappings will be lossy or require extensions.
- Redaction rules can make exports harder to debug.
- Inbound artifact support can be mistaken for semantic endorsement unless confidence, source, and target mapping diagnostics are explicit.

## Layers affected

- **InQL specification** — exchange bridge responsibilities and loss reporting become normative.
- **InQL library package** — exchange APIs may live in core or optional modules.
- **Execution / interchange** — exchanges may include Substrait references, telemetry-shaped observations, lineage events, transformation artifacts, and run-result evidence.
- **Documentation** — docs must identify external exchanges as evidence inputs or projections, not internal truth.

## Unresolved questions

- Which exchange bridge should be implemented first?
- Should exchange bridges live in the core package or optional integration packages?
- What sidecar format should preserve InQL-specific evidence when an external target is lossy?
- Which transformation-project artifacts should be supported in the first bridge slice?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
