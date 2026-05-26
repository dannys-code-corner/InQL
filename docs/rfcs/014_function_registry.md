# InQL RFC 014: Function registry and catalog governance

- **Status:** Implemented
- **Created:** 2026-04-27
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (language specification and layer boundaries)
  - InQL RFC 002 (Substrait lowering and extension policy)
  - InQL RFC 003 (`query {}` blocks and relational authoring)
  - InQL RFC 007 (Prism planning and optimization)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - Incan RFC 048 (contract-backed models, emit, and interrogation tooling)
  - Incan issue #437 (top-level callable aliases)
  - Incan issue #438 (`incan docs` API documentation extraction)
  - Incan issue #636 / PR #637 (decorated function source signatures in checked API metadata)
  - Incan issue #638 / PR #641 (decorator string argument materialization)
  - Incan issue #640 / PR #643 (generic function references for decorator factories)
  - Incan issue #645 (method-call decorators for registry registration)
  - Incan issue #658 / PR #660 (`const` model constructor initializers for typed version constants)
  - Incan issue #659 / PR #660 (lowercase exported static import/codegen mismatch)
- **Issue:** [InQL #31](https://github.com/dannys-code-corner/InQL/issues/31)
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** v0.1

## Summary

This RFC defines the InQL function registry: the single source of truth for scalar, aggregate, window, generator, and extension functions across query blocks, dataframe method chains, planning, diagnostics, generated documentation, and Substrait interchange. The registry records canonical names, compatibility aliases, arity, type rules, null and error behavior, determinism, function class, boundedness restrictions, documentation metadata, lifecycle metadata, and Substrait mapping strategy so that future function expansion is coherent rather than a pile of ad hoc helpers.

## Core model

1. A function has one canonical InQL identity and zero or more compatibility aliases.
2. A function belongs to one function class: scalar, aggregate, window, generator, table-valued, partition transform, or extension-only.
3. A function signature defines accepted argument shapes, type coercion, return type rules, null behavior, error behavior, and determinism.
4. A function entry records the required Substrait interchange strategy; backend availability must be declared by adapters and must not redefine the InQL semantic contract.
5. A function entry is registered by attaching one `register_function(...)` decorator to a normal public helper; the helper declaration supplies the canonical name and signature, the decorator call supplies typed machine-readable metadata, and the helper docstring carries human-facing explanation.
6. A function entry records lifecycle metadata such as introduced, changed, deprecated, removed, and replacement versions.

## Motivation

Spark and similar systems expose a very large function surface. Copying that surface one helper at a time would make InQL harder to reason about, because related decisions such as null semantics, overflow policy, aliases, aggregate modifiers, and backend fallbacks would be scattered across individual additions. InQL needs a registry-level contract first so later RFCs can add catalog breadth without reopening the same foundational questions.

This is also necessary for diagnostics. If a function is known to InQL but cannot be represented by the current Prism/Substrait contract, authors should get a precise error or fallback explanation. If a name is a Spark-compatible, Snowflake-compatible, or dbt-style portability alias for a canonical InQL function, docs and tooling should be able to say so consistently.

## Goals

- Define the required metadata every InQL function entry must carry.
- Distinguish canonical function names from compatibility aliases.
- Define function classes and require class-specific validation.
- Require registered functions to carry Incan-standard human-facing docstrings without making docstring section policy part of this RFC.
- Define version lifecycle metadata for generated docs and compatibility planning.
- Define Substrait interchange requirements for portable core functions.
- Require explicit diagnostics for unknown, ambiguous, incorrectly used, or backend-rejected functions.
- Require mechanical validation that public helper decorators, public helper signatures, and typed registry entries do not drift.
- Provide the governance model that later catalog RFCs use when adding functions.

## Non-Goals

- Defining the full function catalog.
- Defining every scalar, aggregate, window, nested, or format function.
- Mandating a specific internal storage format for registry entries.
- Requiring every function to be available on every execution backend.
- Defining backend adapter capability registration in detail.
- Making SQL string parsing the primary function authoring surface.

## Guide-level explanation (how authors think about it)

Authors should be able to call functions through the normal InQL surfaces and rely on one semantic catalog:

```incan
from pub::inql.functions import avg, col, count, lit, lower, trim

cleaned = orders.with_column("normalized_email", lower(trim(col("email"))))
summary = cleaned.group_by([col("customer_id")]).agg([count(), avg(col("amount"))])
```

The author does not need to know whether `avg` maps to a core Substrait function, a Substrait extension URI, or a semantics-preserving Substrait rewrite. The author does need clear feedback if a function is known but used in the wrong query context, or if an execution adapter cannot consume the emitted Substrait representation.

## Reference-level explanation (precise rules)

Each registered function must have a canonical name. Canonical names must be stable once an RFC reaches Planned unless a later RFC explicitly supersedes them.

Each registered function may have aliases. An alias must resolve to exactly one canonical function in a given scope. If two imported extensions introduce the same alias for different canonical functions, InQL must report an ambiguity rather than choosing silently.

Each registered function must declare a function class. A scalar function must produce one value per input row. An aggregate function must produce one value per group or relation. A window function must require a window specification unless another RFC explicitly defines a default. A generator or table-valued function must be represented as a relation-shaping operation rather than as a scalar expression.

Each registered function must declare arity and argument constraints. The constraints may include fixed arity, variadic arity, named arguments, literal-only arguments, order-sensitive arguments, or type-family constraints.

Each registered function must declare return type rules. Return type rules may be concrete, argument-derived, aggregate-state-derived, or extension-constrained, but they must be visible to typechecking before execution.

Each registered function must declare null behavior. The registry must distinguish null-propagating functions, null-skipping aggregates, null-intolerant functions, null-safe predicates, and functions with custom null rules.

Each registered function must declare error behavior. Checked, unchecked, and `try_` forms must not be treated as interchangeable. A `try_` function must return a nullable or error-carrier result according to the relevant type contract instead of raising the same failure as the strict form.

Each registered function must declare determinism. Nondeterministic functions must not be freely constant-folded or common-subexpression-eliminated as if they were deterministic.

Portable core function entries must be backend-independent semantic contracts. Backend support, backend spelling, lowering strategy for a concrete engine, limitations, fidelity, and cost must be declared by adapters or backend capability registries, not by the function entry itself.

Each portable core function must declare a Substrait interchange strategy. The strategy must be one of:

- core Substrait expression or function
- registered Substrait extension function
- deterministic rewrite to supported Substrait expressions
- structural relation-context lowering, such as sort-field helpers consumed by `SortRel`

Prism must only accept portable core function calls that can be represented by the active InQL/Substrait contract. A function with no valid Substrait mapping must remain Draft, extension-only, or rejected for portable core until that mapping exists.

Execution backends must adapt from the Substrait representation rather than redefining InQL function semantics. A backend may declare that it supports, rewrites, emulates, approximates, or rejects a Substrait function representation, but that declaration belongs to the backend capability layer.

Each registered function must declare lifecycle metadata. The minimum lifecycle field is the InQL package version where the function was introduced. If a function's signature, semantics, alias set, Substrait mapping, or documentation contract changes in a user-visible way, the registry must record a versioned change entry. Deprecated functions must record the deprecation version, replacement guidance when a replacement exists, and removal status if removal is planned or completed.

Each registered function must have a typed registry entry for non-derivable machine metadata and an Incan-standard docstring for human-facing explanation. For ordinary public built-in functions, the canonical declaration shape is a normal public helper annotated with `register_function(...)`. The decorator call registers the helper, derives its stable function reference from the checked helper name by default, and supplies typed metadata that cannot be recovered from the public declaration. The checked helper declaration is the source for parameters, parameter types, and return type. A decorator may supply an explicit canonical-name override for host-language spelling constraints, such as `modulo(...)` registering canonical `mod`; this is not an alias mechanism. The checked InQL package source and typed registry data are the source from which compiler-facing metadata, generated docs, diagnostics metadata, and lowering tables are produced. Generated registry entries may exist for mechanically produced functions, and explicit registry objects may exist for advanced extension cases, but the registry must not depend on arbitrary body inspection, stringly alias metadata, or prose inference.

This RFC intentionally defines required metadata shapes rather than exact enum, model, class, or tagged-union implementations. The implementation may represent lifecycle, declaration-derived signatures, behavior categories, and Substrait mappings as enums, models, classes, generated records, or another typed representation, as long as the resulting normalized function catalog exposes the same fields to docs, typechecking, diagnostics, Prism, and backend capability checks.

At minimum, a registered function's machine metadata must include:

- lifecycle: introduced version, zero or more versioned changes, optional deprecation metadata, optional removal metadata, and replacement guidance when relevant
- signature: argument names, argument type expressions or type-family constraints, required/optional/variadic constraints, default values where supported, and return type rule, derived from the checked public helper declaration whenever possible
- classification: function class such as scalar, aggregate, window, generator, table-valued, partition transform, or extension-only
- behavior: normalized determinism, null behavior, and error behavior categories, including strict versus `try_` behavior where relevant
- interchange: Substrait mapping category, Substrait function or extension reference when applicable, rewrite description when applicable, and structural relation context when the helper is consumed outside scalar Rex lowering

The registry implementation must include a validation path that checks the public API surface against the typed registry metadata. The validation must fail if a registered public helper is not projected through the public facade, if a decorated ordinary built-in function has no matching public helper, if canonical names are duplicated, or if checked API metadata cannot expose the helper signature needed by the generated catalog. This validation is part of the RFC scope, not an optional future cleanup.

Generated Markdown must preserve the canonical registry facts and must use docstrings as the source for simple explanation, parameter intent, and examples. Argument names, argument types, default values, accepted argument shapes, and return types must be derived from checked public helper signatures rather than copied from prose. Hand-written reference pages may add longer conceptual explanation, additional examples, or migration notes, but they must not contradict parsed docstrings and registry metadata.

## Design details

### Syntax

This RFC does not introduce new syntax. Function calls may appear through existing and future InQL expression surfaces.

The `mean = avg` alias form used below depends on Incan support for top-level callable aliases. Until that exists, aliases may be represented by forwarding functions or omitted from the first implementation phase, but aliases must not be modeled as string fields inside function specs.

### Semantics

The registry defines meaning, not just names. Backend-specific behavior may be used only when it conforms to the registry contract for that function.

### Documentation

Function documentation is part of the registry contract, but exact required docstring sections are governed by the repository's implementation standards rather than by this RFC. Public registered functions must use Incan-standard docstrings as the canonical human-written format. Docstrings explain behavior, examples, and author intent; they must not be the source for argument shape, return type rules, null behavior, error behavior, determinism, lifecycle status, or Substrait mapping.

For ordinary built-in functions, the declaration-site `register_function(...)` decorator is the canonical registration surface. The public helper should be ordinary code that delegates to the existing expression or aggregate builder, so authors call a normal function while tooling inspects explicit typed metadata from the decorator call. Docs, LSP, typechecking, Prism, and Substrait lowering must all inspect the same resulting function catalog entry produced from the checked source and typed registry metadata.

The registration decorator must be the single declaration-side registry event for ordinary built-ins. It links exactly one helper symbol to exactly one stable function reference and receives the typed function spec that records lifecycle, determinism, null behavior, error behavior, alias policy, optional canonical-name override, and Substrait mapping. Helper signature and ordinary helper name come from the checked declaration. Backend capability declarations consume those facts; they do not redefine InQL function semantics.

Compatibility aliases must be real callable symbols rather than strings inside a function spec. For example, `mean = avg` should make `mean` an alias of the registered `avg` helper. The function catalog may record that alias after name resolution, but the aggregate spec must not contain `aliases=["mean"]`. Backend spellings and backend aliases remain backend capability concerns.

Generated reference pages must render lifecycle metadata in a consistent form. At minimum, generated pages must show when a function was introduced, when user-visible behavior changed, and whether the function is deprecated or removed. Spark's public function docs are useful prior art here: they expose "New in version", "Changed in version", and deprecation notes directly in generated API pages.

The public helper shape should stay compact enough to preserve authoring ergonomics while still making machine facts inspectable. Incan-standard docstrings are the canonical standard for explanation and examples; typed registry entries are canonical for machine facts. The following shape is illustrative only; constructor names, decorator names, enum/model/class boundaries, and helper implementation details may change:

```incan
@register_function(deterministic_spec(
    function_class=FunctionClass.Aggregate,
    lifecycle=FunctionLifecycle(
        since=v0_2,
        changed=[
            FunctionChange(version=v0_3, note="Added decimal return type rule."),
        ],
        deprecated=None,
    ),
    null_behavior=FunctionNullBehavior.NullSkippingAggregate,
    substrait=extension_mapping("avg", AVG_FUNCTION_ANCHOR),
))
pub def avg(expr: ScalarExpr[number]) -> AggregateMeasure[number]:
    """
    Return the average non-null numeric value in each group.

    Examples:
        average_order_value = avg(col("amount"))

    Parameters:
        expr: Numeric scalar expression evaluated for each input row.

    Returns:
        One aggregate measure whose value is null when no non-null input values exist.
    """
    return avg_measure(expr)
```

The exact type names and helper constructors may change during implementation. Tooling must not infer null behavior, error behavior, determinism, lifecycle status, or Substrait mapping from prose alone, and it must not infer examples or simple user-facing explanation from registry entries alone. A public helper must be explicitly linked to exactly one function reference, and generation or validation must fail if the helper signature and registry signature drift apart.

### Interaction with other InQL surfaces

`query {}` blocks and `DataSet[T]` method chains must resolve function names through the same registry semantics. InQL RFC 012 owns the scalar-expression contract; this RFC owns function identity and metadata within that expression model.

Aggregate, window, generator, nested-data, format, sketch, and extension RFCs must add functions through this registry model instead of defining incompatible local catalogs.

### Compatibility / migration

Existing helper names such as `sum`, `count`, `add`, `mul`, `eq`, and `gt` may continue as compatibility shims if they resolve to registered canonical functions. InQL should migrate documentation toward canonical names while preserving useful aliases where semantics match.

## Alternatives considered

- **Ad hoc helpers only.** Rejected because it spreads function semantics across unrelated APIs and makes backend diagnostics weaker.
- **Copy a backend catalog directly.** Rejected because InQL needs a portable author contract even when DataFusion, Spark, Snowflake, Arrow, dbt adapters, or another engine differs.
- **Expose only SQL strings.** Rejected because it loses typed Incan authoring and makes compiler diagnostics harder.
- **Let backends define function semantics.** Rejected because Prism's canonical interchange is Substrait. InQL functions must define backend-independent semantics and a Substrait representation strategy; adapters may only declare whether they can execute that representation faithfully.
- **Use only explicit registry data.** Rejected for ordinary built-ins because a hand-authored `FunctionRegistry([...])` list creates a second binding surface and makes drift likely. Explicit registry objects may still be appropriate for generated functions or extension loading.
- **Split decorator links from a central metadata list.** Rejected for ordinary built-ins because it creates two authoring events for one function: one declaration-side link and one catalog-side entry. The decorator should be the registration event and should receive the typed spec that carries the machine-readable contract.

## Drawbacks

- The registry adds upfront design work before large catalog expansion.
- Public helpers and function specs duplicate some signature information, so registration validation must catch drift.
- Compatibility aliases can become confusing if not documented clearly.
- Substrait extension mappings can grow stale unless tests and docs treat them as part of the public contract.

## Layers affected

- **InQL specification** — future function RFCs must use the registry model for canonical names, aliases, function class, type rules, null behavior, determinism, lifecycle metadata, and Substrait mapping strategy.
- **InQL library package** — public function helpers should resolve to registered functions rather than independent helper-family concepts.
- **Incan compiler** — function resolution and diagnostics should preserve registry identity when checking InQL expressions.
- **Execution / interchange** — Prism lowering must use registry Substrait mapping metadata to choose core Substrait, registered extension, semantics-preserving rewrite, or rejection. Backend capability declarations live outside the function entry.
- **Documentation** — function reference pages must be generated from, or mechanically checked against, structured registry docstrings and metadata.

## Implementation Plan

### Phase 1: Registry metadata model

- Add typed package-owned registry metadata for the current public function surface.
- Represent canonical names, stable function references, function class, lifecycle, declaration-derived signature, behavior categories, alias policy, and Substrait mapping category.
- Provide lookup helpers for registry entries by function reference and canonical name.

### Phase 2: Public helper registration

- Convert the current `functions` module surface from bare aliases to public helper functions where registration metadata needs to attach to the public call surface.
- Link each registered helper to exactly one stable function reference through the `register_function(...)` decorator.
- Preserve existing helper behavior and import names.

### Phase 3: Docs and tests

- Add focused tests for registry completeness, lookup behavior, alias policy, and Substrait mapping facts.
- Add focused validation that the decorated public helper surface and typed registry entries agree on function reference, canonical name, argument count, argument names, and return type.
- Update current function reference docs to describe the implemented registry and its relationship to generated metadata.
- Verify checked API metadata with an Incan toolchain that includes Incan issue #636 / PR #637, Incan issue #638 / PR #641, Incan issue #640 / PR #643, and Incan issue #645 support for registry decorators.

## Progress Checklist

### Spec / design

- [x] Resolve package-versus-compiler registry ownership.
- [x] Resolve compatibility alias import policy.
- [x] Remove exact docstring section policy from this RFC.
- [x] Resolve logical decorator and typed registry direction.
- [x] Keep RFC 014 status and checklist aligned with implementation progress.

### Package registry

- [x] Add typed function registry models and enums.
- [x] Add stable function references for the current public helpers.
- [x] Add registry entries for current scalar, aggregate, predicate, and literal helpers.
- [x] Add lookup and catalog helper functions.

### Public helper registration

- [x] Link registered public helpers to stable function refs with logical decorators.
- [x] Preserve current `functions` module imports and runtime behavior.
- [x] Confirm checked API metadata preserves decorated function signatures with the Incan #636 fix.

### Tests

- [x] Test registry completeness for current public helpers.
- [x] Test lookup by function reference and canonical name.
- [x] Test Substrait mapping and alias policy facts.
- [x] Test registry/API drift validation for decorated public helpers and typed registry entries.
- [x] Test decorated helper metadata extraction with the fixed Incan toolchain.

### Docs

- [x] Update current function reference docs.
- [x] Update release notes for the registry-backed function catalog foundation.

## Design Decisions

- **Registry ownership:** the checked InQL package source is the source of truth. Compiler-facing metadata, generated docs metadata, diagnostics metadata, and lowering tables are derived from checked package source and typed registry data. The compiler must not maintain an independent InQL function registry.
- **Authoring DX:** ordinary function authors should write normal public helpers and attach one registry decorator whose arguments contain the typed function spec. The registry derives the stable function reference from the checked public helper name, avoiding a separate authored constant or central list.
- **Decorator capability:** Incan issue #636 / PR #637 is required for decorator-authored helpers because checked API metadata must preserve source signatures for decorated functions. Incan issue #638 / PR #641 is required for decorator string argument materialization. Incan issue #640 / PR #643 provides generic signature-preserving decorator factories. Incan issue #645 is required for method-call decorators such as `register_function(...)`. The RFC design is one registry decorator attached to the public helper.
- **Lifecycle constants:** typed lifecycle metadata uses immutable version constants such as `v0_1`. These are `const` model values, not mutable registry state or generated strings.
- **Alias policy:** core semantic aliases may be available through normal public imports when they are real callable aliases of the canonical function. Dialect, warehouse, Spark, Snowflake, dbt, and backend compatibility aliases require explicit opt-in modules.
- **Docstrings:** exact docstring section requirements are not an RFC 014 concern. Public registered functions must follow the repository's Incan-standard docstring policy, but registry metadata and public helper signatures own machine facts.
- **Substrait mapping:** typed registry entries must represent whether a function maps to a core Substrait function, a registered extension function, a deterministic rewrite, or structural relation-context lowering. Backend capability declarations consume the emitted Substrait representation; they do not redefine InQL semantics.
