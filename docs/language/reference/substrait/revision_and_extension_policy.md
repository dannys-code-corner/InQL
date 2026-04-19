# Substrait revision and extension policy (Reference)

This page is the **operational policy reference** for InQL's Substrait revision pinning and extension function management. The normative rules — that pinning is required and that functions outside the core bundle must use registered extension URIs — live in [InQL RFC 002][rfc-002]. This page provides the operational detail: what must be declared in a release, how extension URIs are registered, what constitutes a breaking vs additive change, and the checklist contributors follow when bumping the pinned revision.

## Revision pinning

### Requirements

Each conforming InQL toolchain release **must** declare:

- The exact Substrait **revision** it targets (commit hash or tagged release, depending on the Substrait project's versioning model at time of the InQL release).
- Any **bundled extension function sets** (YAML or equivalent) shipped alongside the toolchain. Each set must identify its URI prefix, the Substrait revision it was authored against, and the set of functions it covers.

This information **must** appear in:

1. The toolchain's **public release artifacts** — for example, the compiler binary's `--version` output or an accompanying manifest file (`substrait-pin.json` or equivalent).
2. The toolchain's **compiler documentation** — the release notes for that version and, where applicable, the published [operator catalog][ref-operator-catalog] for that toolchain version.

### Compatibility matrix

| Change type                                                                           | Required action                                                                                                                                |
| ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Patch-level Substrait revision bump (clarifications or fixes only; no schema changes) | Document in release notes; no catalog or RFC amendment required                                                                                |
| Minor Substrait revision bump (additive schema changes)                               | Document in release notes; update operator catalog if new `Rel` nodes are adopted                                                              |
| Breaking Substrait schema change                                                      | Major toolchain version bump; RFC 002 amendment required; migration guide required                                                             |
| New extension URI registered                                                          | Document in operator catalog for the toolchain version; no RFC amendment required unless the new URI changes normative lowering behavior       |
| Extension URI retired or renamed                                                      | Document in release notes with a deprecation warning; implementations must emit a deprecation diagnostic for plans referencing the retired URI |

### Pin bump: release-note checklist

When bumping the pinned Substrait revision, the release notes entry **must** include:

- [ ] Previous revision reference (commit hash or tag).
- [ ] New revision reference (commit hash or tag).
- [ ] Summary of schema changes that affect emitted plans.
- [ ] Any capability reclassifications — for example, a gap capability promoted to core, an extension reclassified, or (for regressions) a core capability demoted.
- [ ] Required consumer update guidance if consumer APIs or protobuf schemas changed in a consumer-breaking way.

## Extension URI registration

### Policy

Functions not in the pinned core Substrait bundle **must** use extension URIs that are:

1. **Registered** in the toolchain's public catalog before the function is exposed in a stable release. Pre-release builds may use provisional URIs clearly labeled as unstable.
2. **Stable across patch releases**: once a URI is published in a stable release, it **must not** change meaning or be silently dropped without a deprecation cycle.
3. **Documented**: the operator catalog entry for the URI **must** specify the function name, argument types, return type, and semantic contract (including any edge-case behavior that differs from SQL or Substrait conventions).

### URI structure

InQL toolchain extension URIs **should** follow the pattern:

```text
https://inql.io/extensions/<version>/<namespace>.yaml
```

Where:

- `<version>` is the toolchain version that introduced the extension (e.g. `v0.1`).
- `<namespace>` groups related functions (e.g. `aggregate`, `string`, `temporal`, `unnest`).

The exact URI scheme is part of the toolchain release process and **must** be documented alongside the release. The current InQL package code uses the `inql.io` base for registered extension examples and treats pre-1.0 entries as provisional until the wider release process is finalized.

### `AdvancedExtension` fields

`AdvancedExtension` fields **may** carry optimization hints or metadata alongside a plan:

- Normative plan semantics **must** be expressible without relying on `AdvancedExtension` fields. A plan that requires `AdvancedExtension` to execute correctly is non-conforming.
- Consumers **must** be able to execute the plan (possibly without the optimization benefit) in the absence of `AdvancedExtension` support.
- If an `AdvancedExtension` field changes a plan's observable output (rather than its performance), it is no longer a hint — it must be modeled as a registered extension.

## Compatibility policy

### Additive changes (default)

Mapping catalog additions — new InQL capabilities mapped to Substrait, new extension URIs registered, new optional capabilities documented — are **additive changes**. They:

- Do not require an RFC amendment.
- **Must** appear in release notes with the capability name and profile tag.
- **Must not** change the semantics of any existing mapping.

### Breaking changes

A change is **breaking** when it:

- Removes or renames an extension URI that was published in a stable release.
- Changes the emitted `Rel` shape for an existing capability in a way that alters consumer behavior or plan serialization.
- Reclassifies a capability from core to gap or non-portable (a regression).

Breaking emitter changes **must**:

1. Ship with release notes that explicitly identify the breaking change and the affected capability.
2. Include an RFC 002 amendment documenting the updated mapping when the change is user-visible (for example, plans that compiled or executed before no longer do so correctly).
3. Where possible, include a migration guide or automated migration tool.

Reclassification from gap → core or extension → core is **not** breaking (it broadens portability). It **must** still be documented in release notes and the operator catalog.

<!-- References -->

[rfc-002]: ../../rfcs/002_apache_substrait_integration.md
[ref-operator-catalog]: ./operator_catalog.md
