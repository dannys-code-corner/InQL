# InQL architecture

This document describes how the **InQL** project is structured and how it relates to the **Incan** compiler. It is modeled on Incan’s architecture overview but scoped to this repository and the relational **data logic** layer (not orchestration or engine-specific runtime in the authoring model).

## What InQL is

InQL is two things that evolve together:

1. **A specification** — Normative design under [docs/rfcs/][inql-rfcs]: language surface and naming, dataset types (including bounded vs unbounded carriers), portable logical plans (Substrait), `query { }` authoring, execution context (session and I/O), and (later) optional pipe-forward. **InQL v0.1** is scoped through execution context; pipe-forward is specified for alignment but not part of that release.
2. **An Incan library package** — `.incn` sources built with `incan build --lib`, published as a dependency for Incan programs.

The **compiler** that parses, typechecks, and lowers InQL syntax into plans or Rust lives in the [Incan repository][incan-repo]. This repo holds the **author-facing package** and the **normative design docs** that implementation work should follow.

## High-level placement

<!-- TODO: update this once we add the DSL/vocab crate -->

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  InQL repo (this project)                                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  docs/rfcs/          ──►  Normative design (numbered proposals + index)     │
│  src/*.incn          ──►  Library surface (exports, helpers, version)       │
│  tests/              ──►  Package tests                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │  informs & consumed by
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Incan compiler (separate repo)                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Lexer / Parser / AST  ──►  Typechecker  ──►  Lowering  ──►  Rust / plans   │
│       (relational syntax, models, and `DataSet` types are checked here)     │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Intended language pipeline (relational surface)

The compiler implements this story over time. Conceptually:

```text
Incan models (row types)
        │
        ▼
  DataSet[T] carriers  ◄──  programmatic API (bounded vs unbounded traits)
        │
        ├──►  query { } blocks
        │         │
        │         └──►  Substrait-shaped logical plan
        │                    │
        │                    └──►  Session: bind, execute, read/write
        │
        └──►  optional pipe-forward  (later release; same naming core as blocks/chains)
```

Across blocks, method chains, and (when present) pipe-forward, authors share one notion of **naming** and **query schema evolution**—including forms like `.column`, `alias.column`, bare names in the query schema, and outer bindings—plus clear **layer boundaries** between data logic and execution. Precise rules live in the specification documents under [docs/rfcs/][inql-rfcs].

## Repository layout

| Path                   | Role                                                |
| ---------------------- | --------------------------------------------------- |
| `incan.toml`           | Package metadata (`name`, `version`)                |
| `src/lib.incn`         | Public module: re-exports and package docs          |
| `src/*.incn`           | Library implementation modules                      |
| `tests/`               | `incan test` targets                                |
| `docs/rfcs/`           | Specification index and individual proposals        |
| `docs/architecture.md` | This document: repo placement vs the Incan compiler |
| `docs/README.md`       | Pointer into documentation                          |

Normative behavior is defined in the **RFC series**, not only in code. If code and a spec disagree, treat it as a bug unless the document is explicitly superseded.

## Build and test (this package)

From the repo root, with `incan` on your `PATH`:

```text
make ci
  │
  └──►  incan fmt --check (package dirs)  →  incan build --lib  →  incan test
```

Equivalent raw commands:

```text
incan build --lib
  │
  └──►  Incan frontend (parse, check, …) + backend emit a Rust crate for the library
        (same staged pipeline as application builds; see [Incan architecture docs][incan-architecture])

incan test
  │
  └──►  Discover and run tests under tests/
```

**GitHub Actions** does not assume a preinstalled `incan` binary: the workflow checks out the [Incan compiler repository][incan-repo], runs `cargo build --release`, adds `target/release` to `PATH`, then runs `make ci` in this tree.

For **stage-by-stage** debugging (`--parse`, `--check`, `--emit-rust`, etc.), use the Incan CLI options documented in the Incan project.

## Where to read more

| Topic                             | Location                                                              |
| --------------------------------- | --------------------------------------------------------------------- |
| Full compiler module map, IR, LSP | [Incan compiler architecture][incan-architecture] (in the Incan repo) |
| InQL specification                | [docs/rfcs/][inql-rfcs]                                               |
| Contributing                      | [CONTRIBUTING.md][inql-contributing]                                  |

<!-- Link references (single place for targets) -->

[incan-repo]: https://github.com/dannys-code-corner/incan-programming-language
[incan-architecture]: https://github.com/dannys-code-corner/incan-programming-language/blob/main/workspaces/docs-site/docs/contributing/explanation/architecture.md
[inql-rfcs]: rfcs/README.md
[inql-contributing]: ../CONTRIBUTING.md