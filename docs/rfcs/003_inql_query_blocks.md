# InQL RFC 003: `query {}` blocks — syntax, typing, Substrait

- **Status:** Implemented
- **Created:** 2026-03-22
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — naming and query schema; **must** stay consistent)
  - InQL RFC 001 (dataset types — **prerequisite**; `FROM` sources must conform to `DataSet[T]`)
  - InQL RFC 002 (Apache Substrait — **normative `Rel`-level contract** for lowering)
- **Issue:** [InQL #4](https://github.com/dannys-code-corner/InQL/issues/4)
- **RFC PR:** [InQL #59](https://github.com/dannys-code-corner/InQL/pull/59)
- **Written against:** Incan v0.3
- **Shipped in:** InQL v0.1

## Summary

This RFC specifies the **`query { ... }`** expression: grammar, typechecking (including clause-level use of `.column`, `relation.column`, bare identifiers, and aggregate rules), vocabulary activation for the `query` keyword (InQL package as dependency), and lowering to Apache Substrait. InQL also accepts the expression-position colon spelling `query:` for consistency with Incan vocabulary declarations. Naming-form semantics and current query schema are defined in InQL RFC 000; this RFC **must** remain consistent with that document. It depends on InQL RFC 001: `FROM` sources **must** conform to InQL RFC 001's `DataSet[T]` trait (`DataFrame[T]`, `LazyFrame[T]`, or `DataStream[T]`) so that `T` supplies fields for resolution. InQL RFC 002 owns the Substrait `Rel` and expression contract, mapping catalog, and read vs binding boundaries; this RFC **must** conform to InQL RFC 002 for serialized plan semantics. `SELECT DISTINCT` is part of the minimum clause surface defined here.

## Motivation

A SQL-familiar surface inside Incan improves readability and enables compile-time validation of relational work against `model` schemas. `query {}` is the checked authoring form; it lowers to operations on `DataSet` values (InQL RFC 001) and/or directly to Substrait (InQL RFC 002) for portability.

## Goals

- Specify `query { ... }` as a single expression whose body is an ordered sequence of clauses with unambiguous grammar.
- Typecheck relational positions: `.column` on the primary `FROM` relation; `relation.column` for joins; bare names per schema-first rules below.
- Lower a checked `query {}` tree to Apache Substrait as the normative interchange for supported operators; document gaps, extensions, and unsupported nodes. Lowering **must** conform to InQL RFC 002.
- Integrate with InQL RFC 001 so `FROM <expr>` resolves to a type conforming to `DataSet[T]` with known `T`.
- Enable end-to-end batch execution: `query {}` → Substrait → execution context.

## Non-Goals

- Defining the `DataSet[T]` trait, `DataFrame` / `LazyFrame` / `DataStream` types, or their method APIs — InQL RFC 001.
- Pipe-forward (`|>`) relational syntax — InQL RFC 005 (must stay consistent with InQL RFC 000).
- Substrait `Rel`-level mapping catalog, extension URI policy, read-root vs binding boundaries — InQL RFC 002.
- Execution context, session, DataFusion — InQL RFC 004.
- Cluster execution — out of scope for InQL.

## Guide-level explanation

```incan
from pub::inql import DataFrame, avg, count, desc, sum
from models import Order, OrderSummary

def summarize_orders(orders: DataFrame[Order]) -> DataFrame[OrderSummary]:
    return query {
        FROM orders
        WHERE .status == "completed"
        GROUP BY .region
        SELECT
            .region as region,
            count() as order_count,
            sum(.amount) as total_revenue,
            avg(.amount) as avg_order_value,
        ORDER BY desc(.total_revenue)
    }
```

The compiler checks `.status`, `.amount`, and `GROUP BY` / `SELECT` consistency. The `DataFrame[OrderSummary]` return type records the intended output row model; full field/type compatibility validation against annotated output models is tracked as schema-validation follow-up work. The checked tree lowers to Substrait (InQL RFC 002); execution uses the execution context.

## Reference-level explanation

### Packaging and activation

- Projects that depend on InQL **must** obtain `query` through library-driven vocabulary activation in the host compiler.
- A compilation unit with InQL active **must** parse `query { ... }` as specified here. It **may** also accept
  expression-position `query:` as an equivalent spelling.
- Aggregate helpers such as `count`, `sum`, `avg`, `min`, and `max` are library symbols, instead of ambient builtins. Examples in this RFC import them from the `pub::inql` facade; implementations **must** provide an equivalent importable surface for aggregate functions used in relational expressions.

### `FROM` and relation to InQL RFC 001

- `FROM <expr>` establishes the primary relation for `.column`. `<expr>` **must** typecheck as a type conforming to `DataSet[T]` per InQL RFC 001 (`DataFrame[T]`, `LazyFrame[T]`, or `DataStream[T]`).
- `T` **must** supply fields for `.name` lookup.

### Primary relation and joins

- `JOIN` introduces named secondary relations. Joined columns **must** use `relation.column` unless bare names are unambiguous per grammar.
- `.column` after `JOIN` **must** refer to the primary `FROM` relation only.

### Current query schema and bare identifiers

Naming forms, schema evolution, and resolution precedence — InQL RFC 000. This subsection lists relational expression positions for this RFC only.

Inside relational expression positions (`WHERE`, `JOIN ON`, `GROUP BY`, `ORDER BY`, `SELECT`, window specs):

1. `.column` → primary relation's `T` fields.
2. `relation.column` → named join relation.
3. Bare identifier → current query schema first, then lexical Incan binding where permitted.

### Expression operators

Relational expression bodies use ordinary Incan expression operators and lower them into InQL's public helper surface. Implementations must treat `left == right`, `left != right`, `left < right`, `left <= right`, `left > right`, and `left >= right` as equivalent to `eq(left, right)`, `ne(left, right)`, `lt(left, right)`, `lte(left, right)`, `gt(left, right)`, and `gte(left, right)` respectively. Arithmetic operators lower through `add`, `sub`, `mul`, `div`, and `modulo`; boolean and unary operators lower through their helper equivalents such as `and_`, `or_`, `not_`, and `neg`.

Inclusive comparison helpers are named `lte` and `gte`; `le` and `ge` are not part of the public helper surface. Single `=` is not a predicate equality operator in query expressions. Equality uses `==`; `=` remains reserved for assignment/binding positions such as named window declarations.

### `SELECT` and alias publication

- `SELECT` defines a projection boundary; output columns become the schema for later clauses in the block.
- **Lateral column aliases**: an alias defined in a `SELECT` list **may** be referenced by subsequent expressions in the same list, in order. An alias is visible to expressions that follow it in the list; it **must not** be visible to expressions that precede it. Implementations **must** rewrite dependent expressions (e.g. inline substitution) before lowering to Substrait, since Substrait projection nodes do not natively support lateral alias references.

### Aggregates

- Under `GROUP BY`, `SELECT` references **must** be grouped or aggregated; illegal mixing **must** error.
- Aggregate function calls in relational expressions **must** resolve through imported library symbols (for example `from pub::inql import count, sum, avg`). The compiler **must not** treat `count`, `sum`, `avg`, `min`, or `max` as implicitly in scope ambient names.
- This RFC defines the minimum required aggregate-function surface and import model for `query {}`; it is not an exhaustive catalog of all InQL functions. Additional functions require additive library evolution or follow-up RFCs that do not change the semantics of the required set defined here.
- `SELECT DISTINCT` **must** be supported as a projection modifier in the minimum `query {}` surface. It removes duplicate rows from the projected schema and lowers using the distinct-row contract defined by InQL RFC 002.

### Clause inventory (minimum)

This RFC **must** require at least:

- `FROM`, `WHERE`, `SELECT`, `GROUP BY`, `ORDER BY`, `LIMIT`
- inner `JOIN ... ON`, `LEFT JOIN ... ON`
- `EXPLODE <expr> as <alias>` for list-valued expressions
- `WINDOW BY <alias> = <window expression>` for ranked/windowed forms in scope

Post-`SELECT` filters on the projected schema use `WHERE` again (a `WHERE` clause ordered after `SELECT` in the block). `HAVING` is **not** InQL syntax and **must not** be introduced.

### Emission

- **Primary interchange:** checked `query {}` **must** lower to Substrait for operators in scope; document limitations. The normative `Rel`-level mapping, extension rules, and read-root policy are InQL RFC 002; this RFC **must not** contradict InQL RFC 002.
- **Batch execution** **must** be achievable through the execution context consuming Substrait (or a defined handoff).
- Dialect-specific textual renderings **may** exist for inspection or debugging, but they are non-normative and **must not** become the portable interchange or an alternate execution contract for `query {}`.

### Lowering to InQL RFC 001 operations (optional path)

- Implementations **may** lower `query {}` to InQL RFC 001 trait/method calls on `DataSet[T]` for execution or optimization; semantics **must** match Substrait lowering.

## Design details

### Interaction with Incan

- Consumes `model` definitions for `T`; does not redefine schema language.

### Compatibility

- New clauses **should** be additive; breaking grammar changes need migration notes.

## Alternatives considered

- **SQL strings only** — rejected (lose static checking).
- **Macros-only query** — rejected (Incan lacks sufficient macro support).

## Drawbacks

- Large compiler surface (parser, checker, Substrait mapping, LSP).
- Substrait coverage vs expressiveness tension.

## Layers affected

- **Parser / AST** for `query {}` and clauses.
- **Typechecker** for relation registry, schema flow, aggregates.
- **IR / lowering** to Substrait (conforming to InQL RFC 002).
- **LSP** inside `query {}`.
- **InQL package**: vocabulary registration for `query`.

## Design Decisions

- **Lateral column aliases in `SELECT`**: an alias defined in a `SELECT` list is visible to subsequent expressions in the same list, in order (lateral column alias semantics). This follows the convention of DuckDB, Snowflake, and MySQL. Implementations must rewrite dependent expressions before Substrait lowering (inline substitution), since Substrait projection nodes are flat. An alias is not visible to expressions that precede it in the list.
- **Shadowing warning**: when a bare name in a relational clause position resolves to a query column that shadows an outer Incan binding of the same name, the typechecker **should** emit a warning and suggest the `.column` form to make relational intent explicit. Writing `.column` explicitly suppresses the warning. The warning is informational — the resolution rule (column wins) still holds. Example message: `bare name \`customer_id\` shadows outer binding; use \`.customer_id\` to make relational intent explicit`.
- **Return type inference**: `query {}` infers the output schema from the `SELECT` list. The result preserves the collection kind of the `FROM` source: a `query {}` over a `DataStream` yields a `DataStream`; over a `LazyFrame` yields a `LazyFrame`; over a `DataFrame` yields a `DataFrame`. Explicit type annotation at the call site is optional but recommended for documentation.
- **Post-`SELECT` clause ordering**: the canonical clause order is `FROM` → `JOIN` → `WHERE` → `GROUP BY` → `SELECT` → `WHERE` (post-`SELECT` filter) → `ORDER BY` → `LIMIT`. `HAVING` is not InQL syntax (InQL RFC 000). Exact diagnostic wording for ordering violations is an implementation detail.
- **Aggregate minimum set**: the initial implementation requires at least `count`, `sum`, `avg`, `min`, `max`. Window functions (`WINDOW BY` and ranked expressions) are part of the clause inventory but their detailed builtin set evolves during implementation. `DataStream` source restrictions follow InQL RFC 001's static capability gating: operations requiring unbounded state are statically rejected.
- **Aggregate function scope:** the minimum aggregate set is exposed through an importable InQL facade (examples use `pub::inql`). These names are ordinary imported symbols that gain aggregate meaning in aggregate-capable relational positions; they are not special ambient builtins.
- **`IN` clause**: not part of the RFC003 clause grammar. A separate RFC is required before introducing `IN` as a query-block clause operator, and its RHS contract must conform to `DataSet[T]` with a compatible schema.
- **Substrait version and mapping catalog**: InQL RFC 002 owns pinning policy, the north-star operator → `Rel` catalog, and extension URI requirements; the exact revision shipped with a toolchain is documented in release artifacts alongside the implementation.
- **Alternate surfaces**: pipe-forward is InQL RFC 005; method chains are InQL RFC 001. This RFC does not mandate alternative surfaces in the initial implementation.
- **Minimum join surface:** the required v0.1 clause inventory includes `JOIN ... ON` (inner join) and `LEFT JOIN ... ON`. `RIGHT` and `FULL OUTER` joins are not part of the required RFC003 minimum; adding them requires an additive extension that preserves the current join semantics.
- **`SELECT DISTINCT`:** `query {}` **must** support `SELECT DISTINCT` in the minimum clause surface. It is the canonical clause-level spelling for duplicate elimination in this surface; method-chain APIs may expose equivalent operations, but they do not replace the query-surface keyword.
- **Ordering syntax:** the implemented v0.1 query surface uses ordering helpers such as `asc(.amount)` and
  `desc(.amount)` rather than postfix SQL tokens such as `.amount DESC`.
