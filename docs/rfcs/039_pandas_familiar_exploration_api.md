# InQL RFC 039: Pandas-familiar exploration API

- **Status:** Draft
- **Created:** 2026-05-30
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (language specification, naming, schema shapes, and relational positions)
  - InQL RFC 001 (dataset carriers and method-chain API surface)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
  - InQL RFC 005 (pipe-forward relational syntax)
  - InQL RFC 012 (unified scalar expression surface)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines a pandas-familiar exploration API for InQL dataset carriers. The API provides dictionary-like column access through `data["column"]`, projection through `data[["a", "b"]]`, boolean filtering through `data[predicate]`, and a small set of familiar method aliases such as `where`, `assign`, `groupby`, `sort_values`, and `head`. These forms are ergonomic aliases over InQL's existing typed relational model; they must not introduce pandas row-indexing, mutable frame, eager execution, or index-alignment semantics.

## Core model

1. InQL dataset carriers may behave dictionary-like for columns.
2. InQL dataset carriers must not behave sequence-like for rows unless a future RFC defines row-position semantics explicitly.
3. Bracket column access returns a bound scalar column expression, not a materialized Series-like value.
4. Bracket projection and bracket filtering lower to the same relational operators as `DataSet[T]` method chains and `query {}` blocks.
5. Pandas-familiar method names are aliases over existing InQL relational operations, not alternate execution contracts.

## Motivation

InQL's cleaner relational APIs are good for production authoring, but data exploration has a different adoption problem. Authors coming from pandas need a familiar fallback surface for the workflows they reach for reflexively: selecting a column, filtering a frame, projecting a few columns, adding a derived column, grouping, sorting, and previewing rows. If InQL only exposes its cleanest API, it forces those users to learn InQL before they can inspect data, which is a poor fit for exploratory work.

At the same time, copying pandas wholesale would be a design error. Pandas carries semantics that do not fit InQL's typed, planned, backend-neutral model: mutable frames, eager local execution, row-position indexing, index alignment, view-versus-copy behavior, dynamic dtype coercion, and a Series object model with local row values. Those features are familiar, but they would make InQL less coherent if imported as language semantics.

This RFC takes the narrower path: provide familiar surface forms where they map cleanly to InQL's relational model, and reject the parts of pandas that would require a different data model.

## Goals

- Define bracket column access on `DataSet[T]` carriers through `data["column"]`.
- Define bracket projection through `data[["a", "b"]]`.
- Define bracket filtering through `data[predicate]` where `predicate` is a boolean scalar expression bound to the same dataset relation.
- Define pandas-familiar aliases for common method-chain operations where semantics already exist in InQL.
- Preserve InQL's typed schema flow, boundedness constraints, scalar expression model, Prism planning, Substrait lowering, and execution boundaries.
- Require clear diagnostics for row-indexing spellings that pandas users might try but InQL does not support.

## Non-Goals

- Achieving pandas API compatibility.
- Introducing row-position indexing through `data[0]`, `data[1:10]`, `.iloc`, or equivalent forms.
- Introducing pandas index labels, index alignment, multi-index behavior, or Series alignment semantics.
- Introducing mutable assignment such as `data["x"] = expr` or `inplace=true`.
- Defining pandas view-versus-copy behavior.
- Making bracket access materialize local row values.
- Defining all exploratory helpers such as `describe`, `sample`, rich display, plotting, or notebook integration.
- Replacing the cleaner InQL `DataSet[T]` method surface or `query {}` blocks.

## Guide-level explanation (how authors think about it)

Authors can use bracket syntax when they want a familiar DataFrame-like way to refer to columns:

```incan
high_value = orders[orders["amount"] > 100]
```

`orders["amount"]` is not a pandas Series. It is a column expression bound to `orders`, equivalent in meaning to a relation-scoped column reference. The comparison produces a boolean scalar expression, and `orders[...]` with that boolean expression filters rows.

Authors can project columns with a list of names:

```incan
order_amounts = orders[["order_id", "customer_id", "amount"]]
```

This is a projection over the same relation. The output preserves column order from the list and follows the same schema rules as the corresponding InQL projection operation.

Authors can use familiar method aliases for common exploration flows:

```incan
enriched = (
    orders
        .where(orders["amount"] > 100)
        .assign("gross_amount", orders["amount"] * 1.21)
)

summary = (
    enriched
        .groupby(["region"])
        .agg([sum(enriched["gross_amount"]) as total_gross, count() as order_count])
        .sort_values("total_gross", ascending=false)
        .head(20)
)
```

Those names are familiar, but the semantics are still InQL. `where` is `filter`, `assign` is derived-column projection, `groupby` is `group_by`, `sort_values` is `order_by`, and `head` is `limit`. They do not execute eagerly unless the surrounding execution context materializes the result.

