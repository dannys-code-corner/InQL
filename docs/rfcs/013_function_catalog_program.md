# InQL RFC 013: Function catalog program

- **Status:** Draft
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 014 (function registry and catalog governance)
  - InQL RFC 015 (core scalar functions and operators)
  - InQL RFC 016 (core aggregate functions)
  - InQL RFC 017 (aggregate modifiers)
  - InQL RFC 018 (common scalar function catalog)
  - InQL RFC 019 (window functions)
  - InQL RFC 020 (nested data functions)
  - InQL RFC 021 (generator and table-valued functions)
  - InQL RFC 022 (semi-structured and format functions)
  - InQL RFC 023 (approximate and sketch functions)
  - InQL RFC 024 (function extension policy)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC is the umbrella tracking RFC for InQL's function catalog expansion. It defines the target shape and acceptance boundary for the related child RFCs that turn InQL from a small builder-first function surface into a typed, registry-backed dataframe and query function catalog. This RFC is complete only when the child RFCs needed for the catalog program are implemented or deliberately rejected.

## Core model

1. Function catalog work is organized by semantic boundary, not by individual function.
2. InQL must establish a registry and governance model before broad function families become stable.
3. Core scalar and aggregate functions form the first author-facing expansion.
4. Aggregate modifiers, window functions, nested data, generators, format functions, approximate/sketch functions, and extensions each require their own semantic contracts.
5. The umbrella RFC tracks program completion; child RFCs define normative behavior.

## Motivation

InQL currently exposes only a small number of functions relative to mature dataframe, warehouse, and SQL systems. Spark, DataFusion, Arrow, Beam, Snowflake, dbt, and standard SQL references show that a credible relational data surface needs many more functions, but adding them without structure would create catalog sprawl and inconsistent semantics.

The function expansion is too broad for one normative RFC if every family is specified precisely. It is also too connected to leave as unrelated RFCs, because decisions about registry metadata, aliases, null behavior, type coercion, aggregate modifiers, backend support, and extension policy affect every child document. An umbrella RFC gives the program a clear lifecycle while letting each semantic cluster remain reviewable.

## Goals

- Establish the function catalog expansion as one coordinated program.
- Define the child RFC set that must be resolved for the program to be considered complete.
- Preserve a north-star target that covers scalar, aggregate, window, nested, generator, format, approximate, sketch, and extension functions.
- Make RFC 014 (function registry and catalog governance) the prerequisite for catalog breadth.
- Allow this umbrella RFC to move to Implemented only when the child set is implemented or deliberately narrowed by explicit design decision.

## Non-Goals

- Defining individual function semantics directly in this RFC.
- Replacing the child RFCs as normative contracts.
- Requiring every Spark, DataFusion, Arrow, Beam, Snowflake, dbt, or SQL function to enter the InQL portable core.
- Defining implementation tickets, milestones, or file-level work.

## Guide-level explanation (how authors think about it)

Authors should eventually see InQL as having a coherent function catalog rather than a handful of unrelated helpers:

```incan
from pub::inql.functions import avg, col, count, date_trunc, lower, sum, trim

summary = (
    orders
        .filter(col("amount") > 0)
        .with_column("email_norm", lower(trim(col("email"))))
        .with_column("order_month", date_trunc("month", col("created_at")))
        .group_by([col("customer_id"), col("order_month")])
        .agg([count(), sum(col("amount")), avg(col("amount"))])
)
```

The child RFCs define which names exist, how they typecheck, how aliases work, and how they lower. This umbrella RFC only tracks the larger program.

## Reference-level explanation (precise rules)

The function catalog program must consist of the following child RFCs unless this RFC is amended:

- InQL RFC 014 (function registry and catalog governance)
- InQL RFC 015 (core scalar functions and operators)
- InQL RFC 016 (core aggregate functions)
- InQL RFC 017 (aggregate modifiers)
- InQL RFC 018 (common scalar function catalog)
- InQL RFC 019 (window functions)
- InQL RFC 020 (nested data functions)
- InQL RFC 021 (generator and table-valued functions)
- InQL RFC 022 (semi-structured and format functions)
- InQL RFC 023 (approximate and sketch functions)
- InQL RFC 024 (function extension policy)

This umbrella RFC must not be marked Implemented while any required child RFC remains Draft, Planned, In Progress, Blocked, or otherwise unresolved. A child RFC may be removed from the required completion set only by a design decision recorded in this RFC or by a superseding RFC.

Child RFCs must use the registry and governance model once InQL RFC 014 is Planned. If a child RFC needs behavior that contradicts InQL RFC 014, it must explicitly amend or supersede the relevant registry rule rather than creating a local exception.

Portable core functions must be distinguished from compatibility aliases and extensions. Compatibility with external systems is an input to catalog design, not an automatic inclusion rule.

## Design details

### Syntax

This RFC introduces no syntax. Syntax belongs in the relevant child RFCs or existing query/dataframe RFCs.

### Semantics

This RFC is normative for program structure and lifecycle. Function behavior is normative only in the child RFC that owns the corresponding semantic family.

### Interaction with other InQL surfaces

All function catalog work must stay consistent with InQL RFC 012. Query blocks and dataframe method chains must not acquire divergent function semantics as the catalog expands.

### Compatibility / migration

Existing helpers may remain while the child RFCs migrate them into the registry-backed catalog. Compatibility shims should resolve to canonical registry entries where semantics match.

## Alternatives considered

- **One giant function RFC.** Rejected because the surface is too broad to review or promote responsibly as one semantic unit.
- **One RFC per function.** Rejected because it fragments shared semantic decisions and creates excessive process overhead.
- **No umbrella RFC.** Rejected because the function expansion is large enough to need one lifecycle marker and a stable child-RFC map.

## Drawbacks

- The umbrella RFC adds one more document to the RFC set.
- Program completion depends on multiple child RFCs, so this RFC may remain open for a long time.
- The child-RFC map must be maintained if scope changes.

## Layers affected

- **InQL specification** — the function catalog program must remain coherent across child RFCs and existing expression/query RFCs.
- **InQL library package** — catalog implementation should follow the child RFC sequence rather than accumulating unrelated helpers.
- **Incan compiler** — compiler-facing function behavior should be driven by the registry and child RFC contracts.
- **Execution / interchange** — backend lowering and support diagnostics should follow the child RFCs and registry metadata.
- **Documentation** — function documentation should reflect the program structure and distinguish core, compatibility, and extension surfaces.

## Design Decisions

- **Child RFC scope:** the current child RFC set is the scope of the function catalog program. InQL RFC 014 through InQL RFC 024 are required children unless this umbrella RFC is later amended or superseded.
- **Implemented status:** this umbrella RFC may be marked Implemented only when all required child RFCs through InQL RFC 024 are implemented, rejected, or superseded by explicit design decision. Extension and sketch families are part of the umbrella scope, not optional follow-on scope.
