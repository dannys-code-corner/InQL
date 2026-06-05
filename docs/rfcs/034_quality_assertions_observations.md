# InQL RFC 034: Quality assertions and observations

- **Status:** Draft
- **Created:** 2026-05-29
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 004 (execution context)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 016 (core aggregate functions)
  - InQL RFC 017 (aggregate modifiers)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 032 (execution observations)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines InQL quality assertions and quality observations. Quality assertions are typed relational checks over datasets, fields, groups, or explicit multi-relation inputs. Quality observations are runtime results produced by executing those assertions. A quality assertion is not an ordinary filter unless the author explicitly asks to filter rows.

## Motivation

Data quality needs to participate in typed relational planning without collapsing into ad hoc post-run tests or silent filters. InQL can express many quality checks as relational work: row counts, null rates, accepted values, uniqueness, ranges, group thresholds, and aggregate conditions. Those checks should produce observations that sessions, CI, and pipeline layers can consume.

## Goals

- Define quality assertions as semantic targets.
- Define quality observations as execution evidence.
- Distinguish assertion declaration from runtime result.
- Support relation, field, group, and explicit cross-relation scopes.
- Preserve the rule that quality does not silently change relation cardinality.

## Non-Goals

- Defining pipeline orchestration, quarantine storage, retry behavior, or promotion gates.
- Defining a full Great Expectations-compatible suite model.
- Defining row-level model validation owned by the base language.
- Treating quality checks as filters by default.

## Guide-level explanation (how authors think about it)

Authors can declare checks and execute them through a session:

```incan
checks = [
    row_count(min=1),
    null_rate(col("customer_id"), max=0.0),
    unique(col("order_id")),
]

observations = session.observe_quality(orders, checks)
```

The observations report pass, fail, error, or skipped status. The `orders` relation is not filtered by these checks unless the author separately applies a filter.

## Reference-level explanation (precise rules)

A quality assertion must include assertion identity, target, predicate or metric expression, severity, mode, scope, and evidence references.

Assertion scope must distinguish relation, field, group, and explicit cross_relation scopes. Cross-relation assertions are valid only when every relation is an explicit input to the assertion plan.

Assertion mode must distinguish observe, require, and quarantine or equivalent policy-neutral states. The mode describes intended handling, but pipeline behavior belongs outside this RFC.

A quality observation must include observation identity, assertion identity, execution attempt identity when applicable, status, metrics, diagnostics, and optional redacted sample references.

Observation status must distinguish passed, failed, errored, skipped, and unsupported.

Quality assertions may be planned as relational work. They must not change the cardinality or contents of the checked relation unless represented as an explicit transformation requested by the author.

Quality expressions must use ordinary InQL scalar, aggregate, and grouping semantics. Invalid expression context must be diagnosed before execution where possible.

## Design details

### Syntax

This RFC introduces no new block syntax. Helper APIs are sufficient for the first version.

### Semantics

Quality assertions produce observations. They may inform session failure, warnings, CI status, or pipeline gates, but those policies are outside the assertion semantics.

### Interaction with other InQL surfaces

Future `query {}` or pipeline syntax may lower to the same assertion model. The assertion model must not become method-chain specific.

### Compatibility / migration

Existing filters and projections are unaffected. Users must opt into quality assertions separately.

## Alternatives considered

- **Treat every quality check as a filter.** Rejected because observations and transformations are different semantics.
- **Leave quality entirely to external tools.** Rejected because typed relational checks need InQL schema and expression semantics.
- **Introduce syntax first.** Rejected because APIs and evidence contracts should settle before syntax.

## Drawbacks

- Quality APIs add another author-facing surface.
- Some checks may require backend support or additional execution cost.
- Separating observe/require/quarantine from pipeline behavior requires clear docs.

## Layers affected

- **InQL specification** — quality assertion and observation semantics become normative.
- **InQL library package** — public helper APIs and session observation APIs are affected.
- **Execution / interchange** — quality plans may lower to backend-executable relational work.
- **Documentation** — docs must distinguish checks, filters, and pipeline gates.

## Unresolved questions

- Which quality helpers belong in the first release?
- Should quality observations always require a Session, or can some be evaluated against in-memory data carriers directly?
- How should redacted sample references be represented?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
