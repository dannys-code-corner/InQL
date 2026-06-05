# InQL RFC 033: Adapter requirements and coverage

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 004 (execution context)
  - InQL RFC 024 (function extension policy)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 032 (execution observations)
  - InQL RFC 040 (interoperability semantic profiles)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines adapter requirements and coverage states for InQL. Requirements describe backend capabilities needed by a plan or evidence contract, while coverage states report whether a specific adapter can satisfy those requirements under the relevant binding and semantic profile. Unknown coverage is not enforcement.

## Motivation

Backend neutrality only works when backend limits are visible. A plan may require extension functions, precise decimal behavior, variant semantics, lineage preservation, audit emission, masking, aggregation thresholding, or other capabilities. If InQL hides adapter uncertainty, downstream systems may assume a guarantee that the selected backend cannot provide.

## Goals

- Define adapter requirements as semantic evidence targets.
- Define coverage states: covered, partially_covered, uncovered, and unknown.
- Require coverage records to name the adapter, semantic profile when relevant, and evidence.
- Keep backend inability distinct from unsupported InQL semantics.
- Make capability uncertainty explicit before execution when possible.

## Non-Goals

- Defining every possible backend capability.
- Defining physical execution strategies.
- Making any one adapter the semantic owner of InQL behavior.
- Defining organization-wide enforcement policy.

## Guide-level explanation (how authors think about it)

An inspection can reveal backend requirements:

```incan
inspection = inspect_plan(summary)

for requirement in inspection.adapter_requirements():
    print(requirement.capability, requirement.guarantee_level)
```

A session can then report whether the selected adapter covers them:

```incan
coverage = session.check_coverage(summary)
```

If coverage is unknown for a requirement whose guarantee level is required, tools should not present that as enforced behavior.

## Reference-level explanation (precise rules)

An adapter requirement must include requirement identity, target, capability, guarantee level, reason references, and optional diagnostic text.

Capability names must be stable public vocabulary when they appear in serialized artifacts. Initial capability families should include extension_function, variant_semantics, decimal_semantics, null_semantics, lineage_preservation, audit_emission, row_filter, column_mask, aggregate_threshold, region_binding, and ordered_execution where applicable.

Guarantee level must distinguish required, preferred, and optional requirements.

A coverage record must include requirement identity, adapter identity, adapter version when available, semantic profile identity when the evaluation depends on a profile, coverage state, evidence references, and diagnostic text.

Coverage state must distinguish:

- covered: the adapter can satisfy the requirement under the current binding
- partially_covered: the adapter can satisfy part of the requirement or only under restrictions
- uncovered: the adapter cannot satisfy the requirement
- unknown: InQL cannot determine whether the adapter can satisfy the requirement

Unknown coverage must not be treated as covered. If a requirement whose guarantee level is required is unknown or uncovered, execution must reject, route, rewrite, require approval, or report non-enforcing behavior according to the higher-level policy using the coverage record.

Backend inability must be reported as adapter coverage or execution failure. It must not be encoded as a normal Substrait-level state for core InQL semantics.

Ingress coverage belongs to plan ingress frontends. It must not be reported as backend adapter coverage unless the same feature also creates an execution requirement for the selected adapter.

## Design details

### Syntax

This RFC introduces no syntax.

### Semantics

Adapter requirements are evidence about what a plan needs. Coverage records are evidence about what a selected adapter can provide.

### Interaction with other InQL surfaces

Function registry entries, semi-structured functions, extensions, quality assertions, governed attribute constraints, and semantic profile assessments may all create adapter requirements. Execution observations may reference coverage records.

### Compatibility / migration

Existing adapters may initially report unknown coverage for capabilities they do not declare. Consumers must distinguish unknown from covered.

## Alternatives considered

- **Fail only at backend runtime.** Rejected because users need pre-execution visibility when possible.
- **Treat unsupported backend features as unsupported InQL semantics.** Rejected because backend inability is not the same as invalid InQL.
- **Use boolean supports flags.** Rejected because partial and unknown coverage are important operational states.

## Drawbacks

- Capability vocabulary must be maintained.
- Adapters need more metadata.
- Early coverage results may be conservative.

## Layers affected

- **InQL specification** — adapter requirement and coverage vocabulary becomes normative.
- **InQL library package** — inspection and session APIs must expose requirements and coverage.
- **Execution / interchange** — adapters must report capability evidence honestly.
- **Documentation** — docs must explain that unknown coverage is not enforcement.

## Unresolved questions

- Which capability families are mandatory in the first implementation?
- Should coverage checks be available without binding physical sources?
- How should adapter-specific diagnostics be normalized?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
