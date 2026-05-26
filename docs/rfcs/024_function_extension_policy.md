# InQL RFC 024: Function extension policy

- **Status:** Implemented
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Substrait lowering and extension policy)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 022 (semi-structured and format functions)
  - InQL RFC 023 (approximate and sketch functions)
- **Issue:** [InQL #41](https://github.com/dannys-code-corner/InQL/issues/41)
- **RFC PR:** [InQL #44](https://github.com/dannys-code-corner/InQL/pull/44)
- **Written against:** Incan v0.2
- **Shipped in:** v0.1

## Summary

This RFC defines how InQL handles functions that should not live in the portable core catalog: geospatial functions, cryptographic functions, engine-specific physical metadata, UDF and UDTF hooks, JVM/reflection-style escape hatches, partition transforms, and dialect-specific compatibility families. It establishes extension registration, namespacing, capability reporting, and rejection rules.

## Motivation

Spark's and Snowflake's function catalogs include useful portable functions, but they also include APIs tied to a specific runtime, physical execution model, warehouse type system, or specialist domain. dbt adds a different kind of pressure: cross-database names that need adapter-specific rendering. InQL needs a deliberate answer for those functions. Some should become registered extensions. Some should be table metadata transforms rather than scalar functions. Some should be rejected from portable InQL core.

Without an extension policy, compatibility pressure will push too much into the core catalog and weaken InQL's backend-neutral contract.

## Goals

- Define core, extension, and rejected function categories.
- Require namespacing and capability reporting for extension functions.
- Define how dialect and adapter compatibility modules relate to canonical functions.
- Define how UDF and UDTF hooks interact with the function registry.
- Keep physical execution metadata out of portable core semantics.
- Provide a home for geospatial, crypto, partition transform, and dialect compatibility families.

## Non-Goals

- Defining the full geospatial, crypto, UDF, or partition transform APIs.
- Guaranteeing Spark compatibility for runtime-specific functions.
- Defining plugin packaging mechanics beyond the function registry contract.
- Replacing Substrait extension policy.

## Guide-level explanation (how authors think about it)

Portable InQL functions should work across supported execution paths when backend support exists. Extension functions are explicit:

```incan
from pub::inql.functions import col
from pub::inql_ext.geo import st_srid

places_with_srid = places.with_column("srid", st_srid(col("geometry")))
```

If an author asks for a function that depends on physical Spark partitions, JVM reflection, or a specific engine runtime, InQL should say no in the portable core rather than pretending it is a normal data logic function.

## Reference-level explanation (precise rules)

The function registry must classify each function as core, extension, compatibility alias, engine-specific, or rejected.

Core functions must have portable InQL semantics independent of one execution engine.

Extension functions must be namespaced. They must declare ownership, function class, type rules, null behavior, determinism, backend support, and interchange behavior. Extension functions may lower through Substrait extensions when they cannot be represented in core interchange.

Engine-specific functions must not be imported into the portable core namespace. They may be exposed through explicitly named engine extension modules if their semantics are honest about runtime dependence.

Rejected functions must be documented when they are likely compatibility requests. JVM reflection functions, physical partition identifiers, input file block metadata, and backend-specific execution counters should be rejected from portable core semantics.

UDF and UDTF hooks must register function metadata before use in InQL planning. A UDF without type, null, determinism, and backend support metadata must not be treated as a fully typed portable function.

Partition transforms such as year, month, day, hour, and bucket transforms may be supported as table metadata or layout expressions, but they must not be confused with ordinary scalar date functions unless their semantics are identical.

## Design details

### Syntax

This RFC does not require new syntax. Extension functions may be imported through normal module paths or registered by package dependencies if the registry can identify their namespace and metadata.

### Semantics

Extension functions are allowed to be non-portable, but their non-portability must be visible. A plan containing extension functions must carry enough identity for downstream tooling to reject, route, or execute it intentionally.

### Interaction with other InQL surfaces

All authoring surfaces must observe the same extension policy. A function rejected from portable core must not become available simply because it is written in a query block instead of a dataframe method chain.

### Compatibility / migration

Compatibility aliases for portable functions may live in core or dialect modules. Compatibility names for non-portable functions should require explicit extension imports.

dbt-style adapter-dispatched names should be modeled as portability aliases or adapter-rendered functions only when their semantics are stable enough to be typed by InQL. They must not introduce templated SQL generation as a separate authoring model.

## Alternatives considered

- **Import every Spark or Snowflake function into core.** Rejected because runtime-specific, physical, warehouse-specific, and dynamic-type functions would undermine InQL's backend-neutral model.
- **Adopt dbt-style templated SQL as the function system.** Rejected because InQL needs typed expression semantics; dbt is a useful portability reference, not the authoring model.
- **Ban all extensions.** Rejected because geospatial, crypto, custom UDF, and backend-specific work are legitimate needs when made explicit.
- **Allow untyped UDFs anywhere.** Rejected because it breaks typed planning and diagnostics.

## Drawbacks

- Authors may need explicit imports for functions they know from another ecosystem.
- Extension capability reporting adds metadata work for extension authors.
- Rejecting familiar engine-specific functions may feel less convenient, but it keeps portable semantics honest.

## Layers affected

- **InQL specification** — function categories and rejection policy must be clear enough to prevent core catalog sprawl.
- **InQL library package** — extension modules and compatibility aliases should register metadata through the same registry model.
- **Incan compiler** — function resolution must preserve namespace and extension identity for diagnostics.
- **Execution / interchange** — Prism and Substrait lowering must carry extension identity or reject unsupported extension functions.
- **Documentation** — docs should list rejected compatibility requests and point to explicit extension alternatives where they exist.

## Implementation log

- [x] Added registry policy categories for portable core, extension-only, compatibility alias, engine-specific, and rejected requests.
- [x] Added explicit function namespaces and namespace-qualified lookup so extension names cannot shadow portable core names.
- [x] Added typed metadata helpers for extension-only, engine-specific, compatibility-alias, and rejected-policy cases.
- [x] Added tests that extension-only functions remain separate from scalar/aggregate function class, core-scoped lookup remains stable, and rejected compatibility requests remain metadata-only.
- [x] Documented the policy categories and rejection model in the function reference.

## Design Decisions

- **Namespace convention:** portable core functions use `inql.functions`. Extension packages should use explicit namespaces such as `inql_ext.<domain>` or another package-owned namespace that cannot be confused with core.
- **Dialect compatibility:** dialect compatibility should be modeled as ordinary opt-in package/module surface unless a later RFC proves a built-in dialect module is necessary. Compatibility aliases for portable functions remain metadata-visible and opt-in.
- **UDF metadata floor:** a UDF or UDTF must provide the same minimum registry facts as any extension function before it participates in typed InQL planning: namespace, function class, lifecycle, determinism, null behavior, error behavior, and interchange/backend support metadata.
- **Rejected requests:** likely compatibility requests that cannot be represented as portable data logic should be represented as rejection metadata, not fake lowerable functions.
