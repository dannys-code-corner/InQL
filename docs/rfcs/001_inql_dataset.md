# InQL RFC 001: Dataset types and carriers (`DataSet[T]`)

- **Status:** Implemented
- **Created:** 2026-03-22
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — naming, schema shapes, layer boundaries)
  - Incan compiler — static capability gating enforcement: [incan#187](https://github.com/dannys-code-corner/incan/issues/187)
  - InQL follow-up when enforcement lands: [InQL #10](https://github.com/dannys-code-corner/InQL/issues/10)
- **Issue:** [InQL #2](https://github.com/dannys-code-corner/InQL/issues/2)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** 0.1.0

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
- Pipe-forward (`|>`) grammar — InQL RFC 005 (deferred; outside the RFC 000–004 milestone).
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

# Deferred plan — compose before execution (signatures use Self; logical Summary row shape via RFC 003)
def build_pipeline(orders: LazyFrame[Order]) -> LazyFrame[Order]:
    ...

# Streaming specifically (signatures use Self; logical Alert row shape via RFC 003)
def process_stream(events: DataStream[Event]) -> DataStream[Event]:
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

For `UnboundedDataSet[T]`, the governing rule is semantic rather than ad hoc: operations that require end-of-input semantics or unbounded retained state are not valid unless a later RFC gives them bounded-state semantics. Typical disallowed examples include global `order_by`, global `limit`, unwindowed `group_by` / `agg`, eager materialization to a finite `DataFrame[T]`, and finite file writes.

### Operation API (for lowering and direct use)

The InQL library **must** expose the following instance methods on `DataSet[T]`. Method names are illustrative; implementations **may** use equivalent spellings if the compiler maps them consistently. Semantics **must** match this table and stay consistent with any normative lowering rules for the same logical operators elsewhere in InQL.

#### Shipped trait signatures (`Self`)

Earlier drafts of this RFC described some methods with a second type parameter `U` (for example `join` with `other: DataSet[U]`, or `select` / `agg` returning `DataSet[U]`). The **InQL library package** (see **Shipped in** in the header) instead declares these methods using Incan’s **`Self`** on the `DataSet[T]` trait: the peer carrier in `join` is `other: Self`, and `select` / `agg` return `Self`. That is the **normative contract** for the library package until a follow-up specifies richer generic method typing in Incan.

**Semantic intent is unchanged:** `join` still combines two relations (with `relation.column` naming per InQL RFC 000); `select` and `agg` still denote projection and aggregation that may change the logical row shape. Tracking output schema `U` at the **typechecker** level for those operations is expected to align with InQL RFC 003 (clause / method-chain lowering), not with extra type parameters on this **`Self`**-based trait surface. Authors should treat the table’s “Role” column as the logical behavior; the “Declared signature” column as what the Incan sources declare today.

| Method         | Declared signature (InQL library)               | Role                                                                                                               |
| -------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| **`filter`**   | `def filter(self, predicate: bool) -> Self`     | Restrict rows by a boolean relational expression (relational argument positions per InQL RFC 000).                 |
| **`join`**     | `def join(self, other: Self, on: bool) -> Self` | Combine with another relation on a join condition; named relations for `relation.column`.                          |
| **`select`**   | `def select(self) -> Self`                      | Project columns and expressions; logical output row type may differ and is tracked when lowering/typing (RFC 003). |
| **`group_by`** | `def group_by(self) -> Self`                    | Define grouping keys for aggregation; keys are relational expressions.                                             |
| **`agg`**      | `def agg(self) -> Self`                         | Apply aggregate functions over groups (often chained after `group_by`); logical schema per lowering (RFC 003).     |
| **`order_by`** | `def order_by(self) -> Self`                    | Define sort keys and directions.                                                                                   |
| **`limit`**    | `def limit(self, n: int) -> Self`               | Cap the number of rows (after sort when both apply).                                                               |
| **`explode`**  | `def explode(self) -> Self`                     | Expand a nested list column into rows (or equivalent).                                                             |

Additional requirements:

- Over time, operations **must** preserve or refine row schema information in a way the typechecker and lowering can verify; the shipped **`Self`**-based signatures intentionally do not encode every schema transition on the trait surface yet.
- Operations that are statically invalid on `UnboundedDataSet[T]` (e.g. unbounded-state operations) **must** produce compile-time errors in the **Incan** typechecker once that enforcement exists (see **Static capability gating**). The InQL library package does not implement that analysis; scheduling and tracking belong on the **Incan** compiler side, not in the **Contract-complete checklist** below.
- Aggregate helpers used with `.agg(...)` are imported library symbols from `pub::inql.functions` (for example `total` for summation; names avoid clashing with Incan/stdlib `sum` / `count`), not ambient builtins.
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

Future RFCs **may** add methods on `BoundedDataSet[T]` or `UnboundedDataSet[T]`, but only where the semantics are inherently boundedness-specific and remain backend-neutral. This RFC does not require any additional core relational methods on those intermediate traits beyond the shared `DataSet[T]` surface.

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

- **`UnboundedDataSet[T]` restrictions:** Operations requiring end-of-input semantics or unbounded retained state are not valid unless a later RFC gives them bounded-state semantics. Typical disallowed examples include global `order_by`, global `limit`, unwindowed `group_by` / `agg`, eager materialization to a finite `DataFrame[T]`, and finite file writes.

- **`collect` / `display`:** Not part of the `DataSet[T]` trait surface. Helpers such as `collect(data)` or `display(data)` belong to the execution context and concrete implementation model defined in InQL RFC 004, not in this RFC.

- **Intermediate traits:** `BoundedDataSet[T]` and `UnboundedDataSet[T]` do not add required core relational methods beyond what this RFC specifies for the shared `DataSet[T]` surface. Future RFCs may add additional methods only where the semantics are inherently boundedness-specific and remain backend-neutral.

- **Static capability gating — compiler enforcement:** The **must** in **Additional requirements** (compile-time errors for invalid `UnboundedDataSet[T]` uses) is normative language for the language; the **InQL** package issue in the header tracks the **library** contract. Typechecker implementation is tracked in **[incan#187](https://github.com/dannys-code-corner/incan/issues/187)**; **[InQL #10](https://github.com/dannys-code-corner/InQL/issues/10)** is a chore to revisit this package and docs when that work lands.

## Contract-complete checklist (library package)

RFC 001 is **contract-complete** for the InQL package when all of the following hold (**execution** / materialization: InQL RFC 004; **static streaming enforcement** in the typechecker: Incan compiler work—separate from this checklist):

- **Hierarchy:** `DataSet`, `BoundedDataSet`, `UnboundedDataSet`, `DataFrame`, `LazyFrame`, `DataStream` are public exports and match the type tree in this RFC.
- **Operation names:** The eight methods in **Operation API** exist on `DataSet[T]` with the **`Self`-based signatures** documented above and in companion library docs.
- **Aggregates:** `pub::inql.functions` exports at least the minimum aggregate helpers required for the import model (`total`, `count_rows` in the shipped package); bodies may be stubs until RFC 004.
- **Tests:** Package tests verify exports, trait assignability, and aggregate symbol importability without requiring runtime relational execution.
- **Docs:** This RFC, `docs/language/reference/dataset_types.md`, `docs/language/explanation/dataset_types.md`, and examples do not contradict the shipped `Self` signatures or stub status.
