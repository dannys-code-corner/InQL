# InQL RFC 001: Dataset types and carriers (`DataSet[T]`)

- **Status:** In Progress
- **Created:** 2026-03-22
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — naming, schema shapes, layer boundaries)
  - Incan compiler — static capability gating enforcement: [incan#187](https://github.com/dannys-code-corner/incan/issues/187)
  - InQL follow-up when enforcement lands: [InQL #10](https://github.com/dannys-code-corner/InQL/issues/10)
  - InQL aggregate helper semantics follow-up: [InQL #23](https://github.com/dannys-code-corner/InQL/issues/23)
- **Issue:** [InQL #2](https://github.com/dannys-code-corner/InQL/issues/2)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC specifies the **dataset type hierarchy** for InQL: the traits and concrete types that carry schema-parameterized tabular data through relational pipelines. The hierarchy is rooted in the **`DataSet[T]`** trait, split into **`BoundedDataSet[T]`** (finite extent) and **`UnboundedDataSet[T]`** (streaming/unbounded), with three concrete types: **`DataFrame[T]`** (materialized/eager), **`LazyFrame[T]`** (deferred plan), and **`DataStream[T]`** (streaming). The bounded/unbounded split enables **static capability gating**: operations that require unbounded state are rejected at compile time when the target is unbounded, without requiring a separate streaming API. This RFC also defines the **relational operation API** on `DataSet[T]` and the **execution backend boundary** so implementations can delegate without exposing engine internals as the author contract.

## Core model

1. **`DataSet[T]`** is the root trait — any schema-parameterized tabular data whose row shape is an Incan `model` `T`.
2. **`BoundedDataSet[T]`** extends `DataSet[T]` — data with a finite, known extent. All relational operations are allowed.
3. **`UnboundedDataSet[T]`** extends `DataSet[T]` — data from a streaming or unbounded source. Operations requiring unbounded state **must** be rejected at compile time.
4. **`DataFrame[T]`** implements `BoundedDataSet[T]` — materialized/eager result; always bounded.
5. **`LazyFrame[T]`** implements `BoundedDataSet[T]` — deferred plan over a bounded source; the workhorse for batch pipelines.
6. **`DataStream[T]`** implements `UnboundedDataSet[T]` — streaming specialization; unbounded.

## Motivation

Typed pipelines need a first-class carrier for columnar data indexed by `T`. Without `DataSet[T]`, relational authoring surfaces would lack a stable primary relation and schema flow for `FROM`-style entry points. The **bounded/unbounded** distinction — inspired by Spark Structured Streaming's principle that a stream is an unbounded table — must be expressed at the **type level** so the compiler can enforce streaming constraints statically rather than at runtime. An intermediate trait layer (`BoundedDataSet` / `UnboundedDataSet`) gives authors clean type signatures for consumers that accept "any batch data" or "any streaming data" without listing concrete types.

## Goals

- Specify the **trait hierarchy**: `DataSet[T]` → `BoundedDataSet[T]` / `UnboundedDataSet[T]` → concrete types.
- Require `T` to be carried from Incan `model` definitions (or an equivalent fixed field bundle) for strongly typed mode.
- Define **`LazyFrame[T]`** as the universal deferred plan type for batch relational work.
- Define **`DataFrame[T]`** as the materialized/eager result — always bounded; the product of collecting or executing a `LazyFrame`.
- Define **`DataStream[T]`** as the streaming specialization: same operation API through `DataSet[T]`, but unbounded, enabling compile-time constraint enforcement.
- Define **static capability gating** through the trait hierarchy: `BoundedDataSet` → all operations; `UnboundedDataSet` → unbounded-state operations rejected; `DataSet` → most restrictive (because the concrete kind may be unknown).
- Specify the **relational operation API** on `DataSet[T]` as the programmatic relational surface (implementations **may** share a lowering path with other authoring surfaces; that is outside the scope of this RFC).
- Specify an **execution backend boundary**: materialize, run plan, or hand off Substrait / IR to a consumer — without mandating a single engine.

## Non-Goals

- Normative naming rules (four naming forms, current query schema, resolution order) — InQL RFC 000.
- Apache Substrait `Rel`-level mapping and extension policy — InQL RFC 002.
- Clause-based relational grammar, aggregate rules, Substrait lowering from that surface — InQL RFC 003.
- Execution context, session, DataFusion — InQL RFC 004.
- Pipe-forward (`|>`) grammar — InQL RFC 005 (not in v0.1 scope).
- Cluster-scale scheduling, shuffle, distributed fault tolerance — orchestration layer.
- Drop-in API compatibility with Apache Beam, Flink, or Spark SDKs.

## Guide-level explanation

Authors import dataset types from the InQL package and parameterize with a `model`:

```incan
from pub::inql import LazyFrame
from models import Order

def load_orders() -> LazyFrame[Order]:
    ...
```

They compose data using methods exposed through the `DataSet[T]` trait:

```incan
from pub::inql import LazyFrame
from models import Order

def high_value_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return orders.filter(.amount > 100)
```

Authors can derive computed columns through `with_column(...)`:

```incan
from pub::inql import LazyFrame
from pub::inql.functions import col, int_expr, mul
from models import Order

def enrich_orders(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    return orders.with_column("amount_x2", mul(col("amount"), int_expr(2)))
```

Because `DataStream[T]` shares the same operation API, streaming code looks identical — only the type signature changes:

```incan
from pub::inql import DataStream
from models import Event

def important_events(events: DataStream[Event]) -> DataStream[Event]:
    return events.filter(.severity == "critical")
```

### Type signature levels

The trait hierarchy gives authors three levels of specificity:

```incan
from pub::inql import DataSet, BoundedDataSet, UnboundedDataSet
from models import Order, Event

# Accepts any carrier — generic utilities
def row_count[T](data: DataSet[T]) -> int:
    ...

# Batch only — Parquet writers, batch sinks
def write_parquet(data: BoundedDataSet[Order]) -> None:
    ...

# Streaming only — Kafka sinks, event processors
def write_to_kafka(events: UnboundedDataSet[Event]) -> None:
    ...
```

And two levels of concrete-type specificity:

```incan
from pub::inql import DataFrame, LazyFrame, DataStream
from models import Order, Summary, Event, Alert

# Materialized data in hand
def inspect(data: DataFrame[Order]) -> None:
    ...

# Deferred plan — compose before execution
def build_pipeline(orders: LazyFrame[Order]) -> LazyFrame[Summary]:
    ...

# Streaming specifically
def process_stream(events: DataStream[Event]) -> DataStream[Alert]:
    ...
```

## Reference-level explanation

### Packaging

- The dataset types and traits in this RFC **must** be exposed from a buildable Incan library package with public exports.
- This RFC **may** require vocabulary only for symbols strictly needed for the dataset API surface; vocabulary for other InQL authoring surfaces is a separate concern.

### Type hierarchy

```text
DataSet[T]                       (root trait — any tabular data)
├── BoundedDataSet[T]            (trait — finite extent)
│   ├── DataFrame[T]             (concrete — materialized/eager)
│   └── LazyFrame[T]             (concrete — deferred plan, bounded source)
└── UnboundedDataSet[T]          (trait — streaming/unbounded)
    └── DataStream[T]            (concrete — streaming)
```

- **`DataSet[T]`** is the root trait. All relational operations are defined here. The compiler **must** apply the **most restrictive** constraint set when the concrete kind is unknown at a call site (because the argument might be unbounded).
- **`BoundedDataSet[T]`** extends `DataSet[T]`. All relational operations are allowed without streaming constraints.
- **`UnboundedDataSet[T]`** extends `DataSet[T]`. Operations requiring unbounded state **must** be rejected at compile time.
- **`DataFrame[T]`** implements `BoundedDataSet[T]`. Always bounded. Conceptually the product of collecting or executing a `LazyFrame`. Concrete runtime representation is implementation-defined but **must** preserve `T` in the type system.
- **`LazyFrame[T]`** implements `BoundedDataSet[T]`. Holds a logical plan (or equivalent) until an explicit execute, collect, or write boundary. Always bounded.
- **`DataStream[T]`** implements `UnboundedDataSet[T]`. Shares the `DataSet[T]` operation API but signals that its source is unbounded. The compiler **must** apply static streaming constraints.

The three concrete types **must not** imply three unrelated relational languages. All operations are defined on `DataSet[T]`; the bounded/unbounded distinction is a type-level property that enables or restricts specific operations statically.

### Static capability gating

| Trait bound in signature | Allowed operations                                     | Constraint level                         |
| ------------------------ | ------------------------------------------------------ | ---------------------------------------- |
| `DataSet[T]`             | Intersection of bounded + unbounded capabilities       | Most restrictive (concrete kind unknown) |
| `BoundedDataSet[T]`      | All relational operations                              | Unrestricted                             |
| `UnboundedDataSet[T]`    | Relational operations minus unbounded-state operations | Streaming constraints enforced           |

When a function accepts `DataSet[T]` (the root trait), the compiler **must** enforce streaming constraints because the input **might** be unbounded. Authors who want the full operation set **must** accept `BoundedDataSet[T]` or a concrete bounded type.

For `UnboundedDataSet[T]`, the governing rule is semantic rather than ad hoc: operations that require end-of-input semantics or unbounded retained state are not valid unless a later RFC gives them bounded-state semantics. In v0.1, the obvious disallowed examples include global `order_by`, global `limit`, unwindowed `group_by` / `agg`, eager materialization to a finite `DataFrame[T]`, and finite file writes.

### Operation API (for lowering and direct use)

The InQL library **must** expose the following instance methods on `DataSet[T]` (exact signatures may live in companion library docs; semantics **must** match this table and stay consistent with any normative lowering rules for the same logical operators elsewhere in InQL). Method names are illustrative; implementations **may** use equivalent spellings if the compiler maps them consistently.

| Method            | Role                                                                                                                                                                                    |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`filter`**      | Restrict rows by an explicit predicate builder in the current package slice (for example `eq(col("status"), str_lit("open"))`), with future sugar lowering to the same semantic target. |
| **`join`**        | Combine with another `DataSet[U]` on a join condition; named relations for `relation.column`                                                                                            |
| **`select`**      | Project columns and expressions; output row type becomes a new schema `U` the typechecker can track.                                                                                    |
| **`with_column`** | Add or replace one projected column by name using an explicit projection builder expression.                                                                                            |
| **`group_by`**    | Define grouping keys for aggregation; keys are relational expressions.                                                                                                                  |
| **`agg`**         | Apply aggregate functions over groups (often chained after `group_by`); produces grouped/aggregated schema.                                                                             |
| **`order_by`**    | Define sort keys and directions.                                                                                                                                                        |
| **`limit`**       | Cap the number of rows (after sort when both apply).                                                                                                                                    |
| **`explode`**     | Expand a nested list column into rows (or equivalent).                                                                                                                                  |

Additional requirements:

- Operations **must** preserve or update `T` (or output model `U`) in a way the typechecker can verify.
- Operations that are statically invalid on `UnboundedDataSet[T]` (e.g. unbounded-state operations) **must** produce compile-time errors, not runtime failures.
- Aggregate helpers used with `.agg(...)` are imported library symbols (from `pub::inql.functions`), not ambient builtins.
- The minimum required aggregate-helper surface for the current package slice is `col`, `sum`, and `count`.
- The current InQL-only implementation uses `col(...)` builders as the semantic target that later `.column` sugar and query-block lowering should compile to.
- The current InQL-only projection implementation uses `with_column(name, expr)` plus projection builders such as `add(...)`, `mul(...)`, and `int_expr(...)` as the semantic target that later projection sugar should compile to.
- This RFC defines the minimum required aggregate-function import model for `.agg(...)`; it is not an exhaustive catalog of all present or future InQL functions. Additional functions **may** be added later through additive library evolution or follow-up RFCs, provided they do not change the semantics of the required set defined by the InQL RFC suite.

### Execution backend boundary

- Implementations **must** separate the author-facing `DataSet` API from engine-specific code (Rust crates, Substrait consumers, etc.).
- Substrait consumption or emission at the collection/plan layer **may** be specified here as optional; the Substrait contract (InQL RFC 002) governs plan semantics. If more than one relational authoring surface emits Substrait, they **must not** produce contradictory plans for the same logical pipeline.
- The execution context owns the session, plan optimization, and concrete execution backend (DataFusion as reference implementation).
- Materialization helpers such as `collect(data)` or `display(data)` belong to the execution context and concrete implementation model, not to the `DataSet[T]` trait surface defined in this RFC.

### Interaction with Incan

- Models supply field names and types for `T`.
- Rust interop is expected for backends until stdlib covers execution.

## Design details

### Unified API model

The design draws on Spark Structured Streaming's core insight: a stream is an unbounded table. Rather than defining separate operation APIs for batch and streaming, `DataSet[T]` provides one relational operation surface. The bounded/unbounded property is expressed through the type system (`BoundedDataSet` vs `UnboundedDataSet`), allowing the compiler to enforce streaming constraints statically — an improvement over Spark's runtime `AnalysisException` approach.

### Trait naming

- **`DataSet[T]`** is InQL's root trait for any schema-parameterized relational carrier. It is intentionally aligned with the Spark notion of a typed `Dataset`, but spelled `DataSet` for Incan style.
- **`DataFrame[T]`** is a concrete eager kind, not Spark's untyped `DataFrame = Dataset[Row]` alias.
- **`BoundedDataSet[T]`** and **`UnboundedDataSet[T]`** are intermediate traits that give clean type signatures for batch-only and streaming-only consumers respectively.

### Future extensibility

`UnboundedDataSet[T]` currently has one concrete implementor (`DataStream[T]`). The intermediate trait is justified by: clean symmetry with `BoundedDataSet[T]` in type signatures, and future extensibility (e.g. a `ChangeStream[T]` for CDC, a `WindowedStream[T]`, or other streaming specializations).

Future RFCs **may** add methods on `BoundedDataSet[T]` or `UnboundedDataSet[T]`, but only where the semantics are inherently boundedness-specific and remain backend-neutral. v0.1 does not require any additional core relational methods beyond the shared `DataSet[T]` surface.

### Compatibility

- New dataset methods or kinds **should** remain backward compatible or go through a deprecation path.

## Alternatives considered

- **Only a clause-based relational surface, no programmatic API** — rejected; traits/methods give tests, lowering targets, and incremental adoption.
- **Flat hierarchy (no intermediate traits)** — rejected; without `BoundedDataSet` and `UnboundedDataSet`, authors cannot write "any batch data" in a type signature without listing concrete types. The intermediate traits make capability gating clean and type-driven.
- **Three independent kinds with separate operation APIs** — rejected; a unified API through `DataSet[T]` reduces surface area.
- **`DataStream` as the sole foundational type** (batch as bounded streams from the start) — deferred; batch-first validates the relational core with simpler semantics. The trait hierarchy ensures the model can evolve in this direction without breaking author code.

## Drawbacks

- Five types/traits in the hierarchy is more surface area than a single `DataSet[T]` plus runtime flags.
- The static capability gating rule (root trait = most restrictive) may surprise authors who expected full operations on `DataSet[T]` without thinking about boundedness.

## Layers affected

- **InQL library** (primary): types, traits, Rust companion / interop.
- **Typechecker**: generics for `DataFrame[T]` etc.; static streaming constraint checks for `UnboundedDataSet[T]`; capability gating based on trait bounds.
- **Parser**: only if dataset API introduces new surface syntax beyond ordinary calls.

## Design Decisions

### Resolved

- **`UnboundedDataSet[T]` restrictions:** Operations requiring end-of-input semantics or unbounded retained state are not valid unless a later RFC gives them bounded-state semantics. In v0.1, disallowed examples include global `order_by`, global `limit`, unwindowed `group_by` / `agg`, eager materialization to a finite `DataFrame[T]`, and finite file writes.

- **`collect` / `display`:** Not part of the `DataSet[T]` trait surface. Helpers such as `collect(data)` or `display(data)` belong to the execution context and concrete implementation model defined in InQL RFC 004, not in this RFC.

- **Intermediate traits:** `BoundedDataSet[T]` and `UnboundedDataSet[T]` do not add required core relational methods in v0.1. Future RFCs may add additional methods only where the semantics are inherently boundedness-specific and remain backend-neutral.
