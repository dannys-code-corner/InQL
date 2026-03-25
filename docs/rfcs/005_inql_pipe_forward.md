# InQL RFC 005: Pipe-forward relational syntax (`|>`)

- **Status:** Blocked
- **Created:** 2026-03-18
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — naming and query schema; **must** stay aligned)
  - InQL RFC 001 (dataset types — carriers and method APIs)
  - InQL RFC 003 (`query {}` — primary clause surface)
  - Incan RFC 040 (Scoped DSL Glyph Surfaces — prerequisite for `|>` support)
- **Issue:** [InQL #6](https://github.com/dannys-code-corner/InQL/issues/6)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** -

## Summary

This RFC specifies an optional **pipe-forward** surface for InQL relational pipelines: `|>`-chained stages applied to an existing `DataSet[T]` expression, with grammar, precedence, and desugaring to the same relational semantics as `query {}` and collection method chains. Identifier resolution and current query schema behavior **must** match InQL RFC 000 §§2–4; this RFC does **not** redefine naming rules.

> **Blcoked** for the following reason:
>
> - **Glyph dependency**: `|>` depends on Incan RFC 040.

## Motivation

Some authors prefer a linear, left-to-right pipeline over clause blocks or method chains. In Incan specifically, pipe-forward becomes useful when authors already have a `DataSet[T]` value in scope and want concise shorthand for a few relational steps without switching into a larger `query {}` block. A dedicated RFC keeps InQL RFC 003 focused on `query {}` while still committing to one relational semantic core across surfaces (InQL RFC 000 §6).

### Prior art

This RFC draws inspiration from Google's pipe query syntax in BigQuery ([documentation](https://docs.cloud.google.com/bigquery/docs/pipe-syntax-guide)), which shows that a linear relational pipeline can remain readable for complex query construction. That reference is informative, not normative: InQL pipe-forward semantics are defined by this RFC together with InQL RFC 000 and RFC 003. This RFC aims to stay close to the core value of that design, especially expression-first pipeline authoring, while still allowing Incan-specific choices where required by the type system, library model, and RFC-defined surface conventions.

## Goals

- Define normative pipe-forward syntax (tokens, allowed stage heads, and how an existing `DataSet[T]` value becomes the current input relation).
- Specify desugaring or lowering to the same relational plan shape as `query {}` / InQL RFC 001 operations (without mandating a single internal representation).
- Require consistency with InQL RFC 000: `.column`, `relation.column`, bare names, and `SELECT` alias boundaries behave identically to `query {}` in equivalent pipelines.
- Cover tooling expectations at a high level (LSP, diagnostics) where they differ from `query {}`.

## Non-Goals

- Replacing `query {}` as the primary checked surface — InQL RFC 003 remains the default clause grammar unless product direction changes.
- Normative naming rules — InQL RFC 000.
- `DataSet[T]` type definitions and backend boundary — InQL RFC 001.
- Substrait emission details — may reuse InQL RFC 003 paths after desugar; no duplicate Substrait spec unless a gap is found.
- Execution context and session — InQL RFC 004.
- Finalizing a dedicated aggregate-stage spelling analogous to BigQuery's `AGGREGATE` operator. The initial value of pipe-forward is expression-first relational shorthand; aggregate-stage surface details may be refined in a follow-up amendment as long as semantics stay aligned with InQL RFC 003.

## Guide-level explanation

```incan
new_df = df
    |> where .amount > 100
    |> select { region }
```

This is the core value proposition: when a dataset value already exists in scope, pipe-forward gives a concise linear shorthand for a few relational steps. The semantics are identical to an equivalent `query {}` block or method chain; only syntax differs. `.amount` and `region` resolve using the same rules as the other InQL surfaces.

Another common pattern is incremental narrowing over an existing dataset:

```incan
preview = orders
    |> where .status == "completed"
    |> order_by .created_at desc
    |> limit 20
```

Pipe-forward also supports enrichment without leaving the linear flow:

```incan
enriched = orders
    |> join customers on .customer_id == customers.id
    |> where .amount > 100
    |> select { .order_id, customers.name, .amount }
```

Because the pipe attaches to any `DataSet[T]`-producing expression, it can continue from a `query {}` result as well:

```incan
filtered = query {
    FROM orders
} |> where .amount > 100
```

## Reference-level explanation

### Token and stage syntax

- Pipe-forward uses the `|>` token to chain relational stages.
- A pipe-forward expression may begin with any expression whose type conforms to `DataSet[T]` per InQL RFC 001; that expression establishes the primary relation and its schema `T`.
- A pipe chain may follow any expression whose type conforms to `DataSet[T]`, including a named dataset value, a method-chain result, or a `query { ... }` expression.
- Each subsequent stage is introduced by `|>` followed by a relational operation keyword and its arguments.
- Stage keywords are case-insensitive. Examples in this RFC use lowercase because pipe-forward is intended as lightweight shorthand over existing dataset values.
- Parentheses may be used to make the pipe attachment explicit when needed.

### Relational operation keywords

Pipe-forward stages use the same relational operations as `query {}` (InQL RFC 003). The keyword set for the initial implementation includes at least: `where`, `join`, `group_by`, `select`, `order_by`, `limit`, `explode`.

### Desugaring

A pipe-forward expression **must** desugar to the same relational plan as an equivalent `query {}` block or `DataSet[T]` method chain. Implementations **may** desugar to `query {}` AST nodes, to InQL RFC 001 trait method calls, or directly to Substrait — provided the resulting plan is semantically identical.

### Identifier resolution

All naming rules from InQL RFC 000 §2–§4 apply identically:

- `.column` resolves against the primary relation established by the input dataset expression.
- `relation.column` resolves against named join relations.
- Bare identifiers resolve against the current pipeline schema first, then lexical Incan bindings.
- `SELECT` alias boundaries apply: aliases introduced in a `SELECT` stage are visible in subsequent stages, not in sibling expressions.

### Vocabulary activation

Pipe-forward stage keywords are activated through library-driven vocabulary, consistent with InQL RFC 003's `query` keyword activation. A compilation unit with InQL active **must** recognize pipe-forward stage keywords as soft keywords.

## Design details

1. Dependency on InQL RFC 000
  Pipe-forward **must not** introduce a second resolution order for bare names or `.column`. Any deviation requires amending InQL RFC 000.

2. Dependency on Incan RFC 040
  This RFC is blocked on Incan RFC 040. The planned implementation path for `|>` is the scoped DSL glyph mechanism defined there, not ambient global operator support in ordinary Incan expressions.

## Alternatives considered

- **Fold pipe-forward into InQL RFC 003**: rejected. Keeps `query {}` RFC smaller and allows pipe syntax to ship on an independent timeline.

## Drawbacks

- **Three** user-visible relational spellings (`query {}`, chains, pipes) increase documentation and pedagogical load; InQL RFC 000 mitigates by unifying semantics.

## Layers affected

- **Parser / AST** for pipe stages and `|>` token.
- **Typechecker**: same relation schema flow as `query {}` after desugar or shared analysis.
- **Lowering / IR**: shared path with InQL RFC 003 where possible.
- **LSP**: pipe-specific completions if syntax diverges from `query {}`.

## Design Decisions

- **Keyword casing**: pipe-forward stage keywords are case-insensitive. Examples in this RFC use lowercase to emphasize pipe-forward as lightweight shorthand over existing values; uppercase spellings remain valid.
- **Entry shape (initial)**: a pipe chain starts from an expression whose type conforms to `DataSet[T]`. There is no standalone `from <expr>` entry sugar in this surface.
- **Vocabulary activation**: library-driven, sharing the activation mechanism with InQL RFC 003's `query` keyword. Pipe-forward stage keywords are soft keywords activated by the InQL library dependency.
- **Method chain interaction**: mixing `|>` stages and `.method()` calls in a single pipeline expression is not supported initially. Authors choose one surface per pipeline. Interoperability may be explored in a future amendment.
- **Aggregate-stage shape**: this RFC does not yet standardize a dedicated `aggregate` stage matching BigQuery's `AGGREGATE` operator. That surface may be refined in a follow-up amendment, provided the resulting semantics remain aligned with InQL RFC 003.
- **Not in v0.1 scope**: pipe-forward is deferred from InQL v0.1. It does not unlock new capability beyond what `query {}` and method chains already provide.
