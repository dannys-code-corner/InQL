# Substrait conformance corpus (Reference)

This page documents where InQL's Substrait conformance scenarios live and how they are represented. The normative Substrait contract still lives in [InQL RFC 002][rfc-002], with operator-level mappings in the [Substrait operator catalog][ref-operator-catalog].

The corpus is the machine-readable validation layer for the RFC 002 v1 implementation profile ("v1 implementation profile (InQL code path)").

## Source of truth

The canonical conformance corpus is implemented in InQL package code:

- `src/substrait/conformance.incn`

The corpus uses typed models/enums (`SubstraitConformanceScenario`, `ConformanceStatus`, `ConformanceRel`, and related enums) for machine-readable contracts, and uses module/API docstrings for the human-readable contract.

Canonical operation semantics flow through `src/dataset/ops.incn`, while proto-backed Substrait emission and plan inspection live in `src/substrait/plan.incn`.

For the current package-level RFC 002 profile, conformance checks are intentionally split between:

- real boundary facts that the package can prove now (relation kind, read kind, join variant, set operation, reference ordinal, extension URI presence)
- richer planning semantics that remain deferred to future `query {}` lowering and Prism work

## Representation contract

Each scenario is selected by `CoreScenarioKey` and materialized via `core_scenario(key) -> SubstraitConformanceScenario`.

- Machine-readable fields include strongly typed enums for status/profile/relation/portability fields.
- Tag and reference collections are modeled as list-backed newtypes (`ConformanceCapabilityTags` and `ConformanceReferences`) rather than pipe-delimited strings.
- Human-readable content remains in docs plus descriptive scenario text fields (`intent`, `required_rel_shape`, and `expected_constraints`).

## Scenario ID convention

`scenario_id` values must be stable and use this convention:

```text
inql.substrait.<taxonomy-group>.<capability-slug>.<nnn>
```

The numeric suffix is immutable after publication. If requirements change incompatibly, add a new scenario ID instead of mutating semantics under an existing ID.

## Current core coverage

Core scenarios currently implemented in `src/substrait/conformance.incn`:

| Scenario ID                                            | Selector                                                   | Primary core `Rel` coverage |
| ------------------------------------------------------ | ---------------------------------------------------------- | --------------------------- |
| `inql.substrait.core.read_named_table.001`             | `core_scenario(CoreScenarioKey.ReadNamedTable)`            | `ReadRel` (`NamedTable`)    |
| `inql.substrait.core.read_local_files.001`             | `core_scenario(CoreScenarioKey.ReadLocalFiles)`            | `ReadRel` (`LocalFiles`)    |
| `inql.substrait.core.read_virtual_table.001`           | `core_scenario(CoreScenarioKey.ReadVirtualTable)`          | `ReadRel` (`VirtualTable`)  |
| `inql.substrait.core.filter_rows.001`                  | `core_scenario(CoreScenarioKey.FilterRows)`                | `FilterRel`                 |
| `inql.substrait.core.project_computed_columns.001`     | `core_scenario(CoreScenarioKey.ProjectComputedColumns)`    | `ProjectRel`                |
| `inql.substrait.core.join_rel_variants.001`            | `core_scenario(CoreScenarioKey.JoinRelVariants)`           | `JoinRel`                   |
| `inql.substrait.core.cross_rel_cartesian.001`          | `core_scenario(CoreScenarioKey.CrossRelCartesian)`         | `CrossRel`                  |
| `inql.substrait.core.aggregate_grouping_sets.001`      | `core_scenario(CoreScenarioKey.AggregateGroupingSets)`     | `AggregateRel`              |
| `inql.substrait.core.sort_rel_ordering.001`            | `core_scenario(CoreScenarioKey.SortRelOrdering)`           | `SortRel`                   |
| `inql.substrait.core.fetch_rel_limit_offset.001`       | `core_scenario(CoreScenarioKey.FetchRelLimitOffset)`       | `FetchRel`                  |
| `inql.substrait.core.set_rel_operations.001`           | `core_scenario(CoreScenarioKey.SetRelOperations)`          | `SetRel`                    |
| `inql.substrait.core.reference_rel_shared_subplan.001` | `core_scenario(CoreScenarioKey.ReferenceRelSharedSubplan)` | `ReferenceRel`              |

## Taxonomy values

The same taxonomy remains in force for scenario declarations:

- `status`: `ConformanceStatus.Core`, `ConformanceStatus.Extension`, `ConformanceStatus.Gap`, `ConformanceStatus.OptionalMutation`
- `profile_tags`: `ConformanceProfileTag.ReadQueryCore`, `ConformanceProfileTag.OptionalMutation`, `ConformanceProfileTag.GapPolicy`, `ConformanceProfileTag.ReadBindingBoundary`
- `portability`: `ConformancePortability.Portable`, `ConformancePortability.ConsumerConditional`, `ConformancePortability.NonPortable`

## Tooling expectation

Downstream tooling should consume scenario functions and model fields from `src/substrait/conformance.incn` as the machine contract, rather than JSON sidecar files.

Conformance validation for the v1 profile is expected to run against canonical operation functions in `src/dataset/ops.incn`, emitted proto-backed plans from `src/substrait/plan.incn`, and typed model/schema helpers where needed.

The current `ProjectRel` and `AggregateRel` scenarios are boundary-shape scaffolds, not proof that full computed-column, window, grouping-set, or distinct semantics are already implemented in package code.

<!-- Link references (single place for targets) -->

[rfc-002]: ../../rfcs/002_apache_substrait_integration.md
[ref-operator-catalog]: ./operator_catalog.md
