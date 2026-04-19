# Agent Instructions for InQL

<!-- Link references — defined up front so agents see all targets in one scan -->

[incan-repo]: https://github.com/dannys-code-corner/incan-programming-language
[incan-agents]: https://github.com/dannys-code-corner/incan-programming-language/blob/main/AGENTS.md
[incan-contributing]: https://github.com/dannys-code-corner/incan-programming-language/blob/main/CONTRIBUTING.md
[readme]: README.md
[contributing]: CONTRIBUTING.md
[architecture]: docs/architecture.md
[rfcs-index]: docs/rfcs/README.md
[rfc-template]: docs/rfcs/TEMPLATE.md
[writing-rfcs]: docs/contributing/writing_rfcs.md
[issue-templates]: .github/ISSUE_TEMPLATE/
[ci-workflow]: .github/workflows/ci.yml
[incan-toml]: incan.toml
[metadata-incn]: src/metadata.incn
[lib-incn]: src/lib.incn
[tests-dir]: tests/
[release-notes]: docs/release_notes/
[incan-docsite-loop]: https://github.com/dannys-code-corner/incan-programming-language/blob/main/workspaces/docs-site/docs/contributing/tutorials/book/08_docsite_contributor_loop.md
[incan-agents-docs-workflow]: https://github.com/dannys-code-corner/incan-programming-language/blob/main/AGENTS.md#docs-site-workflow-mkdocs-material

**InQL** is the typed **data logic plane** for [Incan][incan-repo]: relational queries, schema-aware table transformations, and streaming-shaped relational work, with a clear split from orchestration and engine-specific runtime in the authoring model. **This repository** holds the InQL **Incan library package** (`.incn` sources) and **normative RFCs** under `docs/rfcs/`. The **Incan compiler** (Rust) that implements parsing, checking, and lowering for InQL surfaces lives in the **Incan** repository.

This document guides AI agents and contributors working **in this repo**. For compiler implementation, Rust conventions, and the full toolchain pipeline, use **[Incan `AGENTS.md`][incan-agents]** and **[Incan `CONTRIBUTING.md`][incan-contributing]**.

> **CRITICAL — THE USER DECIDES WHAT IS RELEVANT.** Scope, PR boundaries, and which files belong on a branch are the **maintainer’s call**, not the agent’s. Do not dismiss work as “noise” or “hygiene” to remove or revert it. Ask when in doubt.
>
> **FORBIDDEN without explicit user approval that quotes the exact paths or commands:** anything that overwrites or deletes uncommitted work — including `git checkout -- <path>`, `git restore <path>`, `git clean`, `git reset --hard`, `stash drop`, or equivalent. If you believe something should be split, reverted, or omitted from a PR, **say so and ask**; do not run destructive git operations on your own initiative.

## Key references

| Topic                                               | Location                                                                                           |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Project overview                                    | [README.md][readme]                                                                                |
| Contributing (human workflow)                       | [CONTRIBUTING.md][contributing]                                                                    |
| Repo vs compiler placement                          | [docs/architecture.md][architecture]                                                               |
| Normative InQL design                               | [docs/rfcs/][rfcs-index]                                                                           |
| InQL RFC file template                              | [docs/rfcs/TEMPLATE.md][rfc-template]                                                              |
| Writing InQL RFCs (how-to)                          | [docs/contributing/writing_rfcs.md][writing-rfcs]                                                  |
| GitHub issue templates                              | [.github/ISSUE_TEMPLATE/][issue-templates]                                                         |
| Incan agent rules (Rust, compiler pipeline, skills) | [Incan `AGENTS.md`][incan-agents]                                                                  |
| Incan docs-site conventions                         | [Contributor loop][incan-docsite-loop] · [Markdown / MkDocs in AGENTS][incan-agents-docs-workflow] |

## What belongs where

| Change                                                                                        | Primary repo            |
| --------------------------------------------------------------------------------------------- | ----------------------- |
| InQL **RFC** text, `README`, `docs/*` (except normative rules in `__research__/`)             | **This repo**           |
| InQL **library** API and tests in `.incn`                                                     | **This repo**           |
| Lexer/parser/typechecker/lowering/**Rust** for InQL syntax, `query {}`, `DataSet` integration | [**Incan**][incan-repo] |

Normative behavior is defined in **`docs/rfcs/`**. If package code and an RFC disagree, treat it as a bug unless the RFC is explicitly superseded.

**RFCs must not** defer normative rules to `__research__/` or internal-only trees; anything readers need belongs in the RFC or public docs (see [docs/rfcs/README.md][rfcs-index]).

**Docs governance matters here:**

- RFCs are **moment-in-time design records**, not current-state implementation notes.
- RFC drafting must start from the **north-star end state**. Do not default to minimizing RFC scope, carving it into the smallest possible change, or reframing a design request as an implementation-sized patch unless the maintainer explicitly asks for an incremental slice.
- Current package behavior belongs in normal docs under `docs/` (for example `language/reference/`, `language/explanation/`, `architecture.md`, and `release_notes/`).
- If implementation teaches something new, either:
  - fix the code to match the RFC,
  - open issues and update regular docs for current behavior, or
  - make a deliberate RFC amendment / follow-on RFC when the design itself changed.
- Do **not** rewrite accepted RFCs into progress logs, phase journals, or “how it currently works” pages.

## General workflow

1. **Branch from `main`**: Prefer `<type>/<issue>-<slug>` (e.g. `feature/8-rfc-table-automation`, `docs/9-mkdocs-ci`), matching team practice.
2. **Follow RFCs**: Behavior changes should be reflected in the right `docs/rfcs/*.md` (or a new RFC) before or alongside code in the appropriate repository.
3. **Run the local gate**: `make ci` (or at least `make fmt-check`, `make build`, `make test`) before considering work done for **this** repo.
4. **Version sync**: If you bump the package version, update **both** [incan.toml][incan-toml] (`[project] version`) and [src/metadata.incn][metadata-incn] (`inql_version()`) in the same commit (see [CONTRIBUTING.md][contributing]).
5. **Documentation**: User-facing or spec changes should update `README.md`, relevant `docs/*`, or RFCs as appropriate. Keep prose markdown **without hard wrapping** (natural paragraphs).
   - Use RFCs for normative design and design history.
   - Use `docs/language/reference/` for current API/contracts.
   - Use `docs/language/explanation/` for current mental models and usage framing.
   - Use `docs/architecture.md` for system boundaries, not implementation diaries.
   - Use `docs/release_notes/` for shipped/user-visible changes.

## Common commands (this repo)

| Command                                  | Purpose                                                                                     |
| ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| `make help`                              | List targets                                                                                |
| `make ci`                                | Same as CI: `fmt-check`, `build`, `test`                                                    |
| `make check` / `make pre-commit`         | Alias-style gate: format check + build + test                                               |
| `make fmt`                               | Format package `.incn` sources (`src/`, `tests/`, `examples/` only)                         |
| `make fmt-check`                         | Check formatting without writing (same scope)                                               |
| `make build`                             | `incan build --lib`                                                                         |
| `make test`                              | `incan test tests` (package `tests/` only; avoids picking up a sibling `./incan/` checkout) |
| `make test-style`                        | Check `Arrange / Act / Assert` markers for each test function in `tests/*.incn`             |
| `make build-locked` / `make test-locked` | Stricter lockfile mode                                                                      |

Requires `incan` on `PATH`, or `make build INCAN=/path/to/incan`. CI builds Incan from source then runs `make ci` (see [.github/workflows/ci.yml][ci-workflow]).

## Incan source (`.incn`) in this package

- Match existing module layout, naming, and export style in [src/lib.incn][lib-incn].
- Add or extend **tests** under [tests/][tests-dir] for observable behavior.
- For language semantics that are not yet specified, **anchor design in RFCs** rather than inventing silent behavior.
- **Docstrings are required**: every function or method with a body (`def ...`) must include a docstring. When touching legacy code that lacks one, add it in the same change.

### Test style contract

- Every `def test_*` function in `tests/*.incn` must include explicit section markers:
  - `# -- Arrange --`
  - `# -- Act --`
  - `# -- Assert --`
- Combined markers are allowed when compactness is warranted (for example `# -- Act & Assert --`) as long as all three dimensions are present in the test body.
- Keep compile-shape tests explicit: if runtime assertions are not meaningful, still include an `Assert` section with a compile-shape assertion/comment so intent is unambiguous.
- The repository gate enforces this via `make test-style`.

## Markdown and RFC style

- **RFCs**: Use [docs/rfcs/TEMPLATE.md][rfc-template] and [docs/contributing/writing_rfcs.md][writing-rfcs]; use **normative** language (`must` / `should`) consistently; keep **Related** headers and cross-links purposeful (prefer sequential dependencies — see existing RFCs).
- **RFC scope discipline**: treat RFCs as north-star contract documents first. Begin with the target steady-state design, then reason backward to implementation slices only after that design is explicit and agreed. Do not repeatedly fall back to "smallest possible RFC" framing; that is a workflow choice, not a default architectural principle in this repo.
- **RFC lifecycle discipline**: once an RFC is accepted/planned, do not silently re-author it into an implementation notebook. Reintroduce only stable, deliberate design changes; move implementation discoveries and current behavior into ordinary docs and issues.
- **Prose docs**: No mandatory hard wrap; prefer clarity and scannable headings.
- **Docs structure**: follow the Incan-style split even before full MkDocs adoption: docs landing page, `language/reference`, `language/explanation`, architecture/contributing pages, RFCs, and release notes.
- **Incan-aligned docs:** Shared **Markdown / MkDocs** norms (Divio layout, no hard wrap, `mkdocs build --strict`, Material admonitions): [Incan docsite contributor loop][incan-docsite-loop], [Docs-site workflow in Incan AGENTS.md][incan-agents-docs-workflow].
- **Links — central definitions:** Prefer **reference-style** Markdown with targets in **one block** at the **bottom** of the file under a `<!-- References -->` HTML comment. Use inline links only when a file has very few links or when a URL must stay literal (e.g. inside a fenced code block for copy-paste).
- **Release notes**: When shipping a version, update [docs/release_notes/][release-notes] (see structure in existing files); use sections such as “Features and enhancements” / “Bugfixes”; link issues and PRs.

## Rust and the Incan compiler

This repository does **not** host the compiler Rust codebase. If your task includes changes under the Incan workspace:

- Treat **[Incan `AGENTS.md`][incan-agents]** as authoritative for **no `.unwrap()` / `.expect()`**, clippy, rustdoc, and pipeline boundaries.
- Use `cargo test`, `cargo clippy`, and snapshot workflows **there**, not as substitutes for `make ci` **here** when you only touch InQL.

## Cursor skills (optional)

Reusable Incan workflows live in the Incan repo under `.cursor/skills/` (e.g. `/write-rfc`, `/review-rfc`, `/bump-rfc`). Use them when the task spans Incan or when drafting/reviewing InQL RFCs and you want the same structural discipline — adapted to **`docs/rfcs/`** and InQL’s numbering.

## PR checklist (this repo)

- [ ] `make ci` passes (or equivalent `fmt-check`, `build`, `test`).
- [ ] Semantics changes cite or update the relevant **RFC** in `docs/rfcs/`.
- [ ] **Version**: if `version` changed, [incan.toml][incan-toml] and [src/metadata.incn][metadata-incn] stay in sync.
- [ ] **README** / **docs** updated for anything a new contributor or user would notice.
- [ ] If the change is user-facing, **release notes** under `docs/release_notes/` updated when appropriate.
- [ ] Compiler or Rust changes (if any in another PR) follow Incan’s gates and **[Incan `AGENTS.md`][incan-agents]**.
