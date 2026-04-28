# InQL RFC 014: Function registry and catalog governance

- **Status:** Draft
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
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.2
- **Shipped in:** —

## Summary

This RFC defines the InQL function registry: the single source of truth for scalar, aggregate, window, generator, and extension functions across query blocks, dataframe method chains, planning, diagnostics, generated documentation, and Substrait interchange. The registry records canonical names, compatibility aliases, arity, type rules, null and error behavior, determinism, function class, boundedness restrictions, documentation metadata, lifecycle metadata, and Substrait mapping strategy so that future function expansion is coherent rather than a pile of ad hoc helpers.

## Core model

1. A function has one canonical InQL identity and zero or more compatibility aliases.
2. A function belongs to one function class: scalar, aggregate, window, generator, table-valued, partition transform, or extension-only.
3. A function signature defines accepted argument shapes, type coercion, return type rules, null behavior, error behavior, and determinism.
4. A function entry records the required Substrait interchange strategy; backend availability must be declared by adapters and must not redefine the InQL semantic contract.
5. A function entry is registered by attaching one registry decorator to a normal public helper; the decorator points at a typed model/spec for machine-readable metadata, and the helper carries an Incan-standard docstring that can be parsed into Markdown reference documentation.
6. A function entry records lifecycle metadata such as introduced, changed, deprecated, removed, and replacement versions.

## Motivation

Spark and similar systems expose a very large function surface. Copying that surface one helper at a time would make InQL harder to reason about, because related decisions such as null semantics, overflow policy, aliases, aggregate modifiers, and backend fallbacks would be scattered across individual additions. InQL needs a registry-level contract first so later RFCs can add catalog breadth without reopening the same foundational questions.

This is also necessary for diagnostics. If a function is known to InQL but cannot be represented by the current Prism/Substrait contract, authors should get a precise error or fallback explanation. If a name is a Spark-compatible, Snowflake-compatible, or dbt-style portability alias for a canonical InQL function, docs and tooling should be able to say so consistently.

## Goals

- Define the required metadata every InQL function entry must carry.
- Distinguish canonical function names from compatibility aliases.
- Define function classes and require class-specific validation.
- Define the required documentation block every registered function must carry.
- Define version lifecycle metadata for generated docs and compatibility planning.
- Define Substrait interchange requirements for portable core functions.
- Require explicit diagnostics for unknown, ambiguous, unsupported, or incorrectly used functions.
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

The author does not need to know whether `avg` maps to a core Substrait function, a Substrait extension URI, or a semantics-preserving Substrait rewrite. The author does need clear feedback if a function is known but cannot be represented by the current portable interchange contract.

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
- explicitly unsupported until a Substrait mapping exists

Prism must only accept portable core function calls that can be represented by the active InQL/Substrait contract. A function with no valid Substrait mapping must remain Draft, extension-only, or rejected for portable core until that mapping exists.

Execution backends must adapt from the Substrait representation rather than redefining InQL function semantics. A backend may declare that it supports, rewrites, emulates, approximates, or rejects a Substrait function representation, but that declaration belongs to the backend capability layer.

Each registered function must declare lifecycle metadata. The minimum lifecycle field is the InQL package version where the function was introduced. If a function's signature, semantics, alias set, Substrait mapping, or documentation contract changes in a user-visible way, the registry must record a versioned change entry. Deprecated functions must record the deprecation version, replacement guidance when a replacement exists, and removal status if removal is planned or completed.

Each registered function must have a typed registry spec for machine-readable metadata and a structured Incan-standard docstring for human-facing explanation and examples. For ordinary public built-in functions, the canonical declaration shape is an explicit constant spec adjacent to a normal helper function registered with `@function(SPEC)`. The spec carries registry facts; the helper carries the user-facing call surface, body, and docstring. Generated registry entries may exist for mechanically produced functions, and explicit registry objects may exist for advanced extension cases, but the registry must not depend on arbitrary body inspection, stringly alias metadata, or prose inference.

This RFC intentionally defines required metadata shapes rather than exact enum, model, class, or tagged-union implementations. The implementation may represent lifecycle, signatures, behavior categories, and Substrait mappings as enums, models, classes, generated records, or another typed representation, as long as the resulting normalized function catalog exposes the same fields to docs, typechecking, diagnostics, Prism, and backend capability checks.

At minimum, a registered function's machine metadata must include:

- lifecycle: introduced version, zero or more versioned changes, optional deprecation metadata, optional removal metadata, and replacement guidance when relevant
- signature: argument names, argument type expressions or type-family constraints, required/optional/variadic/literal-only constraints, default values where supported, and return type rule
- classification: function class such as scalar, aggregate, window, generator, table-valued, partition transform, or extension-only
- behavior: normalized determinism, null behavior, and error behavior categories, including strict versus `try_` behavior where relevant
- interchange: Substrait mapping category, Substrait function or extension reference when applicable, rewrite description when applicable, and unsupported reason when no mapping exists

At minimum, the docstring itself must include:

- one short summary sentence
- plain-language parameter descriptions when the signature alone is not self-explanatory
- plain-language return meaning when the signature alone is not self-explanatory
- at least one example for public functions
- related functions where authors are likely to confuse names or behavior