Row indexing is intentionally rejected:

```incan
first = orders[0]        # compile-time error
preview = orders[1:10]   # compile-time error
row = orders.iloc[0]     # not part of this RFC
```

Authors should use `head(n)` or `limit(n)` for preview-shaped relational limiting, and window functions for ordered row-number analytics.

## Reference-level explanation (precise rules)

### Applicability

The pandas-familiar exploration API applies to values whose type conforms to `DataSet[T]` as defined by InQL RFC 001. The API must preserve the concrete carrier kind where the equivalent relational operation would preserve it. For example, filtering a `LazyFrame[T]` produces a `LazyFrame[T]`; filtering a `DataStream[T]` must follow the same boundedness and capability constraints as `filter(...)` on `DataStream[T]`.

### Bracket column access

For a dataset value `data: DataSet[T]`, `data["name"]` denotes a scalar column expression bound to `data` and the field named `name`.

The key expression in the strongly typed form must be a string literal or another compile-time-known string value whose value can be checked against the schema. If `T` is a closed local model, the named field must exist or compilation must fail. If `T` is open-ended or dynamic, field lookup must follow the schema-shape rules from InQL RFC 000, including warnings for undeclared open-ended fields where that RFC requires them.

`data["name"]` must not materialize a local column, must not produce a Series-like object, and must not imply that `data` is locally available. It is a typed relational scalar expression that may be used in relational expression positions and in ordinary Incan expression positions that consume InQL scalar expressions.

Bound column expressions must preserve relation provenance. A predicate built from `orders["amount"]` may filter `orders`, but must not be accepted as a filter predicate for an unrelated dataset unless an explicit relational operation such as a join establishes the appropriate relation context.

### Bracket projection

For a dataset value `data: DataSet[T]`, `data[["a", "b"]]` denotes an ordered projection containing the named columns in the list.

The projection list in the strongly typed form must be a list literal or another compile-time-known list of string column names. Each projected name must be checked using the same schema-shape rules as bracket column access. The output schema must contain the projected columns in the order supplied by the list.

Duplicate projected names must be rejected unless a future RFC defines duplicate-column schema semantics. Pandas permits duplicate column labels, but InQL's typed relation model requires deterministic field names.

Bracket projection must lower to the same relational projection semantics as the corresponding `DataSet[T]` projection surface or `query { SELECT ... }` form. It must not materialize data.

### Bracket filtering

For a dataset value `data: DataSet[T]`, `data[predicate]` denotes row filtering when `predicate` is a scalar expression whose result type is `bool` and whose relation provenance is compatible with `data`.

Bracket filtering must lower to the same operation as `data.filter(predicate)`. Null handling, boundedness checks, backend capability checks, and diagnostics must be identical to the equivalent `filter(...)` operation.

`data[predicate]` must not accept materialized boolean arrays, local lists of booleans, row masks, or pandas-like index-aligned masks unless a future RFC defines those concepts explicitly.

### Rejected bracket forms

The following forms must produce compile-time diagnostics:

- `data[0]`
- `data[1:10]`
- `data[-1]`
- `data[mask]` where `mask` is a materialized local boolean list or array rather than an InQL scalar expression
- any bracket form whose key cannot be resolved as a string column key, compile-time-known list of string column keys, or boolean scalar expression

Diagnostics should explain that InQL supports dictionary-like column access and relational filtering, not pandas row indexing.

### Familiar method aliases

The initial pandas-familiar method alias set is:

| Alias | Canonical InQL operation | Required semantics |
| ----- | ------------------------ | ------------------ |
| `where(predicate)` | `filter(predicate)` | Filter rows using a boolean scalar expression. |
| `assign(name, expr)` | `with_column(name, expr)` | Add or replace one derived column. |
| `groupby(columns)` | `group_by(columns)` | Group using column names or scalar grouping expressions accepted by the canonical operation. |
| `sort_values(by, ascending=true)` | `order_by(...)` | Sort by one or more named columns or ordering expressions. |
| `head(n)` | `limit(n)` | Apply a relational row limit with the same carrier and boundedness rules as `limit`. |

These aliases must not alter the plan produced by the canonical operation except for source-location metadata used in diagnostics. If a canonical operation is unavailable for a carrier because of boundedness rules, the alias must be unavailable for the same reason.

### Method alias arguments

String column names accepted by familiar aliases must resolve using the same schema-shape rules as bracket column access. Scalar expression arguments must use the unified scalar expression model from InQL RFC 012. Aggregate arguments used after `groupby(...)` must use the aggregate-measure rules defined by the relevant aggregate RFCs.

