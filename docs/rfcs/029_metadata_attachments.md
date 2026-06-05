# InQL RFC 029: Typed metadata attachments

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines typed metadata attachments for InQL semantic targets. Attachments provide a common way to associate lifecycle, source, visibility, typed payloads, provenance, and evidence references with plans, fields, expressions, requirements, observations, and other semantic targets without hardcoding every evidence family into one model.

## Motivation

Relational evidence needs more than lineage edges. A field may carry a redacted label, a source assertion, a planner diagnostic, a session observation, an adapter capability result, or an exported catalog reference. Without a typed attachment model, each feature will invent its own string map and lifecycle rules, making evidence inconsistent and difficult to export.

## Goals

- Define a common attachment shape for semantic targets.
- Require lifecycle, source, visibility, and typed payload metadata.
- Preserve provenance and evidence references.
- Support sensitive and redacted metadata without forcing it into portable plans.
- Let child RFCs define specialized payload schemas while sharing one attachment contract.

## Non-Goals

- Defining every possible metadata key.
- Defining business glossary, certification, stewardship, or ownership lifecycle.
- Defining a hosted metadata store.
- Making arbitrary untyped string maps the public evidence model.

## Guide-level explanation (how authors think about it)

Most authors should see attachments through inspection results:

```incan
plan = inspect_plan(summary)
field = plan.output_field("customer_id")

for attachment in field.attachments():
    print(attachment.namespace, attachment.key, attachment.visibility)
```

The attachment shape lets tools distinguish a user-authored label, a planner-derived fact, an adapter-reported observation, and a redacted sensitive value.

## Reference-level explanation (precise rules)

An InQL metadata attachment must include:

- target semantic identity
- namespace
- key
- typed payload
- lifecycle
- source
- visibility
- evidence references

Attachment lifecycle must distinguish at least authored, planned, analyzed, rewritten, lowered, bound, executed, exported, and imported states.

Attachment source must distinguish at least InQL, user, Prism, Session, adapter, function registry, quality engine, policy engine, external catalog, and imported artifact.

Attachment visibility must distinguish at least public, internal, sensitive, and redacted. Sensitive attachments must not be emitted into portable artifacts by default. Redacted attachments may preserve the existence, target, and reason code of a hidden fact without exposing the payload.

Typed payloads must be schema-versioned when serialized. Consumers must be able to reject unknown payload schemas without treating the attachment as absent.

Attachments must not override semantic structure. A metadata attachment may describe a field, but it must not create a field identity or lineage edge by itself. Structural evidence belongs in the semantic target and lineage models.

## Design details

### Syntax

This RFC introduces no syntax. Future helper APIs may expose attachments, but authoring syntax is not required.

### Semantics

Attachments are evidence records, not semantic authority by default. A child RFC may define an authoritative attachment kind only when the authority, lifecycle, and conflict behavior are explicit.

### Interaction with other InQL surfaces

Function registry metadata may produce attachments when functions affect lineage, adapter requirements, null behavior, determinism, or extension support. Those attachments must derive from registry facts rather than duplicating function names or signatures in a separate evidence catalog.

### Compatibility / migration

Existing APIs may continue returning simple inspection data. New evidence APIs should expose the attachment model so clients can migrate away from ad hoc metadata maps.

## Alternatives considered

- **One model per evidence family with no shared attachment layer.** Rejected because lifecycle, visibility, provenance, and evidence references would drift.
- **Arbitrary string-key maps.** Rejected because untyped payloads are hard to validate and unsafe to export.
- **Put all metadata into Substrait extensions.** Rejected because Substrait extension metadata is not a reliable authoritative store for local evidence.

## Drawbacks

- Attachments introduce generic machinery before all payload families exist.
- Visibility rules require discipline from export adapters.
- Poorly scoped namespaces could still become clutter if review is weak.

## Layers affected

- **InQL specification** — metadata attachments must become the shared extension point for evidence families.
- **InQL library package** — inspection and artifact APIs must preserve typed attachment payloads and visibility.
- **Execution / interchange** — lowering and adapters may carry attachment references but must not leak sensitive payloads by default.
- **Documentation** — docs must show which attachment namespaces are stable public contracts.

## Unresolved questions

- Which attachment namespaces should be reserved by InQL core?
- Should users be able to author arbitrary attachments directly, or only through typed helper APIs?
- What is the first serialized payload schema format for attachments?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
