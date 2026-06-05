# InQL RFC 040: Interoperability semantic profiles

- **Status:** Draft
- **Created:** 2026-05-30
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (core language model and layer boundaries)
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 004 (execution context)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 008 (optimizer boundary, statistics, cost-based optimization, and adaptive execution)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 024 (function extension policy)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 029 (typed metadata attachments)
  - InQL RFC 030 (Prism lineage graph)
  - InQL RFC 031 (local inspection APIs and artifacts)
  - InQL RFC 032 (execution observations)
  - InQL RFC 033 (adapter requirements and coverage)
  - InQL RFC 036 (governed plan bundle)
  - InQL RFC 038 (evidence export bridges)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines interoperability semantic profiles for InQL evidence. A profile describes the semantic environment a plan is being received from, compared with, targeted at, or observed under: an InQL baseline, client protocol, plan ingress frontend, execution engine, adapter binding, SQL dialect, interchange consumer, or conformance baseline. Profiles give ingress coverage records, adapter requirements, coverage records, execution observations, plan diffs, bundles, and exports a shared context without making any external system the owner of InQL relational meaning.

## Motivation

Interoperability requires more than lowering a plan and asking whether an adapter has a support flag. Different target environments can share the same relational vocabulary while differing on edge semantics: type coercion, decimal overflow, timestamp and timezone behavior, identifier resolution, null and NaN ordering, collation, case sensitivity, function definitions, aggregate edge cases, window defaults, nested data behavior, row ordering, and fallback execution.

If InQL does not name the semantic profile used for an inspection or execution, those assumptions will be scattered across adapters, Substrait metadata, docs, and runtime diagnostics. That would make coverage hard to trust. A plan could appear portable while relying on target-specific behavior that was never recorded as evidence.

Profiles provide the missing layer between InQL-authored semantics, plan ingress, and adapter coverage. Prism remains the source of authored and rewritten relational meaning. Profiles describe source and target environments well enough for InQL to produce ingress diagnostics, requirements, coverage records, and observations against them.

## Goals

- Define semantic profiles as versioned evidence records.
- Allow profiles for InQL baselines, client protocols, plan ingress frontends, execution engines, adapter bindings, SQL dialects, interchange consumers, and conformance baselines.
- Name the semantic dimensions that affect relational correctness and evidence interpretation.
- Let adapter requirements and coverage records state which profile they were evaluated against.
- Let execution observations report the profile requested before execution and the profile observed at runtime when available.
- Keep profiles local and open, without requiring a hosted registry or managed control plane.
- Keep external target profiles non-authoritative for InQL semantics.

## Non-Goals

- Defining a profile for one specific external engine.
- Making any external engine, SQL dialect, or interchange format the normative InQL semantic model.
- Defining SQL transpilation, physical planning, or backend execution strategies.
- Defining a full conformance test suite.
- Defining a global registry of every engine version or deployment configuration.
- Guaranteeing semantic equivalence merely because a profile name is present.

## Guide-level explanation (how authors think about it)

Most authors should encounter profiles through inspection, coverage, and execution evidence:

```incan
from pub::inql.inspect import inspect_plan

inspection = inspect_plan(summary)
profile = inspection.semantic_profile("portable_relational")

requirements = inspection.adapter_requirements(profile)
coverage = session.check_coverage(summary, target_profile=profile)
```

The exact API names are illustrative. The important model is that the target profile is explicit. A coverage report should be able to say which semantic profile was used and which dimensions are covered, constrained, mismatched, or unknown.

Execution can attach the same evidence context:

```incan
result = session.collect(summary, target_profile=profile)
observation = result.execution_observation()

assert observation.requested_profile == profile.id
```

If the runtime adapter reports a different engine version, configuration, or semantic mode than the requested profile expected, the observation should record that difference as structured evidence.

## Reference-level explanation (precise rules)

InQL must define an interoperability semantic profile record. A profile record must include:

- profile identity
- profile schema version
- target class
- profile name or family
- source
- target version information when available
- target configuration fingerprint when available
- semantic dimensions
- evidence references
- confidence or completeness
- diagnostics

Target class must distinguish at least:

- inql_baseline
- client_protocol
- plan_ingress_frontend
- execution_engine
- adapter_binding
- sql_dialect
- interchange_consumer
- conformance_baseline

Semantic dimensions must be represented as structured records rather than free-form prose. Initial dimensions should include, where applicable:

- type system and implicit coercion
- numeric and decimal semantics
- temporal, timezone, and calendar semantics
- boolean, null, and NaN semantics
- string comparison, collation, and case sensitivity
- identifier resolution and catalog naming
- client session state and configuration semantics
- relation ordering and determinism
- aggregate and grouping edge semantics
- window frame and ordering semantics
- nested, variant, and semi-structured data semantics
- function and operator identity
- extension and fallback behavior
- plan-stage observability

A semantic dimension record must include dimension identity, lifecycle, declared behavior when known, source, evidence references, confidence, and diagnostics. A dimension may be exact, constrained, unknown, or not_applicable. Unknown dimensions must not be treated as matching InQL semantics.

