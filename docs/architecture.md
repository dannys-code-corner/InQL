# InQL architecture

This document describes the architectural model of **InQL**. It is scoped to the InQL repository and its relationship to the Incan compiler, not to product orchestration or engine-specific operational concerns.

## What InQL is

InQL is two things that evolve together:

1. **A specification** under [docs/rfcs/][inql-rfcs]: naming and core semantics, dataset carriers, Substrait emission, query authoring, the execution boundary, and the internal planning substrate.
2. **An Incan library package**: `.incn` modules built with `incan build --lib`, consumed by Incan programs as a typed relational package.

The Incan compiler remains responsible for parsing, typechecking, lowering, and Rust/code generation. The InQL repo holds the author-facing package, its documentation, and the RFCs that define what that package is supposed to mean.

## Architectural model

InQL is organized around three layers:

- **Prism internally** — the immutable planning and optimization engine over persistent authored plan state and derived optimized views
- **Substrait at the boundary** — the normative emitted logical interchange contract
- **Session for execution** — the execution and binding layer that consumes plans but does not define them

That gives each major concept one job:

- **Prism** thinks about the plan
- **Substrait** communicates the plan
- **Session** executes the plan

Per [RFC 008][rfc-008], that split is intentionally minimal at this stage:

- Prism owns logical plan shaping before execution
- Session owns backend binding, physical planning, runtime metrics, and adaptive behavior
- richer statistics transport and optimizer mechanics remain follow-on work

This separation keeps internal planning concerns, portable interchange semantics, and runtime execution concerns from collapsing into one another.

## Conceptual pipeline

InQL follows this shape:

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
                 └──► derived optimized views
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

The author-facing carrier family is rooted in `DataSet[T]` and includes `LazyFrame[T]`, `DataFrame[T]`, and `DataStream[T]`.

Carriers are expected to be:

- typed by model-derived schema information
- immutable from the author’s point of view
- cheap to branch
- execution-neutral on their own

They should be understood as experiences over planning state, not as independent semantic systems.

For current package behavior, see [Dataset carriers (Reference)][dataset-ref] and [Dataset carriers (Explanation)][dataset-expl].

### Prism

Per [RFC 007][rfc-007], Prism is InQL’s internal logical planning and optimization engine.

Prism is responsible for:

- persistent authored logical plan storage
- cheap branching through structural sharing
- lineage preservation
- logical rewrites and derived optimized views before boundary emission or execution

Prism is not the normative interchange format and not the execution engine.

### Substrait

Per [RFC 002][rfc-002], Apache Substrait is the normative logical interchange boundary for InQL.

That means:

- portable relational work must be expressible as Substrait `Plan` / `Rel`
- logical reads remain logical at the boundary
- extension and gap handling are documented at the Substrait boundary
- internal planning freedom is allowed, but emitted plans must follow RFC 002

Substrait-facing package code lives primarily under [substrait/](../src/substrait/). The current implementation is intentionally split into focused modules for relation building, plan assembly, schema registry, extension bookkeeping, expression lowering, and inspection. For current boundary docs, start with [Substrait read-root and binding contract][substrait-read-root].

### Session

Per [RFC 004][rfc-004], `Session` owns binding and execution.

Session is responsible for:

- resolving logical reads to physical resources
- applying backend-specific execution behavior
- owning physical planning and runtime adaptation policy
- collecting or materializing results
- writing to sinks where appropriate

The public Session surface is sync-first for common author workflows, but the execution substrate underneath it remains async-capable. That keeps local and batch usage ergonomic without collapsing the backend seam into a permanently blocking design.

Session is intentionally outside RFC 002’s normative emitted contract. It consumes plans; it does not define plan semantics.

For current package behavior, see [Execution context (Reference)][execution-ref] and [Execution context (Explanation)][execution-expl].

## Current implementation shape

The package currently uses the following implementation shape:

- author-facing carrier types live in [mod.incn](../src/dataset/mod.incn)
- canonical relational operator helpers live in [ops.incn](../src/dataset/ops.incn)
- Substrait emission lives under [substrait/](../src/substrait/)
- Prism internals live under [prism/](../src/prism/)
- `LazyFrame[T]` currently routes through a backend-native `PrismCursor[T]`
- `DataFrame[T]` and `DataStream[T]` are not yet fully converged on the same internal backing model as `LazyFrame[T]`

This is enough to explain the package architecture while keeping current API behavior in language docs and follow-on gaps in RFCs, issues, and release notes.

## Repository layout

