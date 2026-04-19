# InQL RFC 011: Source discovery and parse-unit expansion

- **Status:** Draft
- **Created:** 2026-04-19
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 004 (execution context and session read boundaries)
  - InQL RFC 009 (session format handler registry)
  - InQL RFC 010 (CSV dialect and interpretation contract)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2-rc5
- **Shipped in:** —

## Summary

This RFC defines InQL's north-star source-discovery contract for file-backed reads. It standardizes how a logical input target becomes one or more parse units before any format-specific handler such as `csv`, `parquet`, or `arrow` interprets those units. The core claim is that input discovery must be its own contract layer, separate from format dialect or schema inference, so that InQL can describe single-file reads, directory or prefix expansion, and future container-aware discovery without mixing storage semantics into format semantics.

## Core model

1. Source discovery and format interpretation are separate layers.
2. Source discovery answers which parse units exist; format handlers answer how each parse unit is interpreted.
3. Source target kind determines default discovery behavior unless an explicit override is required.
4. Session-owned reads must validate discovery semantics before execution begins.
5. Discovery contracts must be explicit enough to support portability, but must not over-promise behavior that varies materially across storage systems or execution engines.

## Motivation

InQL already needs a clean distinction between "what is a CSV" and "what does this path mean." Those are not the same question. A folder, prefix, archive, or compressed object changes how input is enumerated, but it says nothing by itself about delimiter rules, headers, null tokens, or numeric inference. If those concerns are collapsed into a single format-specific options bag, the API becomes harder to reason about and portability becomes fragile.

This matters even before advanced runtime ingestion work exists. A simple read API still needs to answer whether a target is one file, many files, or some higher-level container that expands into parse units. Without a source-discovery contract, those semantics become accidental backend behavior. That is precisely the kind of ambiguity InQL should own and remove.

At the same time, this space can expand too far if we are not disciplined. Incremental discovery, checkpoints, file notifications, and streaming progress semantics belong to a different RFC. This document is about source discovery and parse-unit expansion only.

## Goals

- Define a stable InQL-level contract for source discovery ahead of format parsing.
- Let source target kind drive default discovery behavior for the common path.
- Distinguish single-target reads from multi-target expansion such as directory or prefix discovery.
- Define parse-unit expansion as a format-agnostic concept that can be reused by CSV and other file-backed formats.
- Require early rejection when a backend or handler cannot honor the requested discovery contract.
- Keep discovery semantics separate from dialect, schema inference, and malformed-record behavior.

## Non-Goals

- Defining CSV, JSON, Parquet, Arrow, or other format-specific parsing semantics.
- Defining streaming ingestion, checkpoints, file notifications, or exactly-once progress tracking.
- Standardizing every container or archive behavior across every backend in this RFC.
- Defining source write or sink discovery semantics.
- Reproducing Databricks Auto Loader or any other runtime-specific ingestion API wholesale.

## Guide-level explanation (how authors think about it)

Authors should think about source discovery as one layer below format parsing. First, InQL figures out what concrete parse units exist. Then the selected format handler parses each unit according to its own contract.

For the common path, authors should not have to restate obvious discovery behavior. A file target should imply one parse unit. A directory or prefix target should imply expansion.

For a single file, the discovery contract is trivial.

```incan
from pub::inql import Session
from pub::inql.sources import SourceDiscovery, SourceTarget
from models import Order

session = Session.default()

orders = session.read_format[Order](
    logical_name="orders",
    source=SourceTarget.file("s3://warehouse/orders.csv"),
    format="csv",
)
```

For a directory or prefix, the discovery contract becomes explicit: discover multiple files, then parse them individually under the same format contract.

```incan
from pub::inql.sources import ParseUnitExpansion, SourceDiscovery, SourceTarget

orders = session.read_format[Order](
    logical_name="orders",
    source=SourceTarget.directory("s3://warehouse/orders/"),
    format="csv",
)
```

If the default implied by target kind is not sufficient, an explicit discovery override may still exist for ambiguous or advanced cases. The important mental model is that discovery does not parse bytes. It only decides which parse units exist and in what structural category they belong.

## Reference-level explanation (precise rules)

### Public configuration model

InQL must expose a stable structured source-discovery surface for file-backed reads. The exact namespace may evolve, but the public API must include the equivalent of:

- `SourceTarget`
- `SourceDiscovery`
- `ParseUnitExpansion`

Format-dispatch reads such as `read_format` and format-specific convenience helpers such as `read_csv` or `read_parquet` may accept this configuration directly or through an equivalent builder shape. Anonymous positional flags must not be the normative API.

