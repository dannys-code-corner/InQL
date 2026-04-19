# Execution context (Reference)

This page documents the current public execution surface in the InQL package. It describes the API as implemented today. Normative design intent still lives in [RFC 004][rfc-004].

## Core types

- `Session` is the public execution context for registration, binding, execution, collection, and writes.
- `SessionBuilder` configures a `Session` before construction.
- `SessionError` is the typed error surface for registration, planning, execution, materialization, and sink failures.
- `backends.DataFusion()` is the current reference backend configuration entry point.

## Construction

| API                                                                | Purpose                                                             |
| ------------------------------------------------------------------ | ------------------------------------------------------------------- |
| `Session.default()`                                                | Create a session with the default backend and default configuration |
| `Session.builder()`                                                | Create a builder for backend selection and configuration            |
| `Session.builder().with_datafusion(backends.DataFusion()).build()` | Build an explicit DataFusion-backed session                         |

## Read and registration surface

| API                                  | Returns                              | Notes                                                    |
| ------------------------------------ | ------------------------------------ | -------------------------------------------------------- |
| `session.register(name, source)`     | `Result[None, SessionError]`         | Bind a logical relation name to a source definition      |
| `session.table[T](name)`             | `Result[LazyFrame[T], SessionError]` | Resolve a registered logical relation by name            |
| `session.read_csv[T](name, uri)`     | `Result[LazyFrame[T], SessionError]` | Register and return a deferred CSV-backed relation       |
| `session.read_parquet[T](name, uri)` | `Result[LazyFrame[T], SessionError]` | Register and return a deferred Parquet-backed relation   |
| `session.read_arrow[T](name, uri)`   | `Result[LazyFrame[T], SessionError]` | Register and return a deferred Arrow IPC-backed relation |

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
- Current materialization stores an opaque payload internally; row-level user APIs remain follow-on work.

## Current backend note

DataFusion is the only implemented execution backend today. The public builder/configuration surface is designed so additional backends can be added later without changing the `Session` entry point.

## Related docs

- For the conceptual model behind this surface, see [Execution context (Explanation)](../explanation/execution_context.md)
- For carrier semantics, see [Dataset types (Reference)](dataset_types.md)

[rfc-004]: ../../rfcs/004_inql_execution_context.md