InQL may define profile assessments that compare a plan or bundle with a profile. A profile assessment must include the plan target, profile identity, affected semantic targets, assessed dimensions, result state, evidence references, confidence, and diagnostics.

Profile assessment result state must distinguish at least:

- matched: InQL can determine that the plan's required semantics match the profile for the assessed dimension
- constrained: the profile can satisfy the dimension only under recorded restrictions
- mismatched: the profile does not satisfy the plan's required semantics for the dimension
- unknown: InQL cannot determine whether the profile satisfies the dimension
- not_applicable: the dimension does not apply to the plan or target profile

Adapter requirements and coverage records may cite profile records and profile assessments. If coverage depends on a profile, the coverage record must identify the profile. Coverage evaluated under one profile must not be reused under a different profile unless the evidence proves that the relevant semantic dimensions are equivalent.

Execution observations may include a requested profile and an observed profile. The requested profile is the semantic profile used during pre-execution inspection or coverage checks. The observed profile records runtime facts reported by the adapter, such as engine version, adapter version, semantic mode, or relevant configuration. A mismatch between requested and observed profiles must be diagnostic evidence. It must not silently rewrite the plan's authored semantics.

Profiles must not replace Prism semantic targets, lineage edges, adapter requirements, or execution observations. They provide context for evidence. They do not create fields, lineage, policy decisions, quality observations, or coverage states by themselves.

Serialized artifacts that include profile records must distinguish missing profile evidence from an empty or fully matching profile assessment.

## Design details

### Syntax

This RFC introduces no authoring syntax.

### Semantics

Semantic profiles are evidence contexts. They describe the target environment against which InQL evidence is checked, exported, or observed. They do not define InQL relational meaning.

Profiles may be authored, built into InQL, imported from artifacts, produced by adapters, or observed during execution. The source and lifecycle must be recorded so tools can distinguish a trusted built-in profile from an adapter-reported runtime observation or an imported profile.

### Interaction with other InQL surfaces

Prism remains the source of authored and rewritten relational meaning. Profile assessments consume Prism targets, lineage, schema flow, function registry facts, ingress coverage records, and adapter requirements. They must not infer semantic structure from backend plan strings or external client protocol node identifiers.

Plan ingress frontends may use profile evidence when decoding and analyzing external client plans. A Spark Connect frontend, for example, may use a client protocol profile to decide identifier resolution, function aliases, coercion behavior, and unsupported-feature diagnostics before Prism produces an analyzed plan.

Substrait lowering may carry or reference profile evidence, but Substrait must not be the only profile evidence store.

Function registry entries may contribute profile dimensions when function identity, determinism, null behavior, extension behavior, or backend availability affects compatibility.

Adapter coverage records should cite the profile used for evaluation when the answer depends on target semantics. Execution observations should report runtime profile facts when adapters can provide them.

Governed plan bundles may include profile records and profile assessments so downstream tools can understand which target environments were checked.

Export bridges may project profile evidence into external formats. Lossy exports must report dimensions that could not be represented.

### Compatibility / migration

Existing plans and adapters remain valid without profile evidence. Tools that require profile evidence must report missing profiles as unsupported or unknown evidence rather than assuming portability.

Profile schemas must be versioned from the start. Profile names that appear in serialized artifacts must be stable public vocabulary or explicitly marked as local/private.

## Alternatives considered

- **Use adapter support flags only.** Rejected because support depends on target semantics, engine version, configuration, and execution mode.
- **Use Substrait as the profile model.** Rejected because Substrait is an interchange boundary and does not capture every InQL evidence dimension.
- **Make one external engine profile normative.** Rejected because InQL needs to interoperate with multiple targets without importing one target's semantics as the language definition.
- **Rely only on conformance tests.** Rejected because tests are valuable evidence but do not replace structured profile records, coverage states, or diagnostics.
- **Leave profiles to downstream integrations.** Rejected because independent profile reconstruction would cause drift across adapters, CI, notebooks, agents, and governance exports.

## Drawbacks

- Profiles add another evidence concept that must stay distinct from requirements and coverage.
- Profile dimension vocabulary will require maintenance as InQL and target environments grow.
- Early profiles may contain many unknown dimensions, which can make reports feel conservative.
- Runtime-observed profiles can differ from requested profiles, requiring clear diagnostics.

## Layers affected

- **InQL specification** — semantic profile records, dimensions, and assessment states become part of the relational evidence vocabulary.
- **InQL library package** — inspection, coverage, bundle, and export APIs must be able to expose profile records when available.
- **Execution / interchange** — sessions and adapters may report requested and observed profile evidence without owning InQL semantics.
- **Documentation** — docs must explain profiles as evidence contexts, not as alternative semantic authorities.

## Unresolved questions

- Which semantic dimensions are mandatory in the first implementation?
- Should built-in InQL profiles live in core or in optional integration packages?
- How should profile records compare target configurations without leaking sensitive deployment details?
- Should conformance test results become profile evidence in this RFC or a later RFC?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
