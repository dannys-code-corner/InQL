# InQL RFC 002: Apache Substrait integration

- **Status:** Planned
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
4. **North-star operator catalog**: InQL capabilities map to logical `Rel` kinds as tabulated below; **MVP** subsets are implementation choices but **must not** contradict this RFC for operators they expose.

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

## Guide-level explanation

Authors build `DataSet[T]` values (InQL RFC 001) using `query {}` or relational method chains. After typechecking, the relational work becomes a **Substrait plan**: mostly `FilterRel`, `ProjectRel`, `JoinRel`, `AggregateRel`, and so on, rooted in a `ReadRel` when new data enters the plan.

When a plan says "read this named relation" or "read this logical asset id," the plan carries the **logical** identity. The **execution context** resolves that identity to concrete storage, applies policy, and supplies credentials. That split keeps InQL portable and keeps governance-sensitive details out of the serialized plan's normative story.

## Reference-level explanation

### Normative interchange

- Implementations **must** be able to produce a Substrait `Plan` for every relational operator they expose that is claimed portable in documentation.
- Lowering semantics **must** be identical whether the surface is `query {}`, trait methods, or desugared pipe-forward, for the same checked tree.
- Implementations **may** additionally lower to InQL RFC 001 operations for execution; if both paths exist, they **must** match the Substrait semantics for those operators.

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

### North-star catalog: InQL capabilities → Substrait

| InQL capability (conceptual)                                    | Substrait                                                                                     |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Logical table / registered name                                 | `ReadRel` + named table definition                                                            |
| File or object scan as plan input                               | `ReadRel` + local files (and format options in the pinned spec)                               |
| Literal or embedded rows                                        | `ReadRel` + virtual table                                                                     |
| Predicate pushdown into scan                                    | `ReadRel` filter fields and/or separate `FilterRel` — producer policy, **must** be documented |
| Row filter                                                      | `FilterRel`                                                                                   |
| Add or replace computed columns                                 | `ProjectRel`; drop / reorder via `RelCommon` emit (preferred) or equivalent                   |
| Joins                                                           | `JoinRel`                                                                                     |
| Cross join                                                      | `CrossRel`                                                                                    |
| Group by / aggregates                                           | `AggregateRel`                                                                                |
| Rollup / cube / grouping sets                                   | `AggregateRel` with multiple groupings                                                        |
| Distinct rows                                                   | `AggregateRel` with grouping keys and no measures                                             |
| Window / analytic functions                                     | `ProjectRel` with window expressions                                                          |
| Sort                                                            | `SortRel`                                                                                     |
| Limit / offset                                                  | `FetchRel`                                                                                    |
| Union / intersect / except                                      | `SetRel` with the appropriate set operation enum                                              |
| Reuse of an identical subplan                                   | `Plan` + `ReferenceRel`                                                                       |
| Unnest / explode                                                | **Gap**: extension rel, or documented expansion — **must** be pinned per implementation       |
| Pivot / unpivot                                                 | **Gap**: `ExtensionSingleRel` or documented rewrite to join + aggregate + project             |
| Asof / interval join                                            | **Gap** or non-equi join expression only where consumer contract explicitly allows            |
| Streaming time semantics (watermarks, session windows, state)   | **Outside core Substrait** unless expressed via named extensions or a separate execution IR   |

### Read roots vs binding

- InQL **must** express new data entering a plan as logical reads: names, virtual values, or opaque extension table types that still serialize as Substrait `ReadRel` (or an extension leaf) **without** normative dependence on secret material in the plan text.
- The execution context **must** resolve logical reads to physical resources through its adapter and execution layer; that layer **must not** redefine relational semantics of the plan.
- Product SDKs **may** present a unified import surface; adapter-specific "open connection" APIs **should not** be specified as core InQL — they remain thin wrappers at most.

### Extensions and function URIs

- Functions not in the pinned core Substrait bundle **must** use extension URIs registered in the compiler's public catalog for that toolchain version.
- `AdvancedExtension` fields **may** carry hints; normative semantics **must** be expressible without relying on hints.

### Optional mutation profile

- InQL **may** expose `WriteRel`, `DdlRel`, or `UpdateRel` for warehouse-style mutation. Absence of these in a given distribution **does not** make InQL incomplete for read-only analytical use.

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

Non-normative: toolchains **should** maintain golden Substrait plans or equivalent fixture tests for representative `query {}` and API-lowered trees, and **should** document tested consumers without implying exclusive support.

## Layers affected

- **IR / lowering** to Substrait and extension registration.
- **Conformance / testing** artifacts for serialized plans.
- **Published operator catalog** and release notes for Substrait pin bumps.

## Design Decisions

- **Substrait revision pinning:** this RFC defines the pinning policy, not one timeless revision number. Each conforming InQL toolchain release **must** publish the exact Substrait revision it targets and any bundled extension sets in public release artifacts and compiler documentation.
- **Canonical unnest / explode encoding:** until core Substrait standardizes a portable unnest relation that InQL adopts, `EXPLODE`-style behavior **must** lower through a documented extension relation or another documented non-core encoding listed in the toolchain's public operator catalog. Implementations **must not** present ad hoc or undocumented encodings as portable core behavior.
- **Mutation relations:** `WriteRel`, `DdlRel`, and `UpdateRel` remain an optional mutation profile. They are not part of the minimum read/query analytical core required for InQL v0.1, and implementations **may** expose them only when the execution context and backend support them.
- **Correlated subqueries:** InQL v0.1 does not standardize a single correlated-subquery desugaring because correlated subquery surface syntax is not part of the minimum relational grammar. If a future RFC adds correlated subqueries, that RFC **must** define the lowering contract explicitly rather than relying on implicit emitter policy.
