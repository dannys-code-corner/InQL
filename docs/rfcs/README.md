# InQL RFCs

InQL uses its **own** RFC series (starting at 000), independent of the [Incan language RFCs][incan-rfcs].

**New RFC:** copy [TEMPLATE.md], name the file `NNN_short_slug.md`, pick the next number from the table (or from open issues), and open a PR. Section order and header fields follow that template. For workflow and conventions, see [Writing InQL RFCs].

| RFC            | Status      | Title                                                                                             |     |
| -------------- | ----------- | ------------------------------------------------------------------------------------------------- | --- |
| [000][rfc-000] | Planned     | Language specification — core model, naming, schema shapes, layer boundaries                      |     |
| [001][rfc-001] | In Progress | Dataset types and carriers (`DataSet[T]`, `BoundedDataSet[T]`, `UnboundedDataSet[T]`)             |     |
| [002][rfc-002] | In Progress | Apache Substrait — `Rel`-level contract, mapping catalog, binding boundaries                      |     |
| [003][rfc-003] | Implemented | `query {}` blocks — grammar, typing, Substrait lowering                                           |     |
| [004][rfc-004] | In Progress | Execution context — session, DataFusion, read/transform/write                                     |     |
| [005][rfc-005] | Blocked     | Pipe-forward relational syntax (`\|>`) — optional surface                                         |     |
| [006][rfc-006] | Blocked     | Promote unnest/explode to core Substrait lowering — blocked on upstream Substrait standardization |     |
| [007][rfc-007] | In Progress | Prism logical planning and optimization engine                                                    |     |
| [008][rfc-008] | Planned     | Optimizer boundary, statistics, cost-based optimization, and adaptive execution                   |     |
| [009][rfc-009] | Draft       | Session format handler registry (plugin-style source format registration)                         |     |
| [010][rfc-010] | Draft       | CSV dialect and interpretation contract                                                           |     |
| [011][rfc-011] | Draft       | Source discovery and parse-unit expansion                                                         |     |
| [012][rfc-012] | Implemented | Unified scalar expression surface                                                                 |     |
| [013][rfc-013] | Implemented | Function catalog program                                                                          |     |
| [014][rfc-014] | Implemented | Function registry and catalog governance                                                          |     |
| [015][rfc-015] | Implemented | Core scalar functions and operators                                                               |     |
| [016][rfc-016] | Implemented | Core aggregate functions                                                                          |     |
| [017][rfc-017] | Implemented | Aggregate modifiers                                                                               |     |
| [018][rfc-018] | Implemented | Common scalar function catalog                                                                    |     |
| [019][rfc-019] | Implemented | Window functions                                                                                  |     |
| [020][rfc-020] | Implemented | Nested data functions                                                                             |     |
| [021][rfc-021] | Implemented | Generator and table-valued functions                                                              |     |
| [022][rfc-022] | Implemented | Semi-structured and format functions                                                              |     |
| [023][rfc-023] | Implemented | Approximate and sketch functions                                                                  |     |
| [024][rfc-024] | Implemented | Function extension policy                                                                         |     |
| [025][rfc-025] | Implemented | Typed sketch logical values                                                                       |     |
| [026][rfc-026] | Implemented | Semi-structured variant logical values                                                            |     |
| [027][rfc-027] | Draft       | Relational evidence program                                                                       |     |
| [028][rfc-028] | Draft       | Semantic identity and target model                                                                |     |
| [029][rfc-029] | Draft       | Typed metadata attachments                                                                        |     |
| [030][rfc-030] | Draft       | Prism lineage graph                                                                               |     |
| [031][rfc-031] | Draft       | Local inspection APIs and artifacts                                                               |     |
| [032][rfc-032] | Draft       | Execution observations                                                                            |     |
| [033][rfc-033] | Draft       | Adapter requirements and coverage                                                                 |     |
| [034][rfc-034] | Draft       | Quality assertions and observations                                                               |     |
| [035][rfc-035] | Draft       | Governed attributes and policy checkpoints                                                        |     |
| [036][rfc-036] | Draft       | Governed plan bundle                                                                              |     |
| [037][rfc-037] | Draft       | Plan diff and blast-radius inputs                                                                 |     |
| [038][rfc-038] | Draft       | Evidence export bridges                                                                           |     |
| [040][rfc-040] | Draft       | Interoperability semantic profiles                                                                |     |
| [041][rfc-041] | Draft       | Prism plan ingress and external client frontends                                                   |     |

<!-- TODO: #7: auto populate this table (like how we do in incan) -->

**v0.1 scope:** RFCs 000–004 plus RFC 007. When those foundational RFCs are resolved (Draft → Planned → Implemented), InQL v0.1 is complete: authors can read data, write typed queries, lower through Prism to Substrait, execute through DataFusion, and write results.

New RFCs should follow [TEMPLATE.md] (aligned with Incan’s RFC structure, adapted for InQL).

<!-- References -->

[TEMPLATE.md]: TEMPLATE.md
[Writing InQL RFCs]: ../contributing/writing_rfcs.md
[rfc-000]: 000_inql_syntax.md
[rfc-001]: 001_inql_dataset.md
[rfc-002]: 002_apache_substrait_integration.md
[rfc-003]: 003_inql_query_blocks.md
[rfc-004]: 004_inql_execution_context.md
[rfc-005]: 005_inql_pipe_forward.md
[rfc-006]: 006_unnest_core_substrait.md
[rfc-007]: 007_prism_planning_engine.md
[rfc-008]: 008_optimizer_boundary_stats_cbo_aqe.md
[rfc-009]: 009_session_format_handler_registry.md
[rfc-010]: 010_csv_ingestion_contract.md
[rfc-011]: 011_source_discovery_contract.md
[rfc-012]: 012_unified_scalar_expression_surface.md
[rfc-013]: 013_function_catalog_program.md
[rfc-014]: 014_function_registry.md
[rfc-015]: 015_core_scalar_functions.md
[rfc-016]: 016_core_aggregate_functions.md
[rfc-017]: 017_aggregate_modifiers.md
[rfc-018]: 018_common_scalar_function_catalog.md
[rfc-019]: 019_window_functions.md
[rfc-020]: 020_nested_data_functions.md
[rfc-021]: 021_generator_table_functions.md
[rfc-022]: 022_semi_structured_format_functions.md
[rfc-023]: 023_approximate_sketch_functions.md
[rfc-024]: 024_function_extension_policy.md
[rfc-025]: 025_typed_sketch_logical_values.md
[rfc-026]: 026_semi_structured_variant_values.md
[rfc-027]: 027_relational_evidence_program.md
[rfc-028]: 028_semantic_identity_targets.md
[rfc-029]: 029_metadata_attachments.md
[rfc-030]: 030_prism_lineage_graph.md
[rfc-031]: 031_inspection_artifacts.md
[rfc-032]: 032_execution_observations.md
[rfc-033]: 033_adapter_requirements_coverage.md
[rfc-034]: 034_quality_assertions_observations.md
[rfc-035]: 035_governed_attributes_policy_checkpoints.md
[rfc-036]: 036_governed_plan_bundle.md
[rfc-037]: 037_plan_diff_blast_radius_inputs.md
[rfc-038]: 038_evidence_export_bridges.md
[rfc-040]: 040_interoperability_semantic_profiles.md
[rfc-041]: 041_prism_plan_ingress_frontends.md
[incan-rfcs]: https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs
