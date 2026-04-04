# Writing InQL RFCs (how-to)

This guide is for contributors writing an RFC (design record) in the **InQL** repository.

RFC means “Request for Comments”: a normative design document under [`docs/rfcs/`](../rfcs/README.md), numbered separately from [Incan language RFCs](https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs).

!!! warning "Before you start"

    Always check whether there is already an InQL RFC or an **RFC proposal** issue covering what you want to propose. Use the [RFC index](../rfcs/README.md) and repository search.

Start here:

- [RFC index](../rfcs/README.md)
- [RFC file template](../rfcs/TEMPLATE.md)
- [RFC proposal issue](https://github.com/dannys-code-corner/InQL/issues/new?template=rfc_proposal.yml) (optional)

## When you should write an RFC

Write an InQL RFC when **behavior or contracts** of the relational layer change in a way authors or tools would rely on, for example:

- Language surface: naming, query schema, resolution rules, or new relational syntax
- `DataSet[T]` / carrier types, bounded vs unbounded rules, or public library API contracts
- Substrait (or other) logical plan shape, extension policy, or binding boundaries
- `query {}` grammar, typing, or lowering expectations
- Execution context: session, I/O boundaries, or how plans meet runners
- Any change that must stay consistent across `query {}`, method chains, and future surfaces (see InQL RFC 000)

Purely internal Incan compiler refactors with **no** InQL-visible meaning usually belong in the **Incan** repository, not as an InQL RFC.

## Workflow

1. **(Optional) Open an issue**  
   Use the **RFC proposal** template to align on problem and scope before you invest in a full document.

2. **Create the RFC file**  
   - Copy [TEMPLATE.md](../rfcs/TEMPLATE.md).  
   - Add `docs/rfcs/NNN_short_slug.md` (example: `006_window_semantics.md`).  
   - Pick the next `NNN` from the [RFC index](../rfcs/README.md) and open issues, avoiding collisions.

3. **Fill in the RFC**  
   One coherent proposal per RFC. Cover at least:

   - Motivation and concrete examples (`incan` snippets where it helps)
   - Precise rules (semantics, typing, lowering, edge cases, diagnostics)
   - Goals and non-goals
   - Alternatives, drawbacks, compatibility / migration
   - **Layers affected** (spec, this package, Incan compiler, execution) in normative terms — not a file-by-file task list

4. **Open a PR**  
   Link the PR in the RFC header (**RFC PR**). Expect review and iteration.

5. **Discuss**  
   Use the PR (and the linked issue, if any) to converge.

## After acceptance

- **Implemented:** Set **Shipped in** to the first InQL **package** release that contains the change. When you adopt a `closed/implemented/` layout (optional), move the file there and keep the index accurate.
- **Deferred:** Update **Status** to `Deferred` and record why.

## Closed RFCs (optional layout)

If you introduce subfolders under `docs/rfcs/`, a common pattern is:

- `docs/rfcs/` — active RFCs (Draft, Planned, In Progress, …)
- `docs/rfcs/closed/implemented/` — shipped
- `docs/rfcs/closed/superseded/` — replaced by a newer InQL RFC
- `docs/rfcs/closed/rejected/` — withdrawn or rejected

When superseding or rejecting, update the status line (for example `Superseded by InQL RFC NNN`) and move the file if you use `closed/`.

## Tips for a good RFC

- Prefer concrete examples over abstract prose.
- Write **reference-level** sections so an implementer could build to them.
- Call out **non-goals** explicitly.
- If the design is too large, split into a sequence of smaller RFCs with clear **Related** links.
- **Normative rules** must live in the RFC text (or this repo’s public docs), not in private notes or internal-only trees.

## Compiler and tooling work

InQL RFCs often imply changes in the **Incan** compiler (parse, check, lower). When you implement there, follow that repository’s contributor guide, **AGENTS.md**, and CI gates. The InQL RFC remains the spec; the Incan repo carries the toolchain implementation.

## Further reading (Incan documentation norms)

Narrative docs and RFCs in this repo should stay compatible with how the Incan project writes **Markdown for MkDocs Material**:

- [Incan docs-site contributor loop][incan-docsite-loop] — Divio quadrants, snippets, PR checklist for the Incan docs site.
- [Incan AGENTS.md — Docs-site workflow][incan-agents-docs-workflow] — prose without hard wrapping, `mkdocs build --strict`, and related expectations.

[incan-docsite-loop]: https://github.com/dannys-code-corner/incan/blob/main/workspaces/docs-site/docs/contributing/tutorials/book/08_docsite_contributor_loop.md
[incan-agents-docs-workflow]: https://github.com/dannys-code-corner/incan/blob/main/AGENTS.md#docs-site-workflow-mkdocs-material
