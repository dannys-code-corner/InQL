# InQL RFC 006: Promote unnest/explode to core Substrait lowering

- **Status:** Blocked
- **Created:** 2026-03-27
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 002 (Apache Substrait — normative gap classification for unnest; **prerequisite**)
  - InQL RFC 003 (`query {}` — `EXPLODE` clause; no surface change required)
  - InQL RFC 001 (dataset types — `explode` method on `DataSet[T]`; no surface change required)
- **Issue:** [InQL #14](https://github.com/dannys-code-corner/InQL/issues/14)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** -

> **Blocked** on upstream Apache Substrait standardizing a portable logical unnest/explode `Rel` in a revision InQL can pin to. See [Substrait operator catalog — Gap profiles: Unnest / explode][ref-operator-catalog].

## Summary

InQL RFC 002 classifies `EXPLODE`/unnest as a **gap** capability: no stable logical `Rel` exists in core Substrait, so implementations must lower through a registered extension relation with a declared URI. This RFC records the intent to promote that capability from `gap` to `core` — updating the operator catalog, retiring the extension encoding requirement, and updating Incan compiler lowering — once upstream Substrait ships a portable unnest `Rel` that InQL can adopt.

## Motivation

The current extension encoding for unnest adds extension URI maintenance burden to every conforming InQL toolchain release and limits plan portability to consumers that happen to support the same registered extension. The gap classification is not a permanent design choice; it reflects a gap in Substrait at the time of RFC 002. Once upstream closes that gap with a stable logical `Rel`, there is no reason for InQL to keep the extension path as the normative encoding. Reclassifying promptly gives authors core-portable `EXPLODE` semantics without requiring consumers to register or recognize InQL-specific URIs.

## Goals

- Reclassify the unnest/explode capability from `gap` to `core` in the [Substrait operator catalog reference][ref-operator-catalog].
- Retire the extension encoding requirement for unnest, with appropriate release-note entries per the [revision and extension policy reference][ref-revision-policy].
- Update Incan compiler lowering to emit the core `Rel` instead of the extension relation.

## Non-Goals

- Changing the InQL surface syntax for unnest — `EXPLODE` in `query {}` (InQL RFC 003) and `explode` on `DataSet[T]` (InQL RFC 001) remain unchanged.
- Defining the semantics of the new core `Rel` — that is an upstream Substrait concern; InQL aligns to whatever the pinned revision specifies.
- Keeping the extension encoding as an alternate path — once the core `Rel` is adopted, the extension path is retired.

## Guide-level explanation

From an author's perspective, nothing changes. `EXPLODE` in `query {}` and `.explode()` on a `DataSet[T]` work exactly as before. The only observable difference is in the serialized Substrait plan: before promotion, the emitted plan contains an `ExtensionSingleRel` or `ExtensionLeafRel` with an InQL-registered URI; after promotion, it contains the standard logical unnest `Rel` from the pinned Substrait revision. Consumers that previously required the InQL extension URI to execute unnest plans no longer do.

## Reference-level explanation

### Operator catalog update

When the unnest capability is promoted, `docs/language/reference/substrait/operator_catalog.md` **must** be updated:

- The entry for unnest/explode in the read/query analytical core table **must** change from `gap` to `core`.
- The Substrait column **must** reference the standard logical `Rel` name from the pinned revision.
- The unnest/explode section under Gap profiles **must** be removed or replaced with a note that the capability was promoted (pointing to the release notes for the relevant toolchain version).

### Extension encoding retirement

Per the [revision and extension policy reference][ref-revision-policy]:

- The extension URI registered for unnest/explode **must** be deprecated in the toolchain release that adopts the core `Rel`, with a deprecation diagnostic for plans referencing the old URI.
- The release notes entry **must** include the previous extension URI, the new core `Rel` name, and guidance for consumers to update plan consumption.

### Lowering update (Incan compiler)

- Incan compiler lowering for `EXPLODE` and `.explode()` **must** emit the standard logical unnest `Rel` instead of the extension relation.
- The emitted plan **must** not include the deprecated extension URI for unnest after the toolchain version that adopts the core `Rel`.

### Plan compatibility

Serialized Substrait plans containing the extension encoding for unnest will need to be re-emitted after a toolchain upgrade. This is a plan-level breaking change and **must** be documented in release notes per the revision and extension policy.

## Design details

### Interaction with other InQL surfaces

No surface changes. The `EXPLODE` clause in `query {}` and the `explode` method on `DataSet[T]` retain their existing semantics; only the Substrait emission changes.

### Compatibility / migration

- Breaking for serialized plans: existing plans with the extension encoding for unnest must be re-emitted. No author source code changes are required.
- Non-breaking for InQL source: `EXPLODE` and `.explode()` continue to compile and type-check identically.

## Alternatives considered

- **Keep the extension encoding permanently** — rejected; once a portable `Rel` exists, maintaining a proprietary extension degrades portability and adds unnecessary URI maintenance with no benefit.
- **Support both encodings simultaneously** — rejected; two emitted encodings for the same operation create consumer ambiguity. The extension path should be cleanly retired in the release that adopts the core `Rel`.

## Drawbacks

- Bumping the Substrait pin for this change is a plan-level breaking change, requiring coordinated consumer updates if any downstream tooling relies on the extension encoding.
- Timing depends entirely on upstream Substrait; InQL cannot control when a portable unnest `Rel` ships.

## Layers affected

- **InQL specification** — operator catalog reference updated; revision and extension policy retirement entry required.
- **Incan compiler** — lowering for `EXPLODE` / `explode` updated to emit the core `Rel` (work in the Incan repository).
- **Documentation** — release notes entry; operator catalog update.

## Unresolved questions

- Which exact Substrait revision introduces the portable unnest `Rel`? (Blocked on upstream; track `substrait-io/substrait`.)
- Are there semantic edge cases between the InQL extension encoding and the upstream core `Rel` that require a compatibility shim or a lowering-time rewrite?

<!-- References -->

[ref-operator-catalog]: ../language/reference/substrait/operator_catalog.md
[ref-revision-policy]: ../language/reference/substrait/revision_and_extension_policy.md
