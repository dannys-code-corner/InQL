# InQL RFC 004: Execution context and DataFusion

- **Status:** Planned
- **Created:** 2026-03-24
- **Author(s):** Danny Meijer
- **Related:**
  - InQL RFC 000 (language specification — compilation model, layer boundaries)
  - InQL RFC 001 (dataset types — `DataSet[T]` carriers; `DataFrame[T]` as materialized result)
  - InQL RFC 002 (Apache Substrait — plan interchange; `ReadRel` and logical reads)
  - InQL RFC 003 (query DSL — `query {}` produces plans this RFC executes)
- **Issue:** [InQL #5](https://github.com/dannys-code-corner/InQL/issues/5)
- **RFC PR:** -
- **Written against:** Incan v0.2
- **Shipped in:** -

## Summary

This RFC specifies the **execution context**: the session object that bridges InQL's **typed logical plans** and **real execution**. It defines how authors **read data** into `DataSet[T]` values, **execute plans** (lowered to Substrait per InQL RFC 002), and **write results** back to storage. **Apache DataFusion** is the **reference (and default) execution backend** for plan optimization and execution: it consumes Substrait plans, applies query optimizations (predicate pushdown, projection pruning, join reordering, constant folding), and executes against registered data sources, returning **Apache Arrow** record batches that InQL wraps in typed `DataFrame[T]` carriers. This RFC standardizes the explicit core `Session` contract; higher operational layers may compose, scope, or inject sessions and adapter conveniences on top, but they do not redefine InQL execution semantics. With RFCs 000–004, InQL is usable for read → transform → write workflows.

## Core model

1. A **`Session`** (or **execution context**) is the entry point for InQL programs that interact with data. It holds **table registrations**, **configuration**, and a **reference to the execution backend**.
2. **Reading data** creates `LazyFrame[T]` or `DataStream[T]` values from registered sources. The session resolves logical names to physical data; the plan carries only the logical identity (InQL RFC 002).
3. **Executing plans** passes Substrait plans (or equivalent IR) through the backend's **optimizer** and **executor**, producing **Arrow record batches**.
4. **Materializing results** wraps Arrow output in typed `DataFrame[T]` carriers (InQL RFC 001).
5. **Writing results** sends `DataSet[T]` values (or materialized `DataFrame[T]`) to registered output targets.

## Motivation

InQL RFCs 000–003 define a typed query language that produces portable logical plans. Without an execution context, those plans are inert: there is no way to read data in, execute the relational work, or write results out. The execution context completes the pipeline from authored intent to running workload.

Choosing Apache DataFusion as the reference backend is a pragmatic decision: it is Rust-native, Substrait-aware, provides serious query optimization, and operates on Apache Arrow — the de facto columnar data interchange format in the modern data ecosystem. Naming it explicitly avoids the trap of an abstract "pluggable backend" with no concrete implementation.

The `Session` surface should also feel familiar to users coming from established data runtimes such as Spark: one obvious entry point for reading data, registering logical names, executing plans, and writing results. That familiarity is an ergonomic goal, but not a semantic dependency. InQL keeps its own typed `DataSet[T]` model, explicit execution boundary, and RFC-defined semantics rather than inheriting historical API baggage from other systems.

## Goals

- Define **`Session`** as the execution context: what it holds, how authors create one, and what operations it exposes.
- Specify **read operations**: how named tables, files, and virtual data become `LazyFrame[T]` or `DataStream[T]` values through the session.
- Specify **plan execution**: how Substrait plans (InQL RFC 002) flow through the backend optimizer and executor to produce results.
- Specify **materialization**: how execution output (Arrow record batches) becomes typed `DataFrame[T]` (InQL RFC 001).
- Specify **write operations**: how `DataSet[T]` values are written to registered output targets.
- Name **Apache DataFusion** as the reference and default execution backend, with **Apache Arrow** as the in-memory data representation.
- Define the **backend abstraction** so alternative backends (Polars, DuckDB, remote engines) can be substituted without changing author code, with backend-specific configuration exposed through a dedicated `backends` namespace rather than the root API.
- Clarify the boundary between the execution backend and higher operational or adapter layers that may provide source/sink integrations or scoped session conveniences.
- Shape the `Session` surface as a familiar entry point for data work, taking ergonomic inspiration from established runtimes such as Spark while preserving InQL's typed carrier model and explicit execution boundaries.

## Non-Goals

- Normative naming rules — InQL RFC 000.
- Dataset types and trait hierarchy — InQL RFC 001.
- Substrait `Rel`-level mapping and extension policy — InQL RFC 002.
- `query {}` grammar and clause inventory — InQL RFC 003.
- Orchestration, workflow scheduling, quality gates — execution and operational layers above InQL.
- Distributed execution, cluster scheduling, shuffle — out of scope for InQL; may be addressed by runners in the operational layer.
- Credential management, secret resolution, IAM — operational layer; the session receives resolved bindings, not raw secrets.
- Standardizing workflow-scoped session propagation, active-session lookup, or Reader/Writer convenience APIs — these may be provided by higher operational layers, but are not part of the core InQL contract in this RFC.

## Guide-level explanation

### Creating a session

```incan
from pub::inql import Session

session = Session.default()
```

A session holds registered data sources and configuration. `Session.default()` creates a context with the default backend (DataFusion). Authors who need custom configuration use a builder; backend-specific configuration lives under `pub::inql.backends`. Higher operational layers may wrap this construction behind step- or pipeline-level runtime setup, but the core InQL surface remains an explicit `Session`:

```incan
from pub::inql import Session, backends

session = Session.builder()
    .with_backend(backends.DataFusion())
    .build()
```

### Reading data

```incan
from pub::inql import Session, LazyFrame
from models import Order

session = Session.default()

# Register a named table (logical name → physical source)
session.register("orders", "catalog.schema.orders")

# Create a lazy plan from the registered table
orders: LazyFrame[Order] = session.table("orders")
```

`session.table("orders")` returns a `LazyFrame[Order]` — a deferred plan rooted in a `ReadRel` (InQL RFC 002) that carries the logical name `"orders"`. No data moves until the plan is executed.

For file-based sources:

```incan
from models import Event

events: LazyFrame[Event] = session.read_parquet("s3://bucket/events/*.parquet")
```

For inline data:

```incan
sample: LazyFrame[Order] = session.from_values([
    Order(order_id="1", customer_id="c1", amount=100.0),
    Order(order_id="2", customer_id="c2", amount=250.0),
])
```

### Transforming data

Once you have a `LazyFrame[T]`, use `query {}` (InQL RFC 003) or method chains (InQL RFC 001):

```incan
from pub::inql.functions import count, sum

result = query {
    FROM orders
    WHERE .amount > 100
    GROUP BY .region
    SELECT
        region,
        count() as order_count,
        sum(.amount) as total_revenue,
    ORDER BY total_revenue DESC
}
```

### Executing and collecting

```incan
from pub::inql import DataFrame
from models import OrderSummary

# Execute the plan and materialize results
materialized: DataFrame[OrderSummary] = session.collect(result)
```

`session.collect(result)` takes the `LazyFrame`, lowers to Substrait (InQL RFC 002), passes the plan through DataFusion's optimizer, executes it, and wraps the resulting Arrow record batches in a typed `DataFrame[T]`.

### Writing results

```incan
# Write to a registered output target
session.write(materialized, "catalog.schema.order_summaries")

# Or write to a file
session.write_parquet(materialized, "s3://bucket/summaries/")
```

### End-to-end example

```incan
from pub::inql import Session, LazyFrame, DataFrame
from pub::inql.functions import count, sum
from models import Order, OrderSummary

session = Session.default()
session.register("orders", "catalog.schema.orders")

orders: LazyFrame[Order] = session.table("orders")

summary: LazyFrame[OrderSummary] = query {
    FROM orders
    WHERE .status == "completed"
    GROUP BY .region
    SELECT
        region,
        count() as order_count,
        sum(.amount) as total_revenue,
}

result: DataFrame[OrderSummary] = session.collect(summary)
session.write(result, "catalog.schema.order_summaries")
```

## Reference-level explanation

### `Session` object

- **`Session`** is the primary execution context. It **must** hold:
  - A **table registry**: logical names mapped to data source definitions.
  - A **backend reference**: the execution engine (DataFusion by default).
  - **Configuration**: optimizer settings, runtime parameters, and feature flags.
- `Session.default()` **must** create a context with the DataFusion backend and default configuration.
- `Session.builder()` **must** return a builder that allows backend selection and configuration before constructing the session.
- The `Session` API **should** present a small, discoverable entry-point surface for data work, broadly analogous to familiar runtime entry points in systems such as Spark.
- That ergonomic inspiration **must not** override InQL's typed carrier model, explicit `Session.collect(...)` execution boundary, or the prohibition on raw SQL as a core execution path.

The intended core session surface for v0.1 is:

|               Method               |                                    Purpose                                    |
| ---------------------------------- | ----------------------------------------------------------------------------- |
| `Session.default()`                | Create a session with the default execution backend and default configuration |
| `Session.builder()`                | Create a builder for backend selection and configuration                      |
| `session.register(name, source)`   | Bind a logical relation name to a source definition                           |
| `session.table(name)`              | Resolve a registered logical relation as `LazyFrame[T]`                       |
| `session.read_parquet(uri)`        | Create a `LazyFrame[T]` from Parquet input                                    |
| `session.read_csv(uri)`            | Create a `LazyFrame[T]` from delimited text input                             |
| `session.read_arrow(uri)`          | Create a `LazyFrame[T]` from Arrow IPC input                                  |
| `session.from_values(rows)`        | Create a `LazyFrame[T]` from inline typed values                              |
| `session.collect(data)`            | Execute a deferred plan and materialize `DataFrame[T]`                        |
| `session.write(data, target)`      | Write a dataset to a named or registered target                               |
| `session.write_parquet(data, uri)` | Write bounded data to Parquet output                                          |
| `session.write_csv(data, uri)`     | Write bounded data to CSV output                                              |

This table defines the intended high-level API shape. The detailed normative rules for reading, registration, execution, and writing are specified in the sections below.

### Read operations

|           Method            |    Returns     |        Substrait lowering (InQL RFC 002)         |
| --------------------------- | -------------- | ------------------------------------------------ |
| `session.table(name)`       | `LazyFrame[T]` | `ReadRel` + `NamedTable`                         |
| `session.read_parquet(uri)` | `LazyFrame[T]` | `ReadRel` + `LocalFiles` (Parquet format)        |
| `session.read_csv(uri)`     | `LazyFrame[T]` | `ReadRel` + `LocalFiles` (delimited text format) |
| `session.read_arrow(uri)`   | `LazyFrame[T]` | `ReadRel` + `LocalFiles` (Arrow IPC format)      |
| `session.from_values(rows)` | `LazyFrame[T]` | `ReadRel` + `VirtualTable`                       |

- Read operations **must** return `LazyFrame[T]` — no data is fetched until the plan is executed.
- The schema parameter `T` **must** be inferred from the registered table definition, from the file schema, or from the literal values provided.
- `session.table(name)` **must** resolve `name` through the session's table registry. If the name is not registered, it **must** produce a compile-time or registration-time error.

### Table registration

- `session.register(logical_name, source_identifier)` binds a logical name to a data source definition.
- The `source_identifier` is an opaque string or structured descriptor that the session resolves through its integration and execution layers to a concrete scan. InQL does not define the format of source identifiers beyond requiring that the session can resolve them for the chosen execution backend.
- Registration **may** also accept explicit schema information (an Incan `model` type) for sources where the schema cannot be inferred.

### Plan execution

- `session.collect(lazy_frame)` **must**:
  1. Lower the `LazyFrame`'s logical plan to Substrait (conforming to InQL RFC 002).
  2. Pass the Substrait plan to the backend for optimization and execution.
  3. Wrap the resulting data in a typed `DataFrame[T]` (InQL RFC 001).
- `Session.collect(...)` is the canonical execution entry point in the normative API. Implementations **may** additionally offer a convenience form on `LazyFrame`, but it **must** delegate to session-owned execution semantics rather than bypassing the session boundary.
- The backend **should** apply query optimizations (predicate pushdown, projection pruning, join reordering, constant folding, common-subexpression elimination) before execution.
- Execution and write methods **must** report typed failures that distinguish at least registration / binding errors, lowering or planning errors, backend optimization or execution errors, and I/O or sink errors.

### Write operations

|               Method               |        Input        |              Description              |
| ---------------------------------- | ------------------- | ------------------------------------- |
| `session.write(data, target)`      | `DataSet[T]`        | Write to a registered or named target |
| `session.write_parquet(data, uri)` | `BoundedDataSet[T]` | Write to Parquet files                |
| `session.write_csv(data, uri)`     | `BoundedDataSet[T]` | Write to CSV files                    |

- Write operations **must** execute the plan if the input is a `LazyFrame[T]` (deferred), then write the materialized data.
- File-format write methods **should** accept `BoundedDataSet[T]` (InQL RFC 001) — not `UnboundedDataSet[T]`, because writing unbounded data to a finite file is not well-defined without windowing or partitioning.
- Write to streaming sinks (when supported) **may** accept `UnboundedDataSet[T]`.

### DataFusion as reference backend

Apache DataFusion is the **reference and default** execution backend for InQL v0.1:

- **Plan consumption**: DataFusion accepts Substrait plans through the `substrait` crate's consumer, converting them to DataFusion logical plans.
- **Optimization**: DataFusion's optimizer applies rule-based and cost-based optimizations to the logical plan before execution.
- **Execution**: DataFusion executes the optimized plan against registered table providers, producing Apache Arrow `RecordBatch` results.
- **Arrow as data plane**: DataFusion operates natively on Arrow columnar data. InQL's `DataFrame[T]` wraps Arrow record batches with the typed model `T` on top.

The data flow:

```text
InQL query / method chain
  → Substrait Plan (protobuf, InQL RFC 002)
  → DataFusion LogicalPlan (via substrait consumer)
  → DataFusion optimizer
  → DataFusion physical execution
  → Arrow RecordBatch[]
  → DataFrame[T] (typed InQL carrier, InQL RFC 001)
```

### Backend abstraction

- The `Session` **must** abstract over the execution backend so that alternative implementations (Polars, DuckDB, remote engines, future custom backends) can be substituted without changing author code.
- A single `Session` owns one execution backend for a given execution boundary.
- The backend interface **must** support at minimum: plan execution from Substrait, table provider registration, and result collection as Arrow record batches (or equivalent).
- DataFusion is the **default**; it is not the **only** permitted backend. Implementations **may** offer backend selection through `Session.builder()`.
- `Session.builder()` **must** expose a stable portable configuration subset (at minimum backend selection plus core execution and optimizer settings). Backend-specific tuning **may** be surfaced through backend options, but this RFC does not standardize the full DataFusion configuration surface.
- Backend-specific configuration objects **should** be exposed from `pub::inql.backends` rather than from the root `pub::inql` namespace.
- The normative user-facing session type remains `Session`; this RFC does not define backend-named session types such as `DuckDbSession` or `PolarsSession`.
- External systems such as warehouses, databases, filesystems, or object stores are not necessarily execution backends. They **may** instead appear as sources or sinks resolved through the session's integration layer while plan execution remains owned by the session's selected backend.
- Higher operational layers **may** provide scoped session propagation or convenience APIs for adapters and workflow steps, but those conveniences **must** delegate to `Session` rather than replacing the core execution model defined here.

### Interaction with InQL RFC 001 types

- `session.table(name)` returns `LazyFrame[T]` — a `BoundedDataSet[T]`.
- `session.collect(plan)` returns `DataFrame[T]` — a materialized `BoundedDataSet[T]`.
- Streaming sources (when supported) return `DataStream[T]` — an `UnboundedDataSet[T]`.
- The session **must** preserve type parameter `T` through the full read → transform → collect → write cycle.

## Design details

### Interaction with Incan

- The session is an Incan value — it can be passed, stored, and used in ordinary Incan code.
- `model` definitions supply schema for `T` as in all other InQL RFCs.

### Interaction with operational layers

- Operational layers **may** construct, scope, and inject `Session` values for steps, jobs, or pipeline runs.
- Such layers **may** offer convenience APIs on readers, writers, or workflow steps that rely on a locally scoped session, but those APIs are layered sugar over the `Session` contract rather than alternate execution semantics.
- InQL itself continues to define the session, backend selection, registration, execution, and write boundary.

### Compatibility

- New read/write formats and backend options **should** be additive.
- The `Session` API surface is expected to grow; breaking changes **must** go through a deprecation path.

## Alternatives considered

- **No session / implicit context in core InQL** (Polars-style) — rejected; an explicit session makes backend selection, table registration, and configuration visible rather than ambient. It also maps cleanly to DataFusion's `SessionContext`. Higher operational layers may still provide scoped convenience on top of that explicit core.
- **Session defined in the operational layer only** — rejected; without a session in InQL itself, there is no way to write self-contained InQL programs that read, transform, and write data. The operational layer may compose sessions with workflow and adapter concerns, but the base concept belongs in InQL.
- **Abstract backend only, no named reference** — rejected; naming DataFusion as the reference avoids an abstract interface with no concrete implementation. The abstraction exists for extensibility; DataFusion is what ships.

## Drawbacks

- Coupling to DataFusion (even as "reference") creates an implicit dependency on DataFusion's Substrait consumer maturity and feature coverage.
- The session concept adds API surface beyond pure query semantics.
- Arrow as the data plane is an implementation choice that leaks through `DataFrame[T]`'s runtime representation.

## Implementation architecture

Non-normative: the reference implementation **should** use DataFusion's `SessionContext` as the underlying engine, with InQL's `Session` wrapping it to provide typed APIs, table registration helpers, and Substrait plan submission. The Substrait-to-DataFusion path **should** use the community `substrait` crate (or equivalent).

## Layers affected

- **InQL library**: `Session` type, read/write methods, `backends` module, backend abstraction trait.
- **Rust interop / FFI**: DataFusion integration, Arrow record batch handling.
- **Typechecker**: ensuring `T` flows correctly through session methods.
- **Testing**: end-to-end tests that read, transform, and write using the session.

## Design Decisions

- **Raw SQL escape hatch:** raw SQL execution is not part of InQL and is not permitted as a `Session` escape hatch. SQL belongs in dialect-specific surfaces outside this RFC; it must not be smuggled through the execution-context API as an alternate query path.
- **External catalogs:** v0.1 standardizes logical registration and backend-resolved reads, not a portable catalog API. Integrations with systems such as Unity Catalog, Hive Metastore, or Iceberg REST **may** be supported by a backend or product layer, but their APIs and binding contracts are deferred from this RFC.
- **Collection API shape:** `Session.collect(...)` is the required canonical API. A convenience form on `LazyFrame` **may** exist, but it is secondary and must route through the session.
- **Error model:** execution-facing APIs **must** use typed errors that distinguish at least registration or binding failures, Substrait lowering or planning failures, backend optimization or runtime execution failures, and output or I/O failures. Exact type names are implementation details.
- **Builder configuration surface:** `Session.builder()` exposes a small portable configuration surface in v0.1. It **must not** promise one-to-one access to every DataFusion knob. Backend-specific configuration may be carried through backend options without becoming part of the portable InQL contract.
- **Backend namespace and session shape:** backend-specific configuration belongs under `pub::inql.backends`, while `Session` remains the portable execution entry point. This RFC does not standardize backend-specific session types.
- **Execution backend vs source/sink integrations:** the backend named in `Session` is the engine that optimizes and executes the plan. External systems used for reads or writes may be integrated through registration or adapter layers without becoming separate execution backends in the core model.
- **Scoped session conveniences:** workflow or adapter layers may offer locally scoped session access as ergonomic sugar, but the normative InQL contract remains the explicit `Session` API defined in this RFC.
- **Session API inspiration:** the `Session` surface intentionally takes ergonomic inspiration from familiar data-runtime entry points such as Spark's session object, but InQL keeps its own typed carrier semantics, backend abstraction, and explicit execution model. Familiarity is a usability goal, not a promise of Spark API or semantic compatibility.
