# InQL RFC 027: Relational evidence program

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (core language model and layer boundaries)
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 004 (execution context)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 029 (typed metadata attachments)
  - InQL RFC 030 (Prism lineage graph)
  - InQL RFC 031 (local inspection APIs and artifacts)
  - InQL RFC 032 (execution observations)
  - InQL RFC 033 (adapter requirements and coverage)
  - InQL RFC 034 (quality assertions and observations)
  - InQL RFC 035 (governed attributes and policy checkpoints)
  - InQL RFC 036 (governed plan bundle)
  - InQL RFC 037 (plan diff and blast-radius inputs)
  - InQL RFC 038 (evidence export bridges)
  - InQL RFC 040 (interoperability semantic profiles)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC is the umbrella tracking RFC for InQL's relational evidence program. The program defines the local, open semantic evidence contracts that make typed relational computation inspectable before execution and reviewable after execution: stable semantic targets, metadata attachments, Prism lineage, inspection artifacts, execution observations, adapter coverage, quality observations, governed attributes, plan bundles, plan diffs, export bridges, interoperability semantic profiles, and Prism plan ingress. This RFC is complete only when the child RFCs are implemented, rejected, or explicitly superseded by design decision.

## Core model

1. InQL owns typed relational evidence, not enterprise governance operations.
2. Prism is the semantic checkpoint for authored and rewritten relational meaning.
3. Substrait remains a portable interchange boundary, not the only semantic evidence store.
4. Session and adapter execution may attach observations to semantic targets, but they must not redefine authored relational meaning.
5. Local evidence must be useful without any hosted control plane, catalog service, approval workflow, or proprietary governance product.
6. Downstream systems may consume InQL evidence, but those systems are outside the InQL contract.
7. Interoperability profiles provide evidence context for target environments, not alternate semantic owners for InQL.

## Motivation

InQL already has the pieces of a stronger relational evidence layer: typed carriers, Prism planning, Substrait lowering, registry-backed expressions, aggregate/window/generator semantics, and a session boundary. What is missing is a coherent contract for the evidence that tools need to answer questions such as which source fields produced an output field, which plan rewrite changed a relation, which backend capability was required, which quality assertion failed, and which execution attempt produced a result.

Without this program, lineage, governance, quality, observability, and change-impact work will grow as disconnected features. Some tools will reconstruct meaning from Substrait, some from backend plans, some from session logs, and some from user-facing helper names. That would repeat the same failure InQL exists to avoid: typed relational meaning would be present during authoring, then weakened or reinterpreted at the next boundary.

## Goals

- Establish relational evidence as one coordinated InQL program.
- Define the child RFC set required for semantic identity, lineage, inspection, observations, coverage, quality, governed attributes, plan bundles, plan diffs, exports, and interoperability profiles.
- Keep the program open, local, and backend-neutral.
- Make Prism-authored relational meaning the source of local lineage and schema-flow evidence.
- Define target-environment profile evidence without making any external engine, dialect, or interchange format the semantic owner.
- Ensure execution observations and adapter coverage attach to semantic targets without redefining semantics.
- Allow higher-level governance, catalog, orchestration, audit, and approval systems to consume InQL evidence without becoming part of the InQL contract.

## Non-Goals

- Defining a hosted control plane, managed catalog, approval workflow, policy registry, stewardship UI, certification lifecycle, or proprietary blast-radius service.
- Defining organization-wide governance policy semantics.
- Defining pipeline orchestration, scheduling, retries, checkpointing, or cross-step lifecycle state.
- Making Substrait extension metadata the authoritative evidence store.
- Making a specific backend adapter the semantic owner of lineage, quality, policy, or coverage.
- Defining every external export mapping directly in this umbrella RFC.

## Guide-level explanation (how authors think about it)

Authors and tools should be able to inspect an InQL plan as structured evidence rather than formatted prose:

```incan
from pub::inql.inspect import inspect_lineage

summary = (
    orders
        .filter(col("status") == "paid")
        .group_by([col("customer_id")])
        .agg([sum(col("amount")).alias("total_amount")])
)

lineage = inspect_lineage(summary)
total = lineage.field("total_amount")
```

The exact API is defined in the child RFCs. The important user model is stable: InQL can explain typed relational computation locally, before a backend runs it and without requiring an external governance service.

## Reference-level explanation (precise rules)

The relational evidence program must consist of the following child RFCs unless this RFC is amended or superseded:

