# InQL architecture

This document describes the architectural model of **InQL**. It is scoped to the InQL repository and its relationship to the Incan compiler, not to product orchestration or engine-specific operational concerns.

## What InQL is

InQL is two things that evolve together:

1. **A specification** — Normative design under [docs/rfcs/][inql-rfcs]: naming and core semantics, dataset carriers,  Substrait emission, query authoring, the execution boundary, and the internal planning substrate.
2. **An Incan library package** — `.incn` modules built with `incan build --lib`, consumed by Incan programs as a typed relational package.

The Incan compiler remains responsible for parsing, typechecking, lowering, and Rust/code generation. The InQL repo holds the author-facing package and the RFCs that define what that package is supposed to mean.

## Architectural model

InQL is organized around three layers:

- **Prism internally** — the immutable planning and optimization engine
- **Substrait at the boundary** — the normative emitted logical interchange contract
- **Session for execution** — the execution/binding layer that consumes plans but does not define them

That gives each major concept one job:

- **Prism** thinks about the plan
- **Substrait** communicates the plan
- **Session** executes the plan

This separation is important because it keeps internal planning concerns, portable interchange semantics, and runtime execution concerns from collapsing into one another.

## Conceptual pipeline

InQL is intended to follow this shape:

```text
Incan models / model-derived schema
        │
        ▼
  DataSet[T] carriers
        │
        ├──► method chains
        ├──► query { } blocks
        └──► future pipe-forward / other authoring surfaces
                 │
                 ▼
        Prism logical planning substrate
                 │
                 ├──► authored plan state
                 ├──► lineage-preserving optimization
                 └──► optimized logical view
                          │
                          ▼
                Substrait Plan / Rel emission
                          │
                          ▼
                  Session / backend execution
```

The core rule is:

- authoring surfaces build or manipulate Prism-managed logical work
- Prism prepares that work for boundary emission
- RFC 002 owns the Substrait contract
- RFC 004 owns execution and binding

## Layer responsibilities

### Carriers

The author-facing carrier family is rooted in `DataSet[T]` and currently includes `LazyFrame[T]`, `DataFrame[T]`, and `DataStream[T]`.

Carriers are expected to be:

- typed by model-derived schema information
- immutable from the author’s point of view
- cheap to branch
- execution-neutral on their own

They should be understood as **experiences over shared Prism-managed planning state**, not as independent semantic systems.

### Prism

Per [RFC 007][rfc-007], Prism is InQL’s internal logical planning and optimization engine.

Prism is responsible for:

- persistent logical plan storage
- cheap branching through structural sharing
- lineage preservation
- logical rewrites and optimization before boundary emission or execution

Prism is **not** the normative interchange format and **not** the execution engine.

### Substrait

Per [RFC 002][rfc-002], Apache Substrait is the normative logical interchange boundary for InQL.

That means:

- portable relational work must be expressible as Substrait `Plan` / `Rel`
- logical reads remain logical at the boundary
- extension and gap handling are documented at the Substrait boundary
- internal planning freedom is allowed, but emitted plans must follow RFC 002

Today, the package’s RFC 002-facing code lives primarily in:

- [plan.incn](../src/substrait/plan.incn)
- [conformance.incn](../src/substrait/conformance.incn)
- [schema.incn](../src/substrait/schema.incn)

### Session

Per [RFC 004][rfc-004], `Session` / `SessionContext` own binding and execution.

Session is responsible for:

- resolving logical reads to physical resources
- applying backend-specific execution behavior
- collecting or materializing results
- writing to sinks where appropriate

Session is intentionally outside RFC 002’s normative emitted contract. It consumes plans; it does not define plan semantics.

## Current implementation

The repository currently includes:

- author-facing carrier types exist in [mod.incn](../src/dataset/mod.incn)
- canonical relational operator helpers exist in [ops.incn](../src/dataset/ops.incn)
- RFC 002 emits **real proto-backed Substrait plans**
- conformance scenarios are represented as typed package code in [conformance.incn](../src/substrait/conformance.incn)
- Prism is specified as the internal planning substrate, while parts of its full implementation remain ahead of the current package code

This means the package has a concrete Substrait boundary and conformance layer, while some internal planning mechanics remain transitional.

## Repository layout

| Path                              | Role                                              |
| --------------------------------- | ------------------------------------------------- |
| `incan.toml`                      | Package metadata and Rust dependency declarations |
| `src/lib.incn`                    | Public package exports                            |
| `src/dataset/mod.incn`            | Carrier types and trait surface                   |
| `src/dataset/ops.incn`            | Canonical relational operator helpers             |
| `src/substrait/plan.incn`         | RFC 002 proto-backed Substrait emission helpers   |
| `src/substrait/conformance.incn`  | Typed conformance corpus and validation helpers   |
| `src/substrait/schema.incn`       | Model/schema to Substrait type bridging           |
| `tests/`                          | Package tests run through `incan test`            |
| `docs/rfcs/`                      | Normative RFC series                              |
| `docs/architecture.md`            | This overview                                     |

Normative behavior lives in the RFC series first. If code and RFCs disagree, treat that as a bug or transition state to resolve explicitly.

## Repository vs compiler

The InQL repository and the Incan compiler have different responsibilities.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  InQL repo                                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  RFCs, package modules, tests, architecture, conformance corpus             │
│  Defines the relational package surface and its normative contracts         │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ implemented through
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Incan compiler                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Parsing, typechecking, lowering, Rust emission, LSP, test runner, builds   │
│  Makes InQL package code executable and eventually supports new surfaces    │
└─────────────────────────────────────────────────────────────────────────────┘
```

That distinction matters because some InQL architecture is specified before the compiler fully supports every intended implementation path. Prism is a good example: the planning boundary is specified even where current compiler and tooling constraints still force temporary implementation compromises.

## Build and test

From the repo root, with `incan` on `PATH`:

```text
incan build --lib
incan test
```

In practice:

- `incan build --lib` parses, typechecks, lowers, and emits a Rust crate for the InQL library
- `incan test` discovers and runs tests under `tests/`

CI builds `incan` first, then runs the InQL package checks against that compiler.

## Reading order

If you want the clearest architecture story, read in this order:

1. [RFC 001][rfc-001] — carrier semantics
2. [RFC 002][rfc-002] — Substrait boundary
3. [RFC 004][rfc-004] — execution boundary
4. [RFC 007][rfc-007] — Prism internal planning substrate

That sequence mirrors the intended separation between authoring surface, interchange, execution, and internal planning.

## Where to read more

| Topic                       | Location                                      |
| --------------------------- | --------------------------------------------- |
| InQL RFC index              | [docs/rfcs/README.md][inql-rfcs]              |
| Prism planning engine       | [RFC 007][rfc-007]                            |
| Substrait integration       | [RFC 002][rfc-002]                            |
| Execution context           | [RFC 004][rfc-004]                            |
| Incan compiler architecture | [Incan architecture docs][incan-architecture] |
| Contributing                | [CONTRIBUTING.md][inql-contributing]          |

[incan-architecture]: https://github.com/dannys-code-corner/incan/blob/main/workspaces/docs-site/docs/contributing/explanation/architecture.md
[inql-rfcs]: rfcs/README.md
[inql-contributing]: ../CONTRIBUTING.md
[rfc-001]: rfcs/001_inql_dataset.md
[rfc-002]: rfcs/002_apache_substrait_integration.md
[rfc-004]: rfcs/004_inql_execution_context.md
[rfc-007]: rfcs/007_prism_planning_engine.md
