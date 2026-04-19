# InQL RFC 002: Apache Substrait integration

- **Status:** In Progress
- **Created:** 2026-03-23
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — naming, schema shapes, compilation model)
  - InQL RFC 001 (dataset types — `DataSet[T]` carriers and schema parameter)
- **Issue:** [InQL #3](https://github.com/dannys-code-corner/InQL/issues/3)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** -

## Summary

This RFC defines **Apache Substrait** as the **normative logical interchange** for InQL relational plans: which **`Rel` and expression** shapes implementations produce, how **read roots** remain **backend-agnostic** while **environment binding** (adapters, credentials, runner choice) stays **outside** InQL, and how **extensions** cover capabilities that lack a stable logical `Rel` in core Substrait. The `query {}` surface requires lowering to Substrait; this RFC owns the **cross-surface contract** so method-chain APIs (InQL RFC 001), `query {}` blocks, and optional pipe-forward do not diverge at emission time.

## Core model

1. A **checked** InQL relational tree **must** be expressible as a Substrait **`Plan`** whose executable root is a **`Rel`** tree, optionally a **DAG** via **`ReferenceRel`** when subplans are shared.
2. **Logical reads** are **`ReadRel`** (or extension leaf relations) carrying **names, virtual rows, or extension payloads** instead of host-specific connection strings or secrets in the normative interchange.
3. **Scalar and aggregate** computation uses Substrait **expressions** and **aggregate functions**; functions outside the pinned core set **must** use **registered extension URIs** documented with the compiler.
4. **North-star operator catalog**: InQL capabilities map to logical `Rel` kinds as specified in the [Substrait operator catalog reference][ref-operator-catalog]; **MVP** subsets are implementation choices but **must not** contradict this RFC for operators they expose.

## Motivation

Without a dedicated specification, Substrait lowering risks drifting between front-ends (`query {}`, APIs on `DataSet[T]`) and emitters, and risks smuggling execution concerns (storage URIs, credentials, engine choice) into the query IR. Substrait is the ecosystem's portable relational algebra serialization; InQL needs a single `Rel`-level contract, version pinning rules, and an explicit boundary between plan semantics and operational binding.

## Goals

- Require that conforming implementations **emit Substrait** for relational features they claim to support, using logical `Rel` nodes unless a documented extension applies.
- Publish a **versioned mapping catalog** from InQL plan concepts to Substrait logical relations and expression patterns, marking **core spec**, **extension**, or **documented expansion / gap**.
- Specify **read roots**: logical `ReadRel` shapes **in** InQL vs **adapter resolution** in the host execution environment.
- Require **documented pinning** of Substrait revision and of any bundled extension function sets shipped with the toolchain.
- List **known gaps** (unnest, pivot, advanced joins, streaming-specific semantics) without blocking InQL RFC 003.

## Non-Goals

- Defining orchestration, workflow, or adapter authoring syntax — out of scope; only binding boundaries relative to InQL plans are stated here.
- Mandating a default Substrait consumer (specific engine or library) — implementation detail; InQL RFC 004 names the reference backend.
- Physical Substrait relations as a normative InQL output — consumers **may** use them; InQL **may** emit them when documented as a non-portable or target-specific mode.
- ANSI SQL completeness — mapping is capability-based, not a SQL compliance checklist.

## v1 implementation profile (InQL code path)

The v1 implementation profile for this RFC is explicitly scoped to InQL package code (`.incn`) and is the contract for current delivery tracking.

- Core read/query `Rel` coverage is implemented through a thin proto-backed Substrait boundary in InQL package code.
- Optional mutation relations remain modeled but are not required to be executable in v1.
- Gap and extension semantics are represented as typed contracts in package code and conformance scenarios, rather than ad hoc string payloads.
- Richer planning semantics remain outside this profile when they logically belong to future `query {}` lowering or Prism.

This profile is reflected by:

- `src/substrait/schema.incn`
- `src/substrait/plan.incn`
- `src/substrait/conformance.incn`
- `src/substrait/conformance_catalog.incn`
- `src/substrait/conformance_validate.incn`
- `docs/language/reference/substrait/conformance.md`

### Current implementation status

| Area | Current status |
| --- | --- |
| Read roots, filter, cross, sort, fetch | Implemented at the proto-backed Substrait boundary |
| Join and set operation selection | Implemented at the boundary through explicit package-level enums/helpers |
| Reference rel | Implemented at the boundary for ordinal preservation only |
| Project and aggregate | Present as boundary-shape scaffolds; richer expression/grouping semantics remain deferred |
| Extension URI policy and explode gap encoding | Implemented through a registered package-level URI and documented gap policy |
| `query {}` lowering parity | Deferred to RFC 003 and Prism-backed lowering |
| Optional mutation profile | Deferred; not required for the v0.1 read/query analytical core |

## Guide-level explanation

Authors build `DataSet[T]` values (InQL RFC 001) using `query {}` or relational method chains. After typechecking, the relational work becomes a **Substrait plan**: mostly `FilterRel`, `ProjectRel`, `JoinRel`, `AggregateRel`, and so on, rooted in a `ReadRel` when new data enters the plan.

When a plan says "read this named relation" or "read this logical asset id," the plan carries the **logical** identity. The **execution context** resolves that identity to concrete storage, applies policy, and supplies credentials. That split keeps InQL portable and keeps governance-sensitive details out of the serialized plan's normative story.

## Reference-level explanation

### Normative interchange

- Implementations **must** be able to produce a Substrait `Plan` for every relational operator they expose that is claimed portable in documentation.
- Lowering semantics **must** be identical whether the surface is `query {}`, trait methods, or desugared pipe-forward, for the same checked tree.
- Implementations **may** additionally lower to InQL RFC 001 operations for execution; if both paths exist, they **must** match the Substrait semantics for those operators.

For the full capability → `Rel` mapping, profile classifications, and gap encoding requirements, see the [Substrait operator catalog reference][ref-operator-catalog].

Conformance scenarios **should** use stable scenario IDs and typed InQL model contracts as defined by the [Substrait conformance corpus reference][ref-conformance-corpus], so implementation and CI reporting can track portability status consistently across toolchains.

### Logical `Rel` alphabet

The following are the primary logical relations InQL targets. Exact protobuf message paths follow the pinned Substrait version selected by the toolchain for a given release.

| Substrait `Rel`                                                 | Role                                                                                          |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `ReadRel`                                                       | Scans: named table, local files, virtual rows, extension-defined sources                      |
| `FilterRel`                                                     | Row filter                                                                                    |
| `ProjectRel`                                                    | Derived columns; window expressions appear here per Substrait                                 |
| `JoinRel`                                                       | Joins including semi, anti, single, and mark variants; optional `post_join_filter`            |
| `CrossRel`                                                      | Cartesian product                                                                             |
| `AggregateRel`                                                  | Grouping sets, measures, `FILTER` on measures; distinct via keys-only aggregate               |
| `SortRel`                                                       | Sort                                                                                          |
| `FetchRel`                                                      | Limit / offset                                                                                |
| `SetRel`                                                        | Union, intersect, except variants                                                             |
| `ReferenceRel`                                                  | Shared subplans within a `Plan`                                                               |
| `WriteRel`                                                      | DML / CTAS (optional mutation profile)                                                        |
| `UpdateRel`                                                     | Table update without a full child `Rel` input (optional profile)                              |
| `DdlRel`                                                        | DDL (optional profile)                                                                        |
| `ExtensionSingleRel` / `ExtensionMultiRel` / `ExtensionLeafRel` | Extension escape hatches                                                                      |

### North-star catalog

InQL defines a north-star operator catalog that maps every InQL plan capability to a Substrait `Rel` or expression pattern, and classifies each capability as one of: **core** (maps to a standard logical `Rel` in the pinned revision), **extension** (requires a registered extension URI), **gap** (no stable logical `Rel` in core Substrait; encoding must be documented), or **optional-mutation** (not required for read/query analytical core).

The full, versioned catalog — including all profile tags, gap encoding requirements, and mutation-profile operators — lives in the [Substrait operator catalog reference][ref-operator-catalog]. Conforming implementations **must** follow the mappings listed there for any capability they claim portable.

### Read roots vs binding

- InQL **must** express new data entering a plan as logical reads: names, virtual values, or opaque extension table types that still serialize as Substrait `ReadRel` (or an extension leaf) **without** normative dependence on secret material in the plan text.
- The execution context **must** resolve logical reads to physical resources through its adapter and execution layer; that layer **must not** redefine relational semantics of the plan.
- Product SDKs **may** present a unified import surface; adapter-specific "open connection" APIs **should not** be specified as core InQL — they remain thin wrappers at most.

For the full `ReadRel` variant reference, the detailed execution context obligations, and the adapter boundary contract, see the [read-root and binding contract reference][ref-read-root].

### Extensions and function URIs

- Functions not in the pinned core Substrait bundle **must** use extension URIs registered in the compiler's public catalog for that toolchain version.
- `AdvancedExtension` fields **may** carry hints; normative semantics **must** be expressible without relying on hints.

For revision pin requirements, URI registration policy, bundle naming, compatibility conventions, and the release-note checklist for pin bumps, see the [revision and extension policy reference][ref-revision-policy].

### Optional mutation profile progress

- InQL **may** expose `WriteRel`, `DdlRel`, or `UpdateRel` for warehouse-style mutation. Absence of these in a given distribution **does not** make InQL incomplete for read-only analytical use.

The optional mutation profile operators, per-operator portability notes, and support expectations are listed in the [Substrait operator catalog reference][ref-operator-catalog].

### Reference documents

The following reference documents expand on the operational detail that is too long-lived and versioned to remain inside this RFC text:

| Document | What it covers |
| --- | --- |
| [Substrait operator catalog][ref-operator-catalog] | Full InQL capability → `Rel` mapping; profile tags; gap encoding rules; mutation profile operators |
| [Substrait revision and extension policy][ref-revision-policy] | Revision pin requirements; extension URI registration policy; compatibility conventions; release-note checklist |
| [Substrait read-root and binding contract][ref-read-root] | `ReadRel` variant reference; execution context obligations; adapter boundary |
| [Substrait conformance corpus][ref-conformance-corpus] | Canonical corpus structure, scenario metadata schema, profile taxonomy, and stable scenario ID conventions |

## Design details

### Interaction with Incan

- Field references and types **must** align with `model`-backed schemas (InQL RFC 001) and lower to Substrait types and field indices consistent with the emitted `NamedStruct`.

### Compatibility

- Additive mapping catalog changes **should** be the default; breaking emitter changes **must** ship with release notes and, when user-visible, an RFC amendment.

## Alternatives considered

- **SQL strings only** as interchange — rejected (weak structure for optimizers and cross-language tools).
- **Custom proprietary IR only** — rejected (ecosystem and long-term coupling).
- **Substrait optional** for "portable" builds — rejected; optional Substrait **may** exist only for explicitly non-portable or closed backends if documented as such.

## Drawbacks

- Substrait lags some front-end expressiveness; extensions and rewrites add maintenance.
- Dual lowering (InQL RFC 001 APIs + Substrait) increases test surface unless one path is canonical in practice.
- Producer / consumer version skew requires disciplined pinning and clear compatibility statements.

## Implementation architecture

Non-normative: toolchains **should** maintain golden Substrait plans or equivalent fixture tests for representative API-lowered trees, and later add `query {}` fixtures once that surface lowers through the same boundary.

## Layers affected

- **IR / lowering** to Substrait and extension registration.
- **Conformance / testing** artifacts for serialized plans.
- **Published operator catalog** and release notes for Substrait pin bumps.

## Implementation Plan

### Phase 1: Spec and operator catalog

- Lock down the versioned Substrait revision pinning policy in compiler documentation and release artifacts.
- Publish the normative operator catalog mapping InQL capabilities to Substrait `Rel` kinds, including gap annotations for unnest, pivot, and streaming semantics.
- Document extension URI registration conventions in the public toolchain catalog.

### Phase 2: IR lowering — core boundary

- Lower current package-authored `DataSet[T]` method-chain relational trees to Substrait `Plan` / `Rel` nodes covering: `ReadRel`, `FilterRel`, `ProjectRel`, `JoinRel`, `CrossRel`, `AggregateRel`, `SortRel`, `FetchRel`, `SetRel`, and `ReferenceRel`.
- Keep the current package layer thin: relation-shape ownership stays here, while richer planning semantics remain candidates for Prism.
- Align field references and types with `model`-backed schemas (RFC 001) where the current package code materially owns that boundary.

### Phase 3: Extensions and read binding

- Implement extension URI registration for functions outside the pinned core Substrait bundle.
- Implement logical `ReadRel` emission for named tables, virtual rows, and extension-defined sources without normative secret material in the plan text.
- Implement documented extension encoding for unnest / explode (gap handling).

### Phase 4: Optional mutation profile

- Implement `WriteRel`, `DdlRel`, and `UpdateRel` emission under the optional mutation profile.

### Phase 5: Conformance and testing

- Add golden Substrait plan fixtures for representative API-lowered trees.
- Add `query {}` fixtures later when that surface lowers through the same boundary.
- Verify fixture round-trips when Substrait revision is bumped.
- Update docs-site pages and operator catalog for public release.

## Progress Checklist

### Spec / design

- [ ] Substrait revision pinning policy documented in release artifacts and compiler docs.
- [ ] Normative operator catalog published (including gap annotations).
- [ ] Extension URI registration conventions documented.

### IR / lowering — core relations

- [ ] `ReadRel` emission: named table, virtual rows, extension sources.
- [ ] `FilterRel` emission.
- [ ] `ProjectRel` boundary scaffold emission.
- [ ] `JoinRel` emission (semi, anti, single, mark variants).
- [ ] `CrossRel` emission.
- [ ] `AggregateRel` boundary scaffold emission.
- [ ] `SortRel` emission.
- [ ] `FetchRel` emission (limit / offset).
- [ ] `SetRel` emission (union / intersect / except).
- [ ] `ReferenceRel` ordinal emission at the Substrait boundary.
- [ ] Lowering is identical across `query {}`, method chains, and desugared pipe-forward for the same checked tree once RFC 003 and Prism-backed lowering land.
- [ ] Field references align with RFC 001 `model`-backed schemas and `NamedStruct` indices.

### Extensions and read binding

- [ ] Extension URI registration for non-core functions wired in toolchain catalog.
- [ ] Logical `ReadRel` carries no normative secret material (binding left to execution context).
- [ ] Documented extension encoding for unnest / explode gap.

### Optional mutation profile

- [ ] `WriteRel` emission (optional profile).
- [ ] `DdlRel` emission (optional profile).
- [ ] `UpdateRel` emission (optional profile).

### Tests

- [ ] Golden Substrait plan fixtures for representative API-lowered (`DataSet[T]`) trees.
- [ ] Golden Substrait plan fixtures for representative `query {}` trees once that surface lowers through the same boundary.
- [ ] Fixture round-trip tests on Substrait revision bump.
- [ ] Tests confirm no secret material in emitted `ReadRel` plans.

### Docs

- [ ] Operator catalog page updated in docs-site.
- [ ] Release notes entry added.

## Design Decisions

- **Substrait revision pinning:** this RFC defines the pinning policy, not one timeless revision number. Each conforming InQL toolchain release **must** publish the exact Substrait revision it targets and any bundled extension sets in public release artifacts and compiler documentation.
- **Canonical unnest / explode encoding:** until core Substrait standardizes a portable unnest relation that InQL adopts, `EXPLODE`-style behavior **must** lower through a documented extension relation or another documented non-core encoding listed in the toolchain's public operator catalog. Implementations **must not** present ad hoc or undocumented encodings as portable core behavior.
- **Mutation relations:** `WriteRel`, `DdlRel`, and `UpdateRel` remain an optional mutation profile. They are not part of the minimum read/query analytical core required for InQL v0.1, and implementations **may** expose them only when the execution context and backend support them.
- **Correlated subqueries:** InQL v0.1 does not standardize a single correlated-subquery desugaring because correlated subquery surface syntax is not part of the minimum relational grammar. If a future RFC adds correlated subqueries, that RFC **must** define the lowering contract explicitly rather than relying on implicit emitter policy.

<!-- References -->

[ref-operator-catalog]: ../language/reference/substrait/operator_catalog.md
[ref-revision-policy]: ../language/reference/substrait/revision_and_extension_policy.md
[ref-read-root]: ../language/reference/substrait/read_root_binding_contract.md
[ref-conformance-corpus]: ../language/reference/substrait/conformance.md
