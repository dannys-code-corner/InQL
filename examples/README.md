# InQL examples

Examples demonstrating InQL dataset types and patterns.

## Current status

These examples are **compile-safe signatures** that preserve RFC 001 type contracts.

> Note: Relational expression syntax and executable backends are still evolving in sibling RFCs, so method-chain bodies are documented as comments where needed.

## Example structure

- `dataset_api.incn` — Demonstrates the DataSet[T] operation API
- `trait_hierarchy.incn` — Demonstrates trait hierarchy usage
- `bounded_vs_unbounded.incn` — Demonstrates bounded vs unbounded type signatures
- `models.incn` — Placeholder models for examples

## Running examples

```bash
incan run examples/dataset_api.incn
```

> Note: These examples primarily demonstrate type-shape contracts today. Execution semantics are defined by RFC 004.

## What these examples show

These examples document the **desired API patterns** for the initial InQL dataset surface:

1. **RFC 001** contracts are represented as compile-safe signatures and trait assignments
2. Method-chain bodies show intended relational patterns in comments
3. **RFC 004** will provide execution behavior (DataFusion integration)

Once those are in place, these examples will serve as:

- **Regression tests** — verifying the patterns still work
- **Documentation** — showing users how to use the API
- **Examples** — providing starting points for real code

## Incan status

- **RFC 041** (First-Class Rust Interop Authoring): Implemented in Incan v0.2
- **RFC 042** (Traits Are Always Abstract): Implemented in Incan v0.2

These RFCs provide the trait and interop foundation InQL builds on.

What's still needed:

- **Execution backend** — actual implementation of the operations (RFC 004)
- **Method-chain execution semantics** — examples still keep relational bodies as comments until runtime behavior lands
