# Contributing to InQL

Thank you for your interest in InQL — the typed relational layer for [Incan][incan-repo]. This document is the entry point for contributing to **this repository** (the InQL package and its design RFCs).

## Start here

| Resource | Purpose |
| -------- | ------- |
| [README.md][readme] | What InQL is and how to build the library |
| [AGENTS.md][agents] | AI/agent and maintainer rules; repo vs Incan boundaries |
| [docs/architecture.md][architecture] | How this repo fits next to the Incan compiler |
| [docs/rfcs/][rfcs-index] | Normative design proposals (behavior changes start here) |
| [Writing InQL RFCs][writing-rfcs] | How to draft RFCs, workflow, and tips |
| [Incan `CONTRIBUTING.md`][incan-contributing] | Compiler, tooling, and Rust workspace workflow |
| [Incan docs-site contributor loop][incan-docsite-loop] | Divio layout, snippets, checklist for Incan’s MkDocs site — mirror these patterns when shaping InQL `docs/` |
| [Incan AGENTS — Docs-site workflow][incan-agents-docs-workflow] | Prose **without hard wrapping**, `mkdocs build --strict`, Material-friendly Markdown |

**Compiler and language implementation** (lexer, parser, typechecker, lowering, codegen for `query {}` and related surfaces) lives in the **Incan** repository. Use that project’s docs and gates when you change the toolchain. Use **this** repo for the InQL **library source** (`.incn`) and **InQL RFCs** that specify the relational surface.

## Getting started

1. **Install a matching Incan toolchain**  
   Build or install `incan` so it is on your `PATH` (start from the [Incan repository][incan-repo] and its contributor docs). The CI uses a **reusable composite action** that caches built binaries for faster subsequent runs.

2. **Clone this repository**

   ```bash
   git clone https://github.com/dannys-code-corner/InQL
   cd InQL
   ```

3. **Build and verify**

   Recommended (matches [CI][ci-workflow]: format check, library build, tests):

   ```bash
   make ci
   ```

   Or step by step:

   ```bash
   make fmt-check
   make test-style
   make build
   make test
   ```

   With `incan` on your `PATH` you can call `incan build --lib` and `incan test tests` directly (use the `tests/` path so a sibling Incan checkout under `./incan/` is not collected). Override the binary with `make build INCAN=/path/to/incan` if needed.

## Project structure

See [docs/architecture.md][architecture] for a concise map. In short:

- `incan.toml` — package name and version
- `src/*.incn` — library modules; `lib.incn` re-exports the public surface
- `tests/` — Incan tests for the package
- `docs/rfcs/` — design specifications (numbered separately from Incan’s RFC index)

## Changing behavior

1. **Specify the change in an InQL RFC** under `docs/rfcs/` (or amend an existing RFC). New documents should start from [RFC template][rfc-template]; follow [Writing InQL RFCs][writing-rfcs] for workflow so naming, typing, and lowering rules stay coherent across `query {}`, carriers, and optional pipe-forward.
2. **Implement** in the right place: library APIs here when they are ordinary Incan code; compiler or stdlib changes in the Incan repo as needed.
3. **Keep the [README][readme] and docs accurate** for anything a new user or contributor would notice.

### Function docstrings

- Every function or method with a body (`def ...`) in `.incn` files must include a docstring.
- When modifying legacy code that lacks a docstring, add the missing docstring in the same change.
- Prefer intent-level docstrings (what/why, invariants, boundary behavior), not line-by-line narration.

### Test style (canonical)

- Every `def test_*` in `tests/*.incn` must include explicit `Arrange`, `Act`, and `Assert` section markers:
  - `# -- Arrange --`
  - `# -- Act --`
  - `# -- Assert --`
- You may combine sections for concise cases (`# -- Act & Assert --`), but each test must still contain all three dimensions.
- Compile-shape tests are not exempt; include an `Assert` section stating compile-shape intent.
- Run `make test-style` locally (and note it is part of `make check` / `make ci`).

### RFCs vs regular docs