The common path should infer discovery behavior from source target kind. Explicit discovery configuration should exist only when the default implied by target kind is ambiguous, insufficient, or intentionally overridden.

### Source target categories

The discovery contract must distinguish at least:

- one concrete file or object
- one directory, prefix, or file-set target that may expand into multiple parse units

The contract may define additional categories later, but it must not treat fundamentally different target kinds as the same thing merely because a backend happens to accept one string path for both.

### Parse-unit expansion

Source discovery must define how a logical target expands into parse units before format interpretation begins.

When the target is one concrete file or object, the default discovery result is one parse unit.

When the target is a directory, prefix, or file set, the default discovery result must be a set of individually parsed units. InQL must not define multi-file reads as raw byte concatenation unless a future RFC explicitly standardizes such behavior.

Discovery must remain format-agnostic. It may determine parse-unit boundaries, but it must not define delimiter, quoting, schema inference, or malformed-record rules. Those belong to the selected format contract.

### Explicit overrides

InQL may expose explicit discovery overrides, but those overrides must refine or replace default target-kind behavior in a well-defined way. They must not be required in cases where the target kind already determines the obvious default.

### Capability validation

If a backend or format handler cannot honor the requested discovery contract, it must reject the read before execution begins. Silent fallback from one discovery mode to another must not be part of the portable contract.

### Ordering and determinism

If the discovery contract does not guarantee parse-unit ordering for a target class, that lack of ordering must be explicit. InQL must not leave discovered-unit ordering as an undocumented backend accident.

### Containers and archive-like targets

Container-aware discovery such as archive-member expansion may be supported in the future, but this RFC does not fully standardize that behavior. If an implementation supports container-aware discovery before a follow-on RFC exists, that support should be treated as implementation-specific rather than as portable InQL semantics.

## Design details

### Syntax

This RFC does not introduce new core language grammar. It standardizes library-surface configuration for source discovery.

### Semantics

Source discovery is the contract layer that maps a logical input target to parse units. Format handlers sit downstream from that mapping and interpret each unit according to their own contract.

That separation is not cosmetic. It is what lets InQL describe folder or prefix expansion once, instead of reinventing those semantics inside each format-specific API.

The target kind should carry as much default meaning as possible. Discovery overrides are for non-default cases, not for making authors repeat what the target already says.

### Interaction with other InQL surfaces

This RFC composes directly with InQL RFC 004 because Session-owned reads need a stable pre-execution boundary for input resolution.

This RFC composes with InQL RFC 009 because a format-handler registry should receive already-classified parse units or discovery configuration, not be forced to invent global discovery semantics independently per format.

This RFC composes with InQL RFC 010 because CSV dialect and interpretation begin only after source discovery has established the parse units to which that dialect applies. The same pattern should hold for other format contracts as well.

### Compatibility / migration

This RFC is additive in intent. Existing APIs that accept plain string paths may remain as convenience forms, but their semantics should be documented in terms of the discovery contract rather than left implicit.

Implementations may begin with a small subset of target kinds while still adopting the full contract shape. The contract should be broader than the first implementation slice, but unsupported cases must fail explicitly rather than pretending portability that does not exist.

## Alternatives considered

- Fold discovery into each format-specific RFC: rejected, because that duplicates storage semantics across formats and makes the API harder to reason about.
- Leave discovery entirely backend-defined: rejected, because InQL would lose control over path meaning and parse-unit semantics.
- Combine discovery with future streaming ingestion/runtime semantics: rejected, because discovery and runtime progress are related but distinct design layers.

## Drawbacks

- Adds another explicit configuration layer to read APIs.
- Forces InQL to decide where portability stops instead of inheriting backend behavior wholesale.
- Raises design pressure around containers and archive-like sources before backend convergence exists.

## Layers affected

- **InQL specification** must define source discovery as a contract distinct from format parsing.
- **InQL library package** must expose structured discovery configuration for file-backed reads.
- **Execution / interchange** must validate that the selected backend or handler can satisfy the requested discovery contract before execution starts.
- **Documentation** must explain the distinction between source discovery and format interpretation.

## Unresolved questions

- Should plain string-path overloads remain first-class author APIs once structured `SourceTarget` forms exist, or should they become convenience sugar only?
- Should directory and prefix expansion be one portable target class, or should they remain distinct because storage systems expose them differently?
- What should the explicit discovery-override surface look like once target-kind defaults cover the common path?
- Should parse-unit ordering for multi-file discovery be explicitly unspecified by default, or should the contract require stable ordering when the backend can provide it?
- How much of container-aware discovery should become portable InQL behavior, and how much should remain backend- or integration-specific until a dedicated follow-on RFC exists?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
