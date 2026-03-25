# InQL RFC 000: Language Specification

- **Status:** Planned
- **Created:** 2026-03-18
- **Author(s):** Danny Meijer
- **Related:** -
- **Issue:** [InQL #1](https://github.com/dannys-code-corner/InQL/issues/1)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** -

## Summary

InQL is the typed data logic plane for the Incan ecosystem: relational queries, schema-parameterized DataFrame transformations, and backend-neutral logical planning. This RFC is the foundational specification for InQL v0.1. It defines what InQL is and what it owns, the core semantic model, naming forms and resolution rules, schema shapes, the compilation model, and layer boundaries. Companion RFCs address dataset types, plan interchange, query grammar, execution context, and pipe-forward syntax.

## Core model

InQL treats all data logic as one relational semantic model exposed through multiple authoring surfaces:

1. **`query {}` blocks** — SQL-familiar clause surface
2. **`DataSet[T]` method chains** — programmatic API

These are not separate languages; they are two authoring surfaces over one internal model. A future pipe-forward surface **must** desugar to the same model when introduced.

That model has four ingredients:

1. **Lexical Incan scope** — ordinary bindings, imports, constants, and functions visible from the surrounding module or function.
2. **Relation registry** — the current primary relation plus any named joined relations.
3. **Current query schema** — the columns currently visible at this stage of the relational plan.
4. **Published projection aliases** — the output schema produced after a projection boundary.

The key rule: **in relational clause positions, bare identifiers resolve against the current query schema before lexical Incan bindings.**

## Motivation

Relational code in Incan must resolve field access and column names deterministically and statically where the language promises checking. Without a single foundational specification, `query {}` and method-chain surfaces would drift, schema-shape rules would be inconsistent across carriers, and the boundary between data logic and execution would blur. This RFC consolidates that model so every companion RFC can cite it rather than redefine it, and so that completion of RFCs 000–004 constitutes a usable InQL v0.1.

## Goals

- Define what InQL is, what it owns, and what it does not own.
- Establish the core semantic model: one relational model, multiple authoring surfaces.
- Define the four naming forms and resolution rules for identifier binding inside relational contexts.
- Define schema shapes: fully typed, open-ended, and dynamic.
- Define the compilation model: how InQL source flows through the Incan compiler pipeline to portable plans and execution.
- Define layer boundaries: InQL owns typed data logic and logical planning; execution, orchestration, and operational semantics belong to adjacent layers.
- Establish the relationship to Incan models as the schema surface for query authoring and validation.
- State that `query {}` and method-chain surfaces, when both are present, **must not** change resolution rules relative to each other.
- State that `.column` is only valid inside relational expression positions — `query {}` blocks, relational operation arguments (e.g. `.filter(...)`, `.join(...)`), and future pipe-forward stages — and is not a general Incan expression form.

## Non-Goals

- Dataset types, carriers, and the trait hierarchy (`DataSet[T]`, `BoundedDataSet[T]`, `UnboundedDataSet[T]`) — InQL RFC 001.
- Apache Substrait plan interchange and `Rel`-level mapping — InQL RFC 002.
- `query {}` grammar, clause inventory, and typechecking — InQL RFC 003.
- Execution context, session, DataFusion, read/write — InQL RFC 004.
- Pipe-forward (`|>`) syntax — InQL RFC 005 (not in v0.1 scope).
- Orchestration, workflows, quality gates, contract enforcement — execution and operational layers.
- Governed business meaning and semantic serving — semantic layer.
- Cluster-scale scheduling, distributed fault tolerance — orchestration layer.

## Guide-level explanation

### What InQL does

InQL lets authors express typed data logic — queries, transformations, aggregations, joins — against schema-parameterized datasets, with compile-time validation and backend-neutral logical plans. Authors write relational intent; the compiler checks it against `model` schemas; the toolchain lowers it to a portable plan (Apache Substrait) that an execution context can optimize and run.

### The four naming forms

When you write `.amount`, `amount`, `orders.amount`, or `total_spend` inside an InQL query, what exactly are you referring to?

InQL has four distinct naming forms, and the answer depends on which one you're using:

- `.column` — the field from the current input relation
- `relation.column` — the field from a named joined relation
- bare `column` — a column currently visible in the query schema
- an ordinary Incan name — a binding, function, or constant from the surrounding code

Here are all four forms in one query:

```incan
threshold = 1000

query {
    FROM orders
    JOIN customers ON .customer_id == customers.id
    WHERE .amount > threshold
    SELECT
        .order_id,
        customers.name,
        .amount,
}
```

`.amount` and `.customer_id` are form 1 (primary relation). `customers.name` is form 2 (named join). `threshold` is form 4 (ordinary Incan binding — no column by that name exists in the query schema).

If you remember only five things about InQL naming:

1. `.column` is explicit access to the current input relation.
2. `relation.column` is explicit access to a named joined relation.
3. Bare names resolve to the current query schema first. When a bare name shadows an outer binding, the compiler warns and suggests the `.column` form.
4. Projection aliases are visible to later expressions in the same `SELECT` list (in order) and to all following clauses.
5. If no column matches, ordinary Incan bindings may still be used where the grammar allows ordinary expressions.

### Schema shapes

InQL supports several schema modes without losing its semantic model:

- **Fully typed** (`DataFrame[Order]`) — the compiler knows every field name, type, and nullability. This is the strongest and most ergonomic mode.
- **Open-ended** (`DataFrame[Event]` where `Event` is declared `with OpenEnded`) — declared fields are fully typed; undeclared fields become soft-runtime references with a warning.
- **Dynamic** (`DataFrame[Dynamic]`) — full field resolution is runtime-driven, but the compiler still tracks a lower-bound structural surface (minimum required inputs, minimum guaranteed outputs).

### Two authoring surfaces (v0.1)

The examples in this section assume aggregate helpers such as `sum` are imported from `pub::inql.functions` (they are not ambient builtins).

```incan
from pub::inql.functions import sum

# query {} blocks
query {
    FROM orders
    WHERE .amount > 100
    GROUP BY .region
    SELECT region, sum(.amount) as total
}

# method chains
orders
    .filter(.amount > 100)
    .group_by(.region)
    .agg(sum(.amount) as total)
```

These lower to the same relational plan. Identifier resolution rules do not change between them.

## Reference-level explanation

### 1. What InQL owns

InQL owns the data-logic concerns of the platform:

- query authoring
- relational plan construction
- column and alias resolution
- schema flow through query stages
- typed DataFrame semantics
- backend-neutral logical planning

It **must not** become:

- an orchestration or pipeline framework
- a semantic catalog or governed business-meaning layer
- an execution runtime that swallows runner and adapter concerns
- a catch-all engine abstraction

Its job is to preserve one deterministic, type-aware model of data intent across authoring surfaces.

### 2. Naming forms

Inside InQL relational contexts there are four distinct naming forms:

1. **`.column`** (primary relation)
    Explicit field access against the current input relation. In `query {}`, that is the relation established by `FROM` for `.column` positions — including after `JOIN`. `.column` does not refer to joined relations.

2. **`relation.column`** (named relation)
    Explicit field access against a named relation (e.g. a `JOIN` alias). `relation` is the relation name; `column` is the field on that relation's row type.

3. **Bare `column`**
    A bare identifier interpreted as a column in the current query schema (see §3), in relational clause positions.

4. **Ordinary Incan binding**
    Lexical bindings, functions, constants, imports, and other symbols that are not query-schema columns. Used when the grammar allows a general Incan expression and a bare name does not resolve as a column (§4.5).

The syntax is asymmetric by design:

- `.column` — "field from the current input relation."
- `relation.column` — "field from that named relation."
- bare `column` — "column visible at this stage of the query."

That split keeps a SQL-familiar surface while preserving static checking.

### 3. Current query schema

Every stage of a `query {}` block has a current query schema: the set of column names (and their types) that bare identifiers may denote in relational positions.

It evolves top to bottom, including at least:

- `FROM` establishes the base schema of the primary relation.
- `JOIN` adds named secondary relations; `relation.column` disambiguates fields from those relations.
- `GROUP BY` constrains what may appear unaggregated in subsequent `SELECT`.
- `WINDOW BY` (or equivalent) may extend the schema with computed columns.
- `SELECT ... as alias` produces a new output schema for following clauses.
- Clauses after `SELECT` (e.g. post-`SELECT` filters, `ORDER BY`) see the schema produced by the preceding `SELECT`.

Example (illustrative):

```incan
query {
    FROM orders
    GROUP BY .customer_id
    SELECT
        customer_id,
        sum(.amount) as total_spend,
    WHERE total_spend > 1000
    ORDER BY total_spend DESC
}
```

`total_spend` in the later `WHERE` and `ORDER BY` is not an outer Incan variable. It is a column in the current query schema produced by the preceding `SELECT`.

### 4. Resolution rules

#### Rule 1: `.column` is always the primary input relation

`.status` denotes the `status` field on the primary relation from `FROM`, including after `JOIN`. `.column` does not refer to joined relations; use `relation.column` for those.

```incan
query {
    FROM orders
    WHERE .status == "completed"
}
```

Here `.status` is `orders.status`.

#### Rule 2: `relation.column` is always the named relation

Joined fields **must** use `alias.column` when referring to the joined relation's fields.

```incan
query {
    FROM orders
    JOIN customers ON .customer_id == customers.id
    SELECT
        .order_id,
        customers.name,
        .amount,
}
```

#### Rule 3: bare names resolve against the current query schema first

In relational clause positions, a bare identifier **must** resolve against the current query schema before ordinary lexical Incan bindings are considered, when a matching column exists.

```incan
query {
    FROM orders
    SELECT customer_id, sum(.amount) as total_spend
    ORDER BY total_spend DESC
}
```

Query text reads as relational logic, not a competition between relational names and outer variables. If an outer Incan binding shares a name with a query column, the column wins in those positions.

#### Rule 4: `SELECT` alias visibility

Aliases introduced in a `SELECT` list are available to subsequent expressions in the same projection list, in order. An alias defined on line N is visible to expressions on lines N+1, N+2, etc., but not to expressions that precede it. Aliases also become part of the output schema for all following clauses (`ORDER BY`, post-`SELECT WHERE`, etc.).

```incan
query {
    FROM orders
    SELECT
        sum(.amount) as total_spend,
        total_spend * 0.2 as tax,
    ORDER BY total_spend DESC
}
```

Here `tax` may reference `total_spend` because `total_spend` is defined earlier in the same `SELECT` list. `total_spend` in `ORDER BY` refers to the projected column. This follows the lateral column alias convention used by DuckDB, Snowflake, and MySQL. Exact lowering semantics (e.g. inline expression rewriting for Substrait) are specified by the query grammar.

#### Rule 5: ordinary Incan values when no column matches

If a bare identifier does not resolve to a query schema column, it **may** resolve as an ordinary Incan binding or value where the grammar allows a full Incan expression.

```incan
threshold = 1000

query {
    FROM orders
    WHERE .amount > threshold
}
```

`threshold` is not a column; it resolves as the surrounding binding. This is how InQL stays composable with the rest of Incan instead of becoming an isolated mini-language.

### 5. Ambiguity examples

#### 5.1 Outer binding vs query column

```incan
customer_id = "debug-value"

query {
    FROM orders
    SELECT customer_id
}
```

`customer_id` resolves to the query column, not the outer variable, because relational schema names win in relational clause positions. When an outer binding of the same name exists, the typechecker emits a warning and suggests the explicit form:

```incan
query {
    FROM orders
    SELECT .customer_id
}
```

#### 5.2 Alias after projection

```incan
query {
    FROM orders
    SELECT
        sum(.amount) as total_spend,
    WHERE total_spend > 1000
}
```

`total_spend` in `WHERE` is the projected column from the preceding `SELECT`.

#### 5.3 Joined relation stays explicit

```incan
query {
    FROM orders
    JOIN customers ON .customer_id == customers.id
    SELECT
        customer_id,
        customers.name,
}
```

Bare `customer_id` is the current schema column; `customers.name` stays explicit because it comes from a named joined relation.

### 6. Query surfaces and semantic equivalence

These are two surfaces over the same relational model. Identifier resolution rules do not change across them — only the syntactic setup changes.

1. `query {}`
    The SQL-familiar, clause-oriented surface.

2. Collection method chains
    The method-chain surface:

    ```incan
    orders
        .filter(.amount > 100)
        .group_by(.region)
        .agg(sum(.amount) as total)
    ```

**Normative:** wherever multiple surfaces are supported, identifier resolution (§§2–4) **must** be identical across them.

A future pipe-forward surface is deferred from v0.1 but **must** also share the same resolution rules when introduced.

### 7. Scope restriction: `.column` is only valid in relational positions

`.field` is **not** a general Incan expression form. It is only valid inside **relational expression positions**:

- Inside a `query {}` block (any clause position that permits a relational expression).
- As an argument to a relational operation on a `DataSet[T]` value (e.g. `.filter(.amount > 100)` — here `.amount` is in a relational argument position).
- Inside a future pipe-forward stage.

Using `.field` outside these contexts **must** be a compile-time error. This keeps the dot notation unambiguous: wherever `.field` appears, there is always a well-defined primary relation that supplies the field type.

### 8. Schema shapes

InQL needs to support several schema modes without losing its semantic model.

#### 8.1 Fully typed: `DataFrame[T]`

`DataFrame[T]` carries the full field-level schema through compilation. `.filter(.amount > 100)` can validate `amount` exists and has a compatible type; projection can produce a new typed output shape; grouping and aggregation can be validated against the schema.

When `T` is an Incan model that participates in contract semantics upstream, InQL **should** treat it first as a schema surface: query checking uses the model's fields and types; projection and output typing flow from that model shape.

#### 8.2 Open-ended: `model X with OpenEnded`

For models declared as:

```incan
model Event with OpenEnded:
    event_id: str
    occurred_at: datetime
    user_id: str
```

the declared fields remain fully typed while undeclared fields become soft-runtime references.

- closed `model X:` → declared fields are the complete schema surface
- `model X with OpenEnded:` → declared fields are the minimum guaranteed schema surface; additional runtime fields **may** exist

Operationally:

- declared field → checked normally
- undeclared field → warning plus soft-runtime field node

`OpenEnded` **should** be treated as a compiler-known marker with schema-shaping semantics, not as an ordinary user-defined behavioral trait.

#### 8.3 Dynamic: `DataFrame[Dynamic]`

For untyped DataFrames, the compiler still tracks relational plan shape, but full field resolution becomes runtime-driven.

Even for `DataFrame[Dynamic]`, the compiler **should** infer a **lower-bound structural surface**:

- the **minimum required input surface**: columns that **must** exist because the query reads or references them
- the **minimum guaranteed output surface**: columns that the query definitely preserves or creates

So if a query over `DataFrame[Dynamic]` reads `.foo` and then projects `.foo` plus `bar = some_expr(...)`, the compiler **should** be able to say: input surface is at least `{foo}`; output surface is at least `{foo, bar}`.

Operationally:

- `.column` lowers to a runtime column reference annotated with a required-field dependency
- field-not-found errors do not exist at compile time
- type-driven coercions from model metadata are not inserted

#### 8.4 Four schema shapes

Once both openness and subscription are allowed, InQL recognizes four common schema shapes:

1. **Closed local** — a plain local model. Declared fields are the complete schema surface.
2. **Open local** — a local model `with OpenEnded`. Declared fields are the minimum guaranteed schema surface.
3. **Closed subscribed or projected view** — a consumer model that projects an upstream model into an exact local view.
4. **Open subscribed or projected view** — a consumer model that projects an upstream model but remains open-ended.

This 2x2 framing separates two concerns cleanly:

- **completeness**: closed vs open-ended
- **provenance**: local vs subscribed or projected

InQL **should** be able to reason about all four shapes using the same core machinery: known guaranteed fields, openness of the schema surface, and provenance of fields where available.

#### 8.5 Schema boundary validation

When a boundary exists between untyped and typed DataFrames in pipeline wiring, the platform **should** insert a runtime schema validation check that ensures expected fields exist, types are compatible, nullability is compatible, and collects mismatches into one diagnostic.

Where lower-bound structural surfaces are available, the compiler **should** use them before runtime to detect obviously incompatible wiring and to distinguish "definitely required" columns from merely possible runtime columns.

### 9. Relationship to Incan models

InQL does not invent its own schema-definition system. It relies on Incan models as the typed schema surface for query authoring and validation.

An Incan model gives InQL:

- field names
- field types
- nullability
- structural shape for joins, projections, and outputs

InQL **must not** redefine schema language or model declarations; it consumes models as-is. Subscription and compatibility semantics (narrowing, widening, blast radius) belong in the operational and contract layers above InQL, not in InQL itself. InQL **may** surface subscription-derived schema information during typechecking and planning, but the declaration and enforcement model belongs to the contract layer.

### 10. Compilation model

InQL queries follow the same broad compiler pipeline as ordinary Incan code, with query-specific stages for relational planning:

```text
InQL source
  → parser and AST
  → typechecker (name resolution, schema flow, type inference)
  → InQL IR (relational operators, typed expressions)
  → Substrait emission
  → execution context optimization and execution
```

The user-facing query model stays stable while the lower layers evolve: AST shape may change, IR may change, Substrait maturity may improve, and execution backend implementations may change. The author should still be expressing the same typed data intent.

### 11. Layer boundaries

The boundaries matter:

- **Incan** owns the shared language, type system, modules, traits, compile-time safety, and the core `model` type definitions that queries use as schemas.
- **InQL** owns typed data logic, relational semantics, schema flow, and logical planning.
- **Execution layer** owns runners, workflows, adapters, quality enforcement, contract enforcement, and operational workload semantics.
- **Semantic layer** owns governed business meaning and semantic-serving abstractions.

InQL sits above the language core and below execution, semantics, and product layers.

## Design details

### Cross-RFC consistency

- All companion InQL RFCs **must** stay consistent with this document for naming forms, current query schema behavior, resolution order, schema shapes, and the relational-position restriction on `.column` described in §§2–7.
- Extensions in companion RFCs **must not** contradict these rules without an explicit amendment to this RFC.
- Any future authoring surface (including pipe-forward), when introduced, **must** desugar to the same semantic model and **must not** change identifier resolution rules.

### Compatibility

- Naming and resolution rules are foundational; breaking changes **must** go through a dedicated RFC amendment.
- New schema shapes **should** be additive.

## Alternatives considered

- **Single global lexical scope for bare names in queries** — rejected: breaks SQL-familiar `SELECT` / `ORDER BY` patterns and alias flow.
- **`.column` as a general Incan expression form** — rejected: without a well-defined primary relation in scope, `.field` has no unambiguous meaning. Restricting it to relational positions keeps the notation precise and allows the compiler to always resolve the field type statically.
- **Three authoring surfaces in v0.1** (including pipe-forward) — deferred: v0.1 focuses on `query {}` and method chains; pipe-forward is a syntactic convenience that does not unlock new capability and can be added in a later version without breaking the semantic model.
- **Separate resolution rules per surface** — rejected: would create semantic drift between `query {}` and method chains.

## Drawbacks

- Large foundational document that many companion RFCs depend on; amendments need care.
- Open-ended and dynamic schema modes add compiler complexity even in v0.1.
- The single-model constraint across surfaces requires implementation discipline to avoid surface-specific shortcuts that break semantic equivalence.

## Layers affected

- **Parser / AST**: relational surfaces must preserve the distinct naming forms (`.column`, `relation.column`, bare identifiers, and ordinary Incan expressions) and their clause-sensitive contexts.
- **Typechecker**: resolution order, current query schema flow, `SELECT` alias publication, join alias handling, and schema-shape behavior (typed, open-ended, dynamic) must remain consistent across surfaces.
- **IR / lowering**: both authoring surfaces must lower to one relational semantic model without changing identifier meaning or schema-flow behavior.
- **LSP**: relational positions need resolution-aware highlighting, diagnostics, and completions that reflect the same naming rules as the compiler.
- **Documentation**: companion InQL RFCs and contributor docs must describe the same foundational naming, schema, and boundary rules.

## Design Decisions

- **`.column` scope restriction:** `.field` is only valid in relational expression positions (`query {}` clauses, method-chain relational arguments, and future pipe-forward stages). It is a compile-time error outside those positions. This keeps the dot notation unambiguous and ensures a primary relation is always in scope when `.field` is used.
- **Lateral column aliases in `SELECT`:** an alias defined in a `SELECT` list is visible to subsequent expressions in the same list, in order (lateral column alias semantics). This follows the convention of DuckDB, Snowflake, and MySQL. An alias is not visible to expressions that precede it in the list. Implementations **must** rewrite dependent expressions (e.g. inline substitution) before Substrait lowering, since Substrait projection nodes do not natively support lateral alias references.
- **Pipe-forward deferred to v0.2+:** InQL RFC 005 specifies pipe-forward as an optional surface with the same resolution rules. It is not part of v0.1 scope. This RFC (§6) states the invariant that pipe-forward **must** share §§2–4 resolution rules.
- **Method-chain API scope:** deferred to InQL RFC 001, which defines the `DataSet[T]` operation surface. This RFC does not mandate a particular chain API shape.
- **`HAVING` keyword:** not InQL syntax. Post-`SELECT` filtering uses a second `WHERE` clause (clause ordering details — InQL RFC 003).
- **Schema shape priority for v0.1:** fully typed (`DataFrame[T]`) is the primary mode. Open-ended (`with OpenEnded`) and dynamic (`DataFrame[Dynamic]`) remain part of the normative model in this RFC, but any implementation that does not yet support a permitted behavior **must** reject it explicitly rather than silently reinterpreting or weakening these semantics.
- **Open-ended model marker syntax:** `model X with OpenEnded` is the normative spelling in this RFC for v0.1. If Incan changes how openness is declared, this RFC **must** be amended in lockstep; implementations follow the Incan language as shipped and update InQL specification text accordingly.
- **External / catalog-bound models:** resolution of `model X = external("catalog://...")` (or equivalent) and how such models surface in query schemas is **out of scope for InQL RFC 000** and for v0.1 closure. It depends on Incan contract and catalog semantics and **may** be specified in a later InQL RFC once those foundations exist.
- **Shadowing warning and LSP:** when a bare name matches both a query column and an outer Incan binding, query semantics and the diagnostic (warn; suggest `.column`) are normative above. Whether the LSP offers quick-fixes (e.g. rewrite to `.column`, rename outer binding) is **tooling policy** in the Incan LSP and is not prescribed here; any such fixes **must** preserve the resolution rules in §§2–4.
