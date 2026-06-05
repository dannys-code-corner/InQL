# InQL RFC 032: Execution observations

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 004 (execution context)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 031 (local inspection APIs and artifacts)
  - InQL RFC 040 (interoperability semantic profiles)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines execution observations for InQL sessions. Execution observations correlate runtime attempts with semantic plan targets and record backend, adapter, semantic profile context, status, timing, diagnostics, row counts, and optional trace identifiers without making runtime logs the source of relational semantics.

## Motivation

After a plan executes, users and tools need evidence about what was attempted and what happened. A result table alone cannot explain which plan version ran, which adapter executed it, which diagnostics occurred, or how runtime observations attach to semantic targets. InQL needs a lightweight execution observation model that is structural, redacted by default, and independent of any particular telemetry backend.

## Goals

- Define execution attempts as semantic targets.
- Correlate session execution with plan identity.
- Record backend and adapter information, semantic profile context when available, status, diagnostics, row counts, and timing.
- Allow optional trace/log/metric correlation without requiring a telemetry provider.
- Preserve redaction defaults for payload-heavy or sensitive data.

## Non-Goals

- Defining an OpenTelemetry provider, collector, exporter, or sampling policy.
- Defining pipeline orchestration or retry semantics.
- Recording row payloads, samples, or full backend logs by default.
- Letting backend execution redefine plan lineage or schema semantics.

## Guide-level explanation (how authors think about it)

An author can collect data and then inspect the observation:

```incan
result = session.collect(summary)
observation = result.execution_observation()

assert observation.plan_id == inspect_plan(summary).plan_id
assert observation.status == "success"
```

Execution evidence explains the run. It does not replace plan inspection.

## Reference-level explanation (precise rules)

An execution observation must include an execution attempt target, plan target, session or binding context reference, backend name, adapter version when available, start time, end time or duration, status, diagnostics, and optional row count, byte count, trace identifier, requested semantic profile, and observed semantic profile.

Execution status must distinguish at least success, failure, cancelled, skipped, and unsupported.

Diagnostics must be structured records. Sensitive values, row samples, query payloads, credentials, and source data must not be included by default.

An execution observation may reference local inspection artifacts or semantic targets produced before execution. It must not mutate authored Prism targets or claim lineage that was not present in the plan evidence model.

Telemetry integrations may emit equivalent spans, events, logs, or metrics, but the InQL observation model must remain usable when no telemetry backend is configured.

## Design details

### Syntax

This RFC introduces no syntax.

### Semantics

Execution observations are runtime evidence. They describe an attempt to execute a semantic plan through a session and adapter.

### Interaction with other InQL surfaces

Quality observations, adapter coverage records, semantic profile records, and export bridges may refer to execution observations. Pipeline layers may consume them, but orchestration behavior remains outside this RFC.

### Compatibility / migration

Existing session execution remains valid. Implementations may initially emit partial observations, but unsupported fields must be explicit rather than silently omitted when consumers request them.

## Alternatives considered

- **Use backend logs only.** Rejected because logs are not stable semantic evidence and may be sensitive.
- **Require OpenTelemetry for all observations.** Rejected because local InQL evidence should work without provider configuration.
- **Attach execution data directly to plan nodes.** Rejected because runtime attempts are lifecycle events, not authored plan structure.

## Drawbacks

- Observation records add runtime overhead.
- Redaction can make diagnostics less convenient if users expect full backend logs.
- Correlating observations with plan snapshots requires stable semantic identity.

## Layers affected

- **InQL specification** — execution observation fields and status values become normative.
- **InQL library package** — Session results must expose observations.
- **Execution / interchange** — adapters must report execution facts without redefining semantics.
- **Documentation** — docs must explain observation redaction and telemetry independence.

## Unresolved questions

- What minimum observation fields are required for every adapter?
- Should failed planning, binding, and lowering attempts share the same observation model as execution attempts?
- How should trace identifiers be represented when multiple telemetry systems are active?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