The generated Markdown must preserve the canonical registry facts and must use the docstring as the source for simple explanation and examples. Argument names, argument types, default values, accepted argument shapes, and return types must be derived from the function spec and public helper signature rather than copied from prose. Docstring `Args:` and `Returns:` sections may add human descriptions, but generation must fail if those sections mention parameters or return facts that do not correspond to the derived signature. Hand-written reference pages may add longer conceptual explanation, additional examples, or migration notes, but they must not contradict the parsed docstring and registry metadata.

## Design details

### Syntax

This RFC does not introduce new syntax. Function calls may appear through existing and future InQL expression surfaces.

The `mean = avg` alias form used below depends on Incan support for top-level callable aliases. Until that exists, aliases may be represented by forwarding functions or omitted from the first implementation phase, but aliases must not be modeled as string fields inside function specs.

### Semantics

The registry defines meaning, not just names. Backend-specific behavior may be used only when it conforms to the registry contract for that function.

### Documentation

Function documentation is part of the registry contract. Public registered functions must use Incan-standard docstrings as the canonical human-written format. The function-doc standard should remain structured and parser-friendly: short summary first, optional longer description, then section headers such as `Args:`, `Returns:`, `Examples:`, and `See Also:`. `Args:` and `Returns:` are descriptive sections, not the source of function shape. `Raises:` may exist when useful for authors, but standardized error behavior must live in registry metadata rather than in prose alone.

For ordinary built-in functions, typed constant specs are the canonical surface for registry facts, and `@function(SPEC)` is the canonical registration surface. A spec must be an inspectable typed value. The public helper should be ordinary code that delegates to the spec, so authors call a normal function while tooling inspects explicit data. Docs, LSP, typechecking, Prism, and Substrait lowering must all inspect the same resulting function catalog entry produced from the decorated helper and its spec.

The registration decorator must be narrow. `@function(SPEC)` links exactly one helper symbol to exactly one typed function spec. It must not duplicate lifecycle, determinism, null behavior, error behavior, aliases, or backend support. Those facts belong either in the spec, in real source-level alias bindings, or in backend capability registries.

Compatibility aliases must be real callable symbols rather than strings inside a function spec. For example, `mean = avg` should make `mean` an alias of the registered `avg` helper. The function catalog may record that alias after name resolution, but the aggregate spec must not contain `aliases=["mean"]`. Backend spellings and backend aliases remain backend capability concerns.

Generated reference pages must render lifecycle metadata in a consistent form. At minimum, generated pages must show when a function was introduced, when user-visible behavior changed, and whether the function is deprecated or removed. Spark's public function docs are useful prior art here: they expose "New in version", "Changed in version", and deprecation notes directly in generated API pages.

The docstring shape should be compact enough to live near the function declaration or registry entry, but structured enough that tooling can extract explanation and examples without natural-language guessing. Incan-standard docstrings are the canonical standard for explanation and examples; registry specs are canonical for machine facts. The following shape is illustrative only; constructor names and enum/model/class boundaries are implementation details:

```incan
pub const AVG = AggregateFunctionSpec(
    lifecycle=FunctionLifecycle(
        since=v0_2,
        changed=[
            FunctionChange(v0_3, "Added decimal return type rule."),
        ],
        deprecated=None,
    ),
    signature=FunctionSignature(
        args=[Arg("expr", ScalarExpr[number])],
        returns=AggregateMeasure[number],
    ),
    determinism=Determinism.Deterministic,
    null_behavior=NullBehavior.IgnoreInputNulls,
    error_behavior=ErrorBehavior.TypecheckNumericInput,
    substrait=SubstraitMapping.CoreFunction("avg"),
)

@function(AVG)
pub def avg(expr: ScalarExpr[number]) -> AggregateMeasure[number]:
    """
    Return the average non-null numeric value in each group.

    Args:
        expr: Numeric scalar expression evaluated for each input row.

    Returns:
        One aggregate measure whose value is null when no non-null input values exist.

    Examples:
        from pub::inql.functions import avg, col
        orders.group_by([col("customer_id")]).agg([avg(col("amount"))])

    See Also:
        sum, count, min, max
    """
    return AVG.call(expr)

mean = avg
```

The exact required section set may change before this RFC moves to Planned, but the format must adhere to Incan documentation standards. Tooling must not infer null behavior, error behavior, determinism, lifecycle status, or Substrait mapping from prose alone, and it must not infer examples or simple user-facing explanation from registry specs alone. A public helper must be explicitly linked to exactly one function spec through `@function(SPEC)`, and generation or validation must fail if the helper signature and spec signature drift apart. Generated documentation must derive argument and return shape from those signatures, then attach docstring descriptions where present.

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
- **Put all metadata in decorators.** Rejected because it makes declarations noisy and hides durable semantic metadata in annotation arguments. The decorator should register the helper; the typed spec should carry the machine-readable contract.

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

## Unresolved questions

- Should the registry be part of the InQL package API, compiler metadata, or both?
- Should compatibility aliases be importable by default, or should dialect compatibility require explicit opt-in?
- Which Incan-standard docstring sections should be mandatory for each function class so reference Markdown generation is reliable without making function declarations unreadable?
- What exact Substrait mapping model shape should typed function specs use before adapters declare backend execution support?
