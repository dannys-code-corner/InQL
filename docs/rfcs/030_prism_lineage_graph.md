# InQL RFC 030: Prism lineage graph

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 002 (Apache Substrait integration)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 019 (window functions)
  - InQL RFC 020 (nested data functions)
  - InQL RFC 021 (generator and table-valued functions)
  - InQL RFC 022 (semi-structured and format functions)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 041 (Prism plan ingress and external client frontends)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines the Prism lineage graph for InQL. The graph records relation-level, field-level, and expression-level dependencies over authored and rewritten Prism plans, including reads, projections, filters, joins, aggregates, windows, generators, ordering, limits, nested data access, and semi-structured or format parsing.

## Motivation

Lineage reconstructed from backend SQL, backend plans, or Substrait alone is too late and too lossy. Prism sees typed relational intent before backend lowering and before execution. That makes Prism the right source for local lineage evidence, provided lineage is modeled explicitly instead of inferred later from display names or emitted plans.

## Goals

- Define native InQL lineage edges over semantic targets.
- Distinguish value, control, grouping, join, sort, policy, and quality dependencies.
- Preserve authored-origin and ingress-origin relationships through rewrites.
- Represent exact, conservative, and unknown lineage confidence.
- Cover current relational operations, including functions added by the function catalog program.

## Non-Goals

- Defining global cross-workspace lineage storage.
- Defining business meaning, certification, or ownership.
- Replacing Substrait relation lowering.
- Guaranteeing that every external lineage tool can represent every native InQL edge kind.

## Guide-level explanation (how authors think about it)

Given a grouped relation:

```incan
summary = (
    orders
        .filter(col("status") == "paid")
        .group_by([col("customer_id")])
        .agg([sum(col("amount")).alias("total_amount")])
)
```

InQL should be able to explain that `total_amount` has value lineage from `orders.amount`, grouping lineage from `orders.customer_id`, and control lineage from `orders.status`.

## Reference-level explanation (precise rules)

The Prism lineage graph must contain lineage edges between semantic targets. Each edge must include:

- source target
- destination target
- relationship kind
- transformation kind
- optional expression reference
- confidence
- evidence references

Relationship kind must distinguish at least value, control, grouping, join, sort, window, policy, and quality relationships.

Transformation kind must distinguish at least identity, expression, aggregate, window, generator, filter, join, mask, format_parse, nested_access, semi_structured_access, opaque, and unknown transformations.

Confidence must distinguish exact, conservative, and unknown lineage. Exact lineage means InQL can identify the relevant source target set. Conservative lineage means InQL can identify a safe over-approximation. Unknown lineage means InQL cannot determine the dependency and must report that uncertainty explicitly.

Projection of an input field without transformation must produce value lineage from the input field to the output field. Projection of an expression must produce value lineage from every scalar input expression dependency to the output field.

Filtering must produce control lineage from filter input fields to the filtered relation output. A filter must not be represented as value lineage to every output field unless a child RFC explicitly defines that projection.

Joins must produce join lineage from join key and predicate fields to the joined relation output. Output fields must preserve value lineage from the side that produced the field.

Aggregates must produce grouping lineage from group keys to aggregate result rows and value lineage from aggregate measure inputs to aggregate output fields.

Window functions must produce window lineage from partition, order, frame, function input, and default expressions to the window output field. A window output remains row-level in the surrounding projection.

Generator outputs must have generator output targets. Generated fields must preserve lineage from generator input expressions and any declared schema information.

Nested and semi-structured access must be exact when the accessed schema is typed and known. It must be conservative or unknown when dynamic payload shape prevents exact field lineage.

Rewrites must preserve authored origins. A rewritten edge may point to rewritten targets, but tools must be able to recover the authored targets that explain the lineage.

Plans produced through external client frontends must preserve ingress-origin relationships from client request nodes to Prism targets where available. Ingress-origin relationships explain protocol provenance, but they must not be treated as value, control, grouping, join, sort, policy, or quality lineage by themselves.

## Design details

### Syntax

This RFC introduces no syntax.

### Semantics

Lineage is a semantic graph over Prism targets. It is not an execution log and not a formatted explanation string.

### Interaction with other InQL surfaces

Function registry metadata must provide lineage-relevant facts for functions whose behavior is not derivable from argument structure alone. Opaque user-defined or extension functions must produce conservative or unknown lineage unless they provide explicit metadata.

### Compatibility / migration

Plans without lineage remain executable. Inspection APIs must distinguish unsupported lineage from empty lineage.

## Alternatives considered

- **Use OpenLineage column lineage as the internal model.** Rejected because InQL needs richer relationship kinds than common post-hoc run events.
- **Infer lineage from Substrait only.** Rejected because Prism has authored-origin and type information that may be lost during interchange lowering.
- **Only expose field-level direct lineage.** Rejected because governance, quality, and blast-radius analysis need control, grouping, join, and sort relationships.

## Drawbacks

- Rich lineage requires careful tests for every relation and expression family.
- Conservative lineage may produce noisy downstream impact results.
- Rewrites become more constrained because they must preserve origin evidence.

## Layers affected

- **InQL specification** — lineage relationship and transformation kinds become normative vocabulary.
- **InQL library package** — inspection APIs must expose typed lineage edges.
- **Execution / interchange** — Substrait and adapters may carry references to lineage targets but must not redefine them.
- **Documentation** — docs must explain the difference between value, control, grouping, join, and sort lineage.

## Unresolved questions

- Should sort lineage attach to relation outputs, fields, or a separate ordering target?
- Which nested and semi-structured operations can claim exact lineage in the first release?
- How should lineage represent set operations once InQL adds them?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
