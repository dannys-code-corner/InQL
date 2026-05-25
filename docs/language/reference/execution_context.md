# Execution context (Reference)

This page documents the public execution surface in the InQL package. Normative design intent lives in [RFC 004][rfc-004].

## Core types

- `Session` is the public execution context for registration, binding, execution, collection, and writes.
- `SessionBuilder` configures a `Session` before construction.
- `SessionError` is the typed error surface for registration, planning, execution, materialization, and sink failures.
- `BackendSelection` is the portable backend selection envelope stored by a session.
- `BackendOption` carries adapter-specific configuration without adding one field per backend to `Session`.
- `backends.DataFusion()` is the current reference backend configuration entry point.

## Construction

| API                                                                | Purpose                                                             |
| ------------------------------------------------------------------ | ------------------------------------------------------------------- |
| `Session.default()`                                                | Create a session with the default backend and default configuration |
| `Session.builder()`                                                | Create a builder for backend selection and configuration            |
| `Session.builder().with_backend(selection).build()`                | Build a session from a portable backend-selection envelope           |
| `Session.builder().with_datafusion(backends.DataFusion()).build()` | Build an explicit DataFusion-backed session                         |

## Read and registration surface

| API                                  | Returns                              | Notes                                                    |
| ------------------------------------ | ------------------------------------ | -------------------------------------------------------- |
| `session.register(name, source)`     | `Result[None, SessionError]`         | Bind a logical relation name to a source definition      |
| `session.table(name)`             | `Result[LazyFrame[T], SessionError]` | Resolve a registered logical relation by name            |
| `session.read_csv(name, uri)`     | `Result[LazyFrame[T], SessionError]` | Register and return a deferred CSV-backed relation       |
| `session.read_parquet(name, uri)` | `Result[LazyFrame[T], SessionError]` | Register and return a deferred Parquet-backed relation   |
| `session.read_arrow(name, uri)`   | `Result[LazyFrame[T], SessionError]` | Register and return a deferred Arrow IPC-backed relation |

All read APIs return `LazyFrame[T]`. They create deferred logical work; they do not fetch rows immediately.

## Execution and materialization surface

| API                     | Returns                              | Role                                                                                       |
| ----------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------ |
| `session.execute(data)` | `Result[LazyFrame[T], SessionError]` | Execute the backend path as a validation/checkpoint boundary without materializing locally |
| `session.collect(data)` | `Result[DataFrame[T], SessionError]` | Execute and materialize a local `DataFrame[T]`                                             |
| `lazy.collect()`        | `Result[DataFrame[T], SessionError]` | Convenience form that resolves through the active session at call time                     |

`execute(...)` and `collect(...)` are intentionally different:

- `execute(...)` proves the plan can bind, lower, and run.
- `collect(...)` performs that same work and materializes a local `DataFrame[T]`.

## Write surface

| API                                | Returns                      | Notes                                                |
| ---------------------------------- | ---------------------------- | ---------------------------------------------------- |
| `session.write_csv(data, uri)`     | `Result[None, SessionError]` | Execute deferred input if needed, then write CSV     |
| `session.write_parquet(data, uri)` | `Result[None, SessionError]` | Execute deferred input if needed, then write Parquet |

These writes are Session-owned. They do not bypass the execution context even when the input is deferred.

## Active-session convenience

| API                            | Returns                         | Purpose                                                   |
| ------------------------------ | ------------------------------- | --------------------------------------------------------- |
| `session.activate()`           | `None`                          | Make this session the active session for convenience APIs |
| `Session.get_active_session()` | `Result[Session, SessionError]` | Fetch the currently active session                        |

The active-session model exists for convenience entry points such as `lazy.collect()` and display helpers. Session-owned APIs such as `session.write_csv(...)` do not require activation because the session is already explicit at the call site.

If no active session exists when a convenience API needs one, the operation fails with a typed `SessionError`.

## Data model notes

- `LazyFrame[T]` is the deferred carrier for bounded work.
- `DataFrame[T]` is the materialized local carrier.
- `collect(...)` materialization stores structured metadata plus preview text:
  - resolved output columns
  - row count
  - preview text for display/debugging
- Preview text is for display/debugging; resolved output columns are the schema contract surfaced by collection.

## Backend note

DataFusion is the implemented execution backend. `Session` stores a backend kind plus encoded options, lowers work to Substrait, and dispatches through an internal backend adapter boundary. DataFusion is the first adapter behind that boundary; it is not the shape of the `Session` state.

## Related docs

- For the conceptual model behind this surface, see [Execution context (Explanation)](../explanation/execution_context.md)
- For carrier semantics, see [Dataset carriers (Reference)](dataset_carriers.md)

[rfc-004]: ../../rfcs/004_inql_execution_context.md
