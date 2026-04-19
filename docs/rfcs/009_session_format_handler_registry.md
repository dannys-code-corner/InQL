# InQL RFC 009: Session Format Handler Registry

- **Status:** Draft
- **Created:** 2026-04-18
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:** RFC 004, RFC 007
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2-rc1
- **Shipped in:** —

## Summary

This RFC introduces a Session-owned format handler registry so InQL can support built-in and third-party source formats through one stable contract, instead of hardcoding format-specific branches in Session and backend integration code.

## Motivation

Today, Session format support is effectively hardcoded (`csv`, `parquet`, `arrow`). That works for initial delivery but scales poorly: each new format requires internal edits across Session and backend code paths.

This is painful in two ways:

- Product velocity: adding a new format means patching internals rather than registering behavior.
- Ecosystem fit: external teams cannot define and reuse their own source formats cleanly.

A handler registry makes format support extensible without destabilizing Session core APIs.

## Goals

- Define a stable InQL-level contract for source format handlers.
- Route built-in formats through the same handler contract as custom formats.
- Let Session resolve format behavior by format key, not hardcoded branching.
- Preserve typed Session errors and stable diagnostics when handlers fail.

## Non-Goals

- This RFC does not define full backend plugin architecture.
- This RFC does not change Prism planning semantics.
- This RFC does not change carrier semantics (`LazyFrame` / `DataFrame`).
- This RFC does not require dynamic module loading in the first slice.
- This RFC does not define package-distribution policy for handlers.

## Guide-level explanation (how authors think about it)

Authors should be able to register a handler and then read data through that format key.

```incan
from session import Session
from session.formats import SessionFormatHandler

class FooFormatHandler with SessionFormatHandler:
    def format_name(self) -> str:
        return "foo"

    def infer_schema(self, uri: str) -> Result[list[RowColumnSpec], SessionError]:
        ...

    def register_source(self, ctx: BackendContext, logical_name: str, uri: str) -> Result[None, SessionError]:
        ...

def main() -> None:
    mut session = Session.default()
    session.register_format_handler(FooFormatHandler())?
    rows = session.read_format[OrderLine]("orders", "foo://bucket/orders.foo", "foo")?
```

Built-ins should follow the same path:

```incan
session = Session.default()
rows = session.read_format[OrderLine]("orders", "tests/fixtures/orders.csv", "csv")?
```

Convenience APIs like `read_csv` remain available and delegate to `read_format(..., "csv")`.

## Reference-level explanation (precise rules)

- Session must expose format-handler registration and lookup by format key.
- Handler format keys must be unique per Session instance; duplicate registration must fail with typed error.
- Session must fail unknown format keys with typed error (`unsupported_source` or equivalent documented kind).
- Session must route source validation, schema inference, and backend source registration through the resolved handler.
- Built-in formats must be represented as handlers under the same contract path.
- `read_csv`, `read_parquet`, and `read_arrow` should remain API-compatible and must delegate through registered built-in handlers.
- Handler failure messages must be surfaced as Session errors without dropping backend context text.

Errors and diagnostics:

- Invalid handler registration must produce deterministic error kinds and messages.
- Unknown format key reads must include the requested key in the message.
- Handler registration/runtime failures must preserve stage (`registration`, `schema`, or `backend`) in error kind or message.

## Design details

### Syntax

No new core language syntax is required. This RFC adds InQL library API surface.

### Semantics

A format handler owns format-specific adaptation only:

- source URI validation
- schema inference hints
- backend registration behavior for supported backends

A format handler does not own logical planning or optimization semantics.

### Interaction with other InQL surfaces

- RFC 004: Session execution boundaries remain unchanged; format handlers only provide source adaptation.
- RFC 007: Prism remains backend-agnostic at logical planning level; handlers influence source setup before execution, not logical rewrite strategy.
- Existing `read_*` convenience methods remain as thin aliases over the registry path.

### Compatibility / migration

- Existing author code using `read_csv` / `read_parquet` / `read_arrow` remains source-compatible.
- New `read_format` is additive.
- Internal migration is expected from hardcoded format branches to handler dispatch.

## Alternatives considered

- Keep hardcoded format branches in Session: rejected; poor extensibility and increasing branch complexity.
- Add single callback hook without formal contract: rejected; weak typing and inconsistent behavior between formats.
- Push all format logic into backend modules only: rejected; Session still needs a stable format-level API for authors.

## Drawbacks

- Adds new abstraction surface that must be versioned and documented.
- Increases test matrix across built-in and custom handlers.
- Requires careful error taxonomy to keep debugging quality high.

## Layers affected

- **InQL specification**: RFC 004 alignment must remain explicit; Session format behavior becomes contract-based.
- **InQL library package**: Session API must add handler registration and format-key read path; built-ins must be re-expressed as handlers.
- **Incan compiler**: no mandatory syntax/parser work in first slice; typechecking/emission must continue to support method calls and generic read APIs used by handler-based Session flows.
- **Execution / interchange**: source registration path must be handler-driven before backend planning/execution.
- **Documentation**: Session and execution-context docs must describe handler registration and format-key behavior.

## Unresolved questions

- Should `read_format` require explicit format key always, or allow URI-based inference as optional sugar?
- Should handler registration be mutable Session-only, or also supported at `SessionBuilder` level?
- Should schema inference be required in the contract, or optional with a documented fallback policy?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
