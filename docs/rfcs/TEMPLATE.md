# InQL RFC template

> Use this template for RFCs in [`docs/rfcs/`](README.md). Keep each RFC focused: one coherent proposal, with clear motivation, precise semantics, and where work lands (**this package**, **Incan compiler**, **execution layer**). For workflow and tips, see [Writing InQL RFCs](../contributing/writing_rfcs.md).

## Title

InQL RFC NNN: \<short descriptive title\>

<!-- Status descriptions:

- **Draft:** Initial proposal; needs review.
- **Planned:** Design settled; ready for implementation.
- **In Progress:** Implementation underway.
- **Blocked:** Blocked by another RFC or issue.
- **Deferred:** Deferred to a later time.
- **Implemented:** Complete and shipped (optionally move file under `closed/implemented/` when you adopt that layout).
- **Superseded by InQL RFC NNN:** Replaced by a newer RFC.
- **Rejected:** Will not be pursued.
 -->

- **Status:** Draft
- **Created:** \<YYYY-MM-DD\>
- **Author(s):** \<name (@handle)\>
- **Related:** —  <!-- Upstream InQL RFCs this builds on or must stay consistent with; `—` if none -->
- **Issue:** \<link to GitHub issue\>
- **RFC PR:** \<link to PR adding or updating this file\>
- **Written against:** \<e.g. Incan v0.2\>  <!-- Incan language/toolchain baseline the RFC assumes; not a ship date -->
- **Shipped in:** —  <!-- First **InQL package** release that includes the change. Leave `—` in Draft/Planned. Do not set speculatively. -->

## Summary

One paragraph describing what this RFC proposes.

## Motivation

Explain the problem and why it matters:

- What is painful or confusing today?
- Who benefits?
- Why is this better than the status quo?

## Goals

- …

## Non-Goals

- …

## Guide-level explanation (how authors think about it)

Explain the feature the way an InQL author would reason about it. Prefer concrete examples.

```incan
# Example
```

## Reference-level explanation (precise rules)

Define exact semantics, typing rules, resolution rules, lowering contracts, and edge cases.

- Grammar or surface shape (if applicable)
- Typechecking / name resolution
- Plan shape or interchange (e.g. Substrait), if applicable
- Errors and diagnostics

## Design details

### Syntax

New or changed syntax, if any.

### Semantics

Precise behavior.

### Interaction with other InQL surfaces

How this composes with:

- Other authoring surfaces (`query {}`, `DataSet[T]` APIs, optional pipe-forward) — stay consistent with the foundational language spec where it applies.
- Incan `model` types and ordinary lexical scope.
- Logical plans, execution context, and I/O boundaries as defined in sibling RFCs.

### Compatibility / migration

- Is this breaking for authors or for serialized plans?
- If yes, migration strategy and examples.

## Alternatives considered

Plausible alternatives and why they are worse (or were rejected).

## Drawbacks

Complexity, performance, mental model, or maintenance cost.

## Layers affected

Describe which parts of the system this RFC impacts, in **normative** language (`must`, `must not`, `should`). Do not turn this into a task list or internal file manifest — that belongs in implementation issues and PRs.

- **InQL specification** — coherence with other `docs/rfcs/` documents.
- **InQL library package** — public `.incn` API, tests, manifests.
- **Incan compiler** — parsing, checking, lowering, or diagnostics for InQL constructs (work may live in the Incan repository).
- **Execution / interchange** — Substrait consumers, session, adapters, runtime (often specified relative to an execution context RFC).
- **Documentation** — README, architecture notes, contributor docs.

Omit bullets that do not apply.

## Unresolved questions

Open questions to resolve before implementation (or before promoting **Draft → Planned**).

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
