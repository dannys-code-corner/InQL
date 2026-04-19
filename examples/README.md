# InQL examples

Examples demonstrating InQL dataset types and Session execution patterns.

## Current status

Most examples are still focused on compile-safe RFC 001 type contracts, and the Session examples exercise the RFC 004 execution path end-to-end.

## Example structure

- `dataset_api.incn` — Demonstrates the DataSet[T] operation API
- `trait_hierarchy.incn` — Demonstrates trait hierarchy usage
- `bounded_vs_unbounded.incn` — Demonstrates bounded vs unbounded type signatures
- `session_read_transform_write_csv.incn` — Demonstrates `Session.read_csv[T](name, uri) -> LazyFrame transform -> Session.write_csv(...) -> session.activate() -> display(...)`
- `session_read_transform_write_order_lines_csv.incn` — Same flow with a realistic multi-column `OrderLine` model and fixture
- `models.incn` — Placeholder models for examples

## Running examples

```bash
incan run examples/dataset_api.incn
incan run examples/session_read_transform_write_csv.incn
incan run examples/session_read_transform_write_order_lines_csv.incn
```

> Note: Session examples expect repo fixtures in `tests/fixtures/` and write output files to `tests/target/`.

## What these examples show

These examples document the API patterns for the current InQL dataset and Session surface:

1. **RFC 001** contracts are represented as compile-safe signatures and trait assignments
2. Method-chain bodies show intended relational patterns in comments
3. **RFC 004** now provides execution behavior (`execute`, `collect`, and write sinks over DataFusion)

Once those are in place, these examples will serve as:

- **Regression tests** — verifying the patterns still work
- **Documentation** — showing users how to use the API
- **Examples** — providing starting points for real code

## Incan status

- **RFC 041** (First-Class Rust Interop Authoring): Implemented in Incan v0.2
- **RFC 042** (Traits Are Always Abstract): Implemented in Incan v0.2

These RFCs provide the trait and interop foundation InQL builds on.

What's still needed:

- **Materialized row APIs** — `DataFrame[T]` row-level accessors remain out of scope in the current slice
- **Additional convenience APIs** — broader transformation ergonomics continue in follow-on RFC slices