| Path                             | Role                                               |
| -------------------------------- | -------------------------------------------------- |
| `incan.toml`                     | Package metadata and Rust dependency declarations  |
| `src/lib.incn`                   | Public package exports                             |
| `src/dataset/mod.incn`           | Carrier types and trait surface                    |
| `src/dataset/ops.incn`           | Canonical relational operator helpers              |
| `src/prism/mod.incn`             | Internal Prism graph, cursor, and lowering logic   |
| `src/substrait/relations.incn`   | Concrete `Rel` builders and relation lowering      |
| `src/substrait/plans.incn`       | Top-level `Plan` assembly helpers                  |
| `src/substrait/inspect.incn`     | Relation/plan inspection and output-column helpers |
| `src/substrait/schema_registry.incn` | Named-table schema registration and lookup     |
| `src/substrait/extensions.incn`  | Extension anchors, URIs, and declaration helpers   |
| `src/substrait/expr_lowering.incn` | Builder-to-Substrait expression lowering         |
| `src/substrait/conformance.incn` | Typed conformance facade over catalog + validators |
| `src/substrait/schema.incn`      | Model/schema to Substrait type bridging            |
| `tests/`                         | Package tests run through `incan test`             |
| `docs/language/`                 | Current package docs                               |
| `docs/rfcs/`                     | Normative RFC series                               |
| `docs/release_notes/`            | Release-facing notes                               |

Normative behavior lives in the RFC series first. Current package behavior and usage belong in the language docs. If code and RFCs disagree, treat that as a bug or transition state to resolve explicitly.

## Repository vs compiler

The InQL repository and the Incan compiler have different responsibilities.

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  InQL repo                                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  RFCs, package modules, tests, docs, architecture, conformance corpus       │
│  Defines the relational package surface and its normative contracts         │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ implemented through
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Incan compiler                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Parsing, typechecking, lowering, Rust emission, LSP, test runner, builds   │
│  Makes InQL package code executable and supports language surfaces          │
└─────────────────────────────────────────────────────────────────────────────┘
```

That distinction matters because package design and compiler implementation move at different speeds. The repo owns the package and its design records; the compiler owns the language and tooling machinery that makes those surfaces executable.

## Build and test

From the repo root, with `incan` on `PATH`:

```text
incan build --lib
incan test tests
```

In practice:

- `incan build --lib` parses, typechecks, lowers, and emits a Rust crate for the InQL library
- `incan test tests` discovers and runs package tests under `tests/`

CI builds `incan` first, then runs the InQL package checks against that compiler.

## Reading order

If you want the clearest current story, read in this order:

1. [Language overview][language-root]
2. [Dataset carriers (Explanation)][dataset-expl]
3. [Execution context (Explanation)][execution-expl]
4. [Dataset carriers (Reference)][dataset-ref]
5. [Execution context (Reference)][execution-ref]
6. RFCs for normative and historical design context

## Where to read more

| Topic                       | Location                                                   |
| --------------------------- | ---------------------------------------------------------- |
| Docs landing page           | [docs/README.md][docs-root]                                |
| Language overview           | [docs/language/README.md][language-root]                   |
| Dataset carriers            | [Reference][dataset-ref] · [Explanation][dataset-expl]     |
| Execution context           | [Reference][execution-ref] · [Explanation][execution-expl] |
| Substrait integration       | [Reference docs][substrait-read-root] · [RFC 002][rfc-002] |
| Prism planning engine       | [RFC 007][rfc-007]                                         |
| InQL RFC index              | [docs/rfcs/README.md][inql-rfcs]                           |
| Incan compiler architecture | [Incan architecture docs][incan-architecture]              |
| Contributing                | [CONTRIBUTING.md][inql-contributing]                       |

<!-- References -->
[incan-architecture]: https://github.com/dannys-code-corner/incan/blob/main/workspaces/docs-site/docs/contributing/explanation/architecture.md
[docs-root]: README.md
[language-root]: language/README.md
[inql-rfcs]: rfcs/README.md
[inql-contributing]: ../CONTRIBUTING.md
[dataset-ref]: language/reference/dataset_carriers.md
[dataset-expl]: language/explanation/dataset_carriers.md
[execution-ref]: language/reference/execution_context.md
[execution-expl]: language/explanation/execution_context.md
[substrait-read-root]: language/reference/substrait/read_root_binding_contract.md
[rfc-002]: rfcs/002_apache_substrait_integration.md
[rfc-004]: rfcs/004_inql_execution_context.md
[rfc-007]: rfcs/007_prism_planning_engine.md
[rfc-008]: rfcs/008_optimizer_boundary_stats_cbo_aqe.md
