# InQL

**InQL** is the **typed data logic plane** for [Incan](https://github.com/dannys-code-corner/incan): the place where you express **relational queries**, **schema-aware table transformations**, and **streaming-shaped relational work** with compile-time checks, without folding orchestration, catalogs, or engine-specific runtime into the authoring model. Row shapes come from Incan `model` types; column, join, and alias rules are part of one semantic core whether you use `query { }` blocks, method chains on `DataSet[T]` carriers, or (later) optional pipe-forward (`|>`).

**What InQL is not:** It is not a pipeline or workflow framework, not a semantic catalog, and not a catch-all that swallows execution concerns. It owns **data logic**: query authoring, relational plan shape, resolution and schema flow, typed carrier semantics, and **backend-neutral logical intent**. Execution, binding, and operational semantics live in the layer below (session, adapters, runners).

**Why it matters:** Raw SQL strings and untyped rows defer mistakes to runtime. InQL keeps relational work **in the language**: schemas are ordinary models, invalid references and many aggregation mistakes are caught where Incan promises checking, and plans are intended to lower to **Apache Substrait** so logical intent stays portable while credentials and physical reads are resolved outside the normative plan story.

**Bottom line:** InQL is where you write **checked relational logic** in Incan: the same naming and schema rules apply to `query { }` blocks and `DataSet[T]` APIs, and **execution stays downstream** (sessions, adapters, runners) and consistent.

## What you get

- **Carriers that know their row type** — `DataFrame[T]`, `LazyFrame[T]`, and `DataStream[T]` share a `DataSet[T]` surface; bounded vs unbounded is reflected in the type hierarchy so unsafe streaming operations can be rejected at compile time.
- **SQL-familiar `query { }` blocks** — Clause-oriented relational syntax, typed against the current query schema, aligned with the same resolution rules as method chains.
- **One naming model** — `.column`, `alias.column`, bare names in the query schema, and ordinary Incan bindings are specified so blocks, chains, and future surfaces stay equivalent where it counts.
- **A registry-backed function catalog** — Core operators, aggregates, common scalar functions, window helpers, generators, nested-data helpers, and compatibility aliases share one checked helper model and carry portable metadata for Substrait and backend adapters. Typed helpers accept primitives where that is the natural authoring shape, such as `add(col("amount"), 1)`, `substring(col("sku"), 1, 3)`, or `cast(col("amount_text"), float)`, while query-schema validation checks referenced column types during planning and lowering.
- **Portable logical plans** — Substrait is the normative interchange; read roots stay logical while binding and execution stay in the session layer (see RFCs 002 and 004).

Design is **RFC-driven**; **[docs/rfcs/](docs/rfcs/README.md)** is the source of truth.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow and [architecture.md](docs/architecture.md) for how this repo relates to the Incan compiler.

## Design (RFCs)

Normative proposals live under **[docs/rfcs/](docs/rfcs/README.md)**. InQL’s RFC series is separate from [Incan’s RFC index](https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs).

| RFC     | Topic                                                                                            |
| ------- | ------------------------------------------------------------------------------------------------ |
| **000** | Language specification — naming, query schema, schema shapes, layer boundaries                   |
| **001** | Dataset types — `DataSet[T]`, bounded/unbounded traits, `DataFrame` / `LazyFrame` / `DataStream` |
| **002** | Apache Substrait — `Rel`-level contract, mapping catalog, logical reads vs binding               |
| **003** | `query { }` blocks — grammar, typing, lowering to Substrait                                      |
| **004** | Execution context — session, read / execute / write, DataFusion as reference backend             |
| **005** | Pipe-forward (`\|>`)                                                                             |
| **013** | Function catalog program — shared registry, metadata, policy, and implementation plan            |
| **018** | Common scalar function catalog — math, string, regex, date/time, encoding, and aliases           |

## Project layout

- `Makefile` — build, test, and format targets (`make help`)
- `incan.toml` — package manifest
- `src/lib.incn` — public exports
- `src/` — library modules
- `tests/` — tests
- `.github/workflows/` — CI (builds the pinned Incan release branch, then runs InQL checks)

Build and test from this repo root (with `incan` on your `PATH`):

```bash
make ci
```

Or invoke the toolchain directly:

```bash
incan build --lib
incan test tests
```

See `make help` for other targets (`fmt`, `fmt-check`, `registry-metadata`, `build-locked`, …). Continuous integration builds **Incan from source** from the workflow's pinned `INCAN_REF` release branch or tag, then runs `fmt-check`, `test-style`, function registry metadata validation, `build`, `test`, and the pub-consumer smoke check (see [.github/workflows/ci.yml](.github/workflows/ci.yml)).
