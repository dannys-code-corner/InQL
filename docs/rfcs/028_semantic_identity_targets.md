# InQL RFC 028: Semantic identity and target model

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (core language model and layer boundaries)
  - InQL RFC 004 (execution context)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines the semantic identity and target model required by InQL's relational evidence program. It establishes stable, typed targets for plans, Prism nodes, plan ingress requests, client sessions, relation outputs, fields, scalar expressions, aggregate measures, window expressions, generator outputs, read roots, quality assertions, policy decisions, adapter requirements, coverage records, and execution attempts.

## Motivation

Lineage, metadata attachments, quality observations, policy decisions, execution observations, adapter coverage, and plan diffs all need something precise to attach to. If evidence attaches only to display names or backend plan fragments, it becomes fragile under aliases, rewrites, projections, and execution adapter differences. InQL needs a semantic target model before it can produce trustworthy evidence.

## Goals

- Define the required semantic target categories for local InQL evidence.
- Define which identities are plan-local, artifact-stable, and execution-local.
- Require deterministic identities within one plan, ingress, or profile snapshot where possible.
- Distinguish authored Prism targets from rewritten Prism targets and execution observations.
- Provide a common target vocabulary for child evidence RFCs.

## Non-Goals

- Defining a global registry of all project assets.
- Defining organization-wide identity, ownership, certification, or publication lifecycle.
- Defining physical backend plan identifiers as semantic identities.
- Defining the exact serialization format for every downstream artifact.

## Guide-level explanation (how authors think about it)

An InQL author should not usually construct semantic IDs manually. They appear when tools inspect a plan:

```incan
lineage = inspect_lineage(summary)
field = lineage.output_field("total_amount")

assert field.target.kind == "field"
assert field.target.plan_id == lineage.plan_id
```

The target lets tools attach evidence to the field even if the field was produced by an aggregate measure, renamed by a projection, lowered through Substrait, and executed by a backend adapter.

## Reference-level explanation (precise rules)

InQL must define semantic targets for at least the following categories:

- plan
- ingress request
- client session
- ingress node
- ingress expression
- analyzer binding
- authored Prism node
- rewritten Prism node
- relation output
- field
- scalar expression
- aggregate measure
- window expression
- generator output
- read root
- quality assertion
- quality observation
- policy decision
- adapter requirement
- coverage record
- semantic profile
- profile assessment
- execution attempt

A semantic target must identify its category, its containing plan, ingress request, client session, profile, or execution scope, and the minimum structural path needed to distinguish it from sibling targets in the same scope.

Plan, ingress request, ingress node, ingress expression, analyzer binding, authored node, rewritten node, relation output, field, scalar expression, aggregate measure, window expression, generator output, read-root, semantic profile, and profile assessment identities must be deterministic within one plan, ingress request, or profile snapshot where possible. Client sessions, execution attempts, quality observations, and runtime coverage records may be unique per frontend, session, or execution lifecycle.

InQL must distinguish authored targets from rewritten targets. If an optimizer rewrite removes, fuses, or replaces a target, the rewritten target must preserve an authored-origin relationship rather than reusing an authored identity for a different structure.

Field identities must not be based only on output display names. Renames, aliases, generated names, and duplicate field names must still produce unambiguous field targets.

Backend adapter identifiers may be attached as metadata or observations, but they must not replace InQL semantic targets.

External client protocol identifiers may be attached as ingress-origin evidence, but they must not replace InQL semantic targets.

A client session target identifies external client session state that can affect plan ingress analysis, such as current catalog or namespace, session configuration, temporary relation names, function registrations, profile selection, case-sensitivity mode, or dialect flags. Client session targets are not execution session identities unless an RFC or implementation explicitly binds the external client session to an InQL execution session.

## Design details

### Syntax

This RFC introduces no authoring syntax.

### Semantics

Semantic targets are evidence anchors. They do not by themselves define lineage, policy, quality, or execution behavior. Those contracts belong to child RFCs that reference this target model.

### Interaction with other InQL surfaces

Method chains and `query {}` blocks that express equivalent relational intent should produce equivalent target categories. They are not required to produce byte-identical IDs if their authored syntax differs, but the resulting semantic graph must preserve comparable targets for downstream tools.

### Compatibility / migration

Existing plans without semantic targets remain valid for execution. Tools that require relational evidence must report missing semantic identity as unsupported evidence rather than inferring identity from names alone.

## Alternatives considered

- **Use display names as identities.** Rejected because aliases, duplicate names, rewrites, and generated columns make display names insufficient.
- **Use backend plan node identifiers.** Rejected because backend adapters are not semantic owners and may change plans physically.
- **Require global durable IDs immediately.** Rejected because local plan evidence should work before global registry or publication semantics exist.

## Drawbacks

- Stable identity increases the complexity of planning and inspection.
- Some targets may need conservative IDs until Prism has richer authored-origin tracking.
- Deterministic local IDs can be mistaken for global identity unless docs are clear.

## Layers affected

- **InQL specification** — semantic target categories must become shared terminology for relational evidence RFCs.
- **InQL library package** — inspection and artifact APIs must expose targets rather than unstructured strings.
- **Execution / interchange** — Session, Substrait lowering, and adapters must preserve references to semantic targets where they emit related evidence.
- **Documentation** — docs must explain local plan identity versus global asset identity.

## Unresolved questions

- Which targets require artifact-stable IDs beyond one plan snapshot in the first release?
- Should target IDs be human-readable, opaque, or both?
- How should target identity behave when an optimizer rewrite duplicates one authored expression into multiple rewritten positions?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
