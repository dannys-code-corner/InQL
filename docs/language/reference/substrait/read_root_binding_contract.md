# Substrait read-root and binding contract (Reference)

This page is the **operational reference** for InQL's normative boundary between logical read roots in Substrait plans and execution-context binding. The normative rule — that logical reads carry names and virtual values rather than secrets, and that the execution context resolves them — lives in [InQL RFC 002][rfc-002]. This page expands on the `ReadRel` variant requirements, what a read must and must not contain, the execution context's obligations, and the adapter layer boundary.

## Normative boundary

InQL relational plans **must** express all new data entering the plan as logical reads. A logical read carries a **logical identity** — a name, a virtual row set, or an opaque extension type — without normative dependence on:

- Secret material: credentials, tokens, API keys, or passwords.
- Host-specific connection strings, DSNs, or URIs that encode execution-context policy.
- Engine-specific physical scan parameters that would need to change when the plan is executed on a different conforming consumer.

The execution context **must** resolve logical reads to physical resources through its adapter and execution layer. That resolution **must not** redefine the relational semantics of the plan. The plan's meaning — which rows, which columns, which schema — is fixed at authoring time by the logical read; the execution context only supplies the physical source.

## `ReadRel` variant reference

| Variant | Substrait field | Typical InQL use | Portability |
| --- | --- | --- | --- |
| `NamedTable` | `named_table` (list of name parts) | Registered logical table name; resolved by session registry | Portable across conforming consumers that have registered the same logical name |
| `LocalFiles` | `local_files` (file list + format options) | Parquet, CSV, Arrow IPC scan from a URI | Portable if consumers can resolve the URI; URI format is not standardized by Substrait |
| `VirtualTable` | `virtual_table` (inline rows) | Literal or embedded row data; `session.from_values(...)` | Fully portable; rows are embedded directly in the plan |
| Extension leaf | `ExtensionLeafRel` | Custom source type with no applicable standard `ReadRel` variant | Extension-URI-dependent; portability requires consumers to support the registered URI |

### What a `ReadRel` must carry

- The **logical identity** of the source: name parts for named tables, format type and URI pattern for file scans, or inline data for virtual tables.
- The **base schema** (a Substrait `NamedStruct`): field names and types must match the Incan `model` `T` that parameterizes the resulting `DataSet[T]`.
- Any **filter or column hints** permitted by the pinned Substrait spec (for example, scan-level filter fields on `ReadRel`); these are optimization hints, not semantic changes, and consumers that do not support them must still produce semantically correct results.

### What a `ReadRel` must not carry (in the normative interchange)

- Raw connection strings, credentials, DSN passwords, or any secret material.
- Engine-specific scan parameters that would fail or behave differently on a different conforming consumer unless declared as a registered extension.
- Schema definitions that contradict or override the Incan `model` type checked at compile time.

## Execution context responsibilities

The execution context ([InQL RFC 004][rfc-004] `Session`) **must**:

1. Maintain a **table registry** that maps logical names to physical data source definitions (connection parameters, catalog references, or file paths).
2. **Resolve** `ReadRel` logical names through this registry at execution time — not at plan authoring time — so the serialized plan remains independent of execution-context state.
3. **Supply credentials and connection details** from its own configuration layer, never by reading them from the serialized Substrait plan.
4. Apply any **governance policy** (access control, row filtering, schema masking) that is sensitive enough to keep out of the portable plan, by injecting it at resolution time.
5. **Validate** that the resolved physical source schema is compatible with the `NamedStruct` declared in the `ReadRel` before execution begins.

## Adapter boundary

Product SDKs and higher operational layers **may** provide convenience read APIs (for example, `session.read_parquet(uri)` wrapping a `ReadRel` + `LocalFiles`). These convenience surfaces:

- **Must** produce a `ReadRel` in the normative Substrait interchange with the logical identity encoded appropriately for the variant.
- **Must not** embed execution-context state — resolved credentials, session tokens, resolved endpoint URLs — in the `ReadRel` payload of the normative plan.
- **May** pass execution-context configuration through separate, non-normative channels (for example, `AdvancedExtension` hints, out-of-band session configuration) when needed for optimization, provided the plan remains semantically valid without them.

Adapter-specific "open connection" or "bind source" APIs **should not** be specified as core InQL. They are thin wrappers at most, with the binding contract owned by the execution context per InQL RFC 004.

## Interaction with InQL RFC 001 types

The following table summarizes how each `Session` read method maps to a `ReadRel` variant and the resulting InQL carrier type.

| `Session` method | Returns | `ReadRel` variant |
| --- | --- | --- |
| `session.table(name)` | `LazyFrame[T]` | `NamedTable` |
| `session.read_parquet(uri)` | `LazyFrame[T]` | `LocalFiles` (Parquet format) |
| `session.read_csv(uri)` | `LazyFrame[T]` | `LocalFiles` (delimited text format) |
| `session.read_arrow(uri)` | `LazyFrame[T]` | `LocalFiles` (Arrow IPC format) |
| `session.from_values(rows)` | `LazyFrame[T]` | `VirtualTable` |

In all cases the `LazyFrame[T]` holds a deferred plan — no data is fetched until `session.collect(...)` is called. The `ReadRel` in the deferred plan carries only the logical identity; resolution to a physical source happens at execution time per the execution context obligations described above.

<!-- Link references (single place for targets) -->

[rfc-002]: ../../rfcs/002_apache_substrait_integration.md
[rfc-001]: ../../rfcs/001_inql_dataset.md
[rfc-004]: ../../../rfcs/004_inql_execution_context.md