- InQL RFC 028 (semantic identity and target model)
- InQL RFC 029 (typed metadata attachments)
- InQL RFC 030 (Prism lineage graph)
- InQL RFC 031 (local inspection APIs and artifacts)
- InQL RFC 032 (execution observations)
- InQL RFC 033 (adapter requirements and coverage)
- InQL RFC 034 (quality assertions and observations)
- InQL RFC 035 (governed attributes and policy checkpoints)
- InQL RFC 036 (governed plan bundle)
- InQL RFC 037 (plan diff and blast-radius inputs)
- InQL RFC 038 (evidence export bridges)
- InQL RFC 040 (interoperability semantic profiles)
- InQL RFC 041 (Prism plan ingress and external client frontends)

This umbrella RFC must not be marked Implemented while any required child RFC remains Draft, Planned, In Progress, Blocked, or otherwise unresolved. A child RFC may be removed from the required completion set only by a design decision recorded in this RFC or by a superseding RFC.

Child RFCs must preserve the layer boundary established by this RFC. They may define local InQL evidence contracts and generic export shapes. They must not define proprietary product behavior, hosted storage behavior, managed approval semantics, or organization-wide policy lifecycle rules.

Relational evidence must derive from InQL semantic sources where possible. Prism-authored and Prism-rewritten plans are the authoritative source for local relational lineage. Session and backend adapter observations may report execution facts, diagnostics, and capability coverage, but they must not decide that an authored lineage edge exists or does not exist.

Evidence that affects correctness must not be encoded only as ignorable interchange metadata. If a downstream consumer must understand evidence for correctness, the plan must require a real supported capability, reject execution, or report unknown/uncovered coverage.

## Design details

### Syntax

This umbrella RFC introduces no new syntax. Child RFCs should prefer APIs and artifact contracts before proposing new authoring syntax.

### Semantics

This RFC is normative for program structure, lifecycle, and layer boundaries. Individual semantic contracts are normative only in the child RFC that owns the corresponding evidence family.

### Interaction with other InQL surfaces

Evidence must be independent of authoring surface. Equivalent method chains, `query {}` blocks, and future relational surfaces should produce equivalent semantic targets and lineage where they express equivalent relational intent.

External client frontends must follow the same rule. A Spark Connect, SQL, or other plan ingress frontend may preserve client-origin evidence, but Prism remains responsible for analyzed relational meaning.

The function catalog program remains relevant because function identity, aggregate measures, windows, generators, nested functions, format functions, approximate functions, and extensions all affect lineage and adapter coverage. The evidence program must consume function registry metadata rather than hardcoding function semantics in a separate evidence catalog.

### Compatibility / migration

This program should be additive. Existing plans may lack evidence artifacts until child RFCs are implemented. Serialized evidence artifacts must carry version metadata so consumers can distinguish unsupported evidence from empty evidence.

## Alternatives considered

- **One giant governance RFC.** Rejected because governance, lineage, quality, execution evidence, adapter coverage, and exports are too broad to specify responsibly in one normative document.
- **One RFC per artifact file.** Rejected because artifacts are downstream views of semantic contracts; the RFC boundary should be the concept, not the filename.
- **Use Substrait metadata as the evidence store.** Rejected because Substrait consumers may ignore extension metadata and because InQL needs richer local semantic targets than portable interchange can guarantee.
- **Let each downstream integration reconstruct evidence.** Rejected because it would make lineage and quality inconsistent across tools.

## Drawbacks

- The program creates several RFCs before implementation begins.
- Stable identity and artifact versioning add design surface that simple execution does not need.
- Some evidence will initially be conservative or unknown, which may feel less satisfying than overconfident lineage.
- The umbrella RFC may remain Draft or Planned for a long time while children land.

## Layers affected

- **InQL specification** — the RFC set must define a coherent relational evidence model across existing expression, planning, execution, and function-catalog RFCs.
- **InQL library package** — public inspection, quality, and artifact APIs must follow the child RFCs rather than growing as unrelated helpers.
- **Incan compiler** — compiler-facing support is affected only where child RFCs require typed metadata, stable symbols, or package inspection.
- **Execution / interchange** — Session, Substrait lowering, and adapters must attach execution evidence and capability coverage without owning relational semantics.
- **Documentation** — public docs must distinguish InQL local evidence contracts from downstream governance, catalog, and orchestration products.

## Unresolved questions

- Should this program include an explicit project brief or tracking issue before child RFCs move from Draft to Planned?
- Should any child RFC be split further before implementation begins?
- Should the umbrella completion set include future syntax RFCs if evidence-driven authoring syntax is later proposed?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