`sort_values("amount")` must be equivalent to ordering by the `amount` column in ascending order. `sort_values("amount", ascending=false)` must be equivalent to ordering by the same column in descending order. Multi-column sort arguments may be supported through a compile-time-known list of names or ordering expressions, provided they lower to the same ordering model as `order_by(...)`.

### Interaction with `query {}` and pipe-forward

This RFC does not add new `query {}` clause syntax. Bracket access and familiar method aliases must remain semantically equivalent to `query {}` blocks that express the same relational operations.

If pipe-forward from InQL RFC 005 is implemented, it must not define different pandas-familiar semantics. A pipe-forward stage that corresponds to `where`, projection, grouping, sorting, or limiting must lower to the same relational model as bracket and method-chain forms.

## Design details

### Syntax

This RFC requires the Incan language and InQL vocabulary integration to recognize bracket access on dataset carriers for the forms specified above. The bracket syntax is intentionally overloaded only by key shape: string key for column expression, list of string keys for projection, and boolean scalar expression for filtering.

The RFC does not require `.loc`, `.iloc`, attribute-style column access, or assignment syntax.

### Semantics

The semantic distinction is column dictionary access versus row sequence access. `DataSet[T]` carriers may be indexed by column name because the schema is part of the relational type. They may not be indexed by row position because row order is not an inherent property of an unordered relation and because InQL execution may be lazy, distributed, streamed, or backend-planned.

`head(n)` is a relational limit, not proof of stable row order. Authors who need deterministic preview order should combine `sort_values(...)` or `order_by(...)` with `head(...)`.

### Interaction with Incan `model` types and schema shapes

Closed model schemas provide the strongest checks for bracket column access and projection. Open-ended and dynamic schemas follow the existing InQL schema-shape contract: declared fields remain checked, while undeclared fields may be allowed with warnings or dynamic typing where InQL RFC 000 permits that behavior.

### Compatibility / migration

This RFC is additive. Existing `DataSet[T]` method chains and `query {}` blocks remain valid and remain the canonical semantic reference.

The main compatibility risk is expectation compatibility, not source compatibility. Authors who expect full pandas behavior must receive clear diagnostics and documentation explaining which familiar forms InQL supports and which ones are deliberately absent.

## Alternatives considered

- **Expose only the clean InQL API.** Rejected because exploration workflows need familiarity, and forcing pandas users through only the cleanest InQL surface raises the cost of first use.
- **Implement broad pandas compatibility.** Rejected because pandas row indexes, eager Series values, mutable assignment, and index alignment conflict with InQL's typed relational planning model.
- **Support `.loc` and `.iloc` initially.** Rejected because both imply an index or positional row model that this RFC intentionally excludes.
- **Use attribute column access such as `data.amount`.** Rejected for the initial surface because it collides with ordinary methods and carrier properties more readily than string-key access.
- **Make `data["name"]` return a Series-like value.** Rejected because it would imply local materialization and row-level value access instead of a backend-neutral scalar expression.

## Drawbacks

- Familiar names can create false expectations about pandas parity unless diagnostics and docs are explicit.
- Bracket syntax adds an overloaded authoring surface that tooling must explain carefully.
- Method aliases duplicate parts of the canonical API, increasing documentation and completion surface area.
- Rejecting row indexing is correct for InQL's model but will still surprise some pandas users.
- Compile-time-known projection lists may feel less flexible than pandas during ad hoc exploration, especially for dynamic column selection.

## Layers affected

- **InQL specification** — RFCs 000, 001, 003, 005, and 012 must stay coherent with bracket access, relation provenance, scalar expression typing, and alias semantics.
- **InQL library package** — public `.incn` APIs must expose the familiar aliases and tests must cover equivalence to canonical operations where those aliases are library-level methods.
- **Incan compiler** — parser, typechecker, lowering, diagnostics, formatter, and LSP may need dataset-aware bracket handling and source spans for useful errors.
- **Execution / interchange** — Prism, Substrait lowering, sessions, and backend adapters must receive the same logical operators as canonical InQL operations; this RFC must not introduce a separate execution path.
- **Documentation** — reference and explanation docs should present the pandas-familiar API as an exploration facade and document the rejected pandas semantics directly.

## Unresolved questions

- Should `assign(...)` require exactly one `(name, expr)` pair initially, or should it also support a list of assignments once projection assignment syntax is settled?
- Should `groupby(...)` accept only strings in the familiar surface, or should it accept every scalar grouping expression accepted by `group_by(...)` from day one?
- Should dynamic runtime `list[str]` projection be permitted for dynamic carriers, and if so what output schema should the typechecker assign?
- Should `merge(...)` be included in the initial alias set, or deferred until join output typing and pandas-style join argument spelling are specified more completely?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