- RFCs are design records and normative proposals. They are not the primary place to document current package behavior.
- Current API shape, usage, and implemented details belong in normal docs under `docs/`:
  - `docs/language/reference/` for current contracts
  - `docs/language/explanation/` for mental models and usage guidance
  - `docs/architecture.md` for system boundaries
  - `docs/release_notes/` for shipped/user-visible changes
- If implementation diverges from an RFC, do not quietly rewrite the RFC into an implementation diary. Either fix the code, update ordinary docs and issues, or make a deliberate RFC amendment / follow-on RFC.

## Version bumps

InQL carries its version in two places that **must stay in sync**:

1. `incan.toml` — `[project] version = "…"`
2. `src/metadata.incn` — the string returned by `inql_version()`

Bump both in the same commit.

## Issue labels (triage)

Templates apply **type** labels (`bug`, `feature`, `chore`, `documentation`, `RFC`) where relevant. **[Auto-label workflow][issue-auto-label]** syncs **scope** labels from the shared *Area* field on issue forms and from *Area(s)* checkboxes in [pull request template][pr-template]. Keep those option strings and the workflow’s `AREA_OPTION_TO_LABEL_JSON` in lockstep.

| Label | Use for |
| ----- | ------- |
| `package` | Library source, tests, manifest |
| `specification` | `docs/rfcs/` (normative specs) |
| `documentation` | README and other docs outside the RFC series (often with the `documentation` template) |
| `automation` | CI, `Makefile`, `.github/`, repo config |
| `other` | Does not fit the labels above |

### Triage GitHub App (CI)

The auto-label workflow uses a shared **organization-level GitHub App** installation for `dannys-code-corner`. Configure these **organization Actions secrets** and grant access to this repository:

| Secret | Purpose |
| ------ | ------- |
| `TRIAGE_APP_ID` | App ID |
| `TRIAGE_APP_INSTALLATION_ID` | Installation ID for the **organization-level** app installation |
| `TRIAGE_APP_PRIVATE_KEY` | App private key (PEM) |

Install the app on the `dannys-code-corner` organization and grant it access to `InQL` (and any future repositories that should share triage automation). Without these secrets the workflow fails at the token step.

## Pull request guidelines

1. Run **`make ci`** (or at least `make fmt-check`, `make build`, and `make test`) and fix failures.
2. If you changed semantics, cite the **RFC** (or add/update one).
3. Use **clear commit messages** and a PR description that states intent and scope.
4. For Rust-formatting or `cargo`/`clippy` expectations, follow the [Incan contributor guide][incan-contributing] when your change touches that repository.

## Questions?

When you open an issue, GitHub offers **templates** under [`.github/ISSUE_TEMPLATE/`][issue-templates] (bug report, feature request, chore, documentation, RFC proposal), aligned with the Incan project’s shape.

Open an issue on this repository for InQL-specific design or package questions; use the [Incan repository][incan-repo] for compiler and language issues that are not InQL-scoped.

<!-- References -->

[incan-repo]: https://github.com/dannys-code-corner/incan
[incan-contributing]: https://github.com/dannys-code-corner/incan/blob/main/CONTRIBUTING.md
[readme]: README.md
[agents]: AGENTS.md
[architecture]: docs/architecture.md
[rfcs-index]: docs/rfcs/README.md
[writing-rfcs]: docs/contributing/writing_rfcs.md
[rfc-template]: docs/rfcs/TEMPLATE.md
[ci-workflow]: .github/workflows/ci.yml
[issue-auto-label]: .github/workflows/issue_auto_label.yml
[pr-template]: .github/pull_request_template.md
[issue-templates]: .github/ISSUE_TEMPLATE/
[incan-docsite-loop]: https://github.com/dannys-code-corner/incan/blob/main/workspaces/docs-site/docs/contributing/tutorials/book/08_docsite_contributor_loop.md
[incan-agents-docs-workflow]: https://github.com/dannys-code-corner/incan/blob/main/AGENTS.md#docs-site-workflow-mkdocs-material
