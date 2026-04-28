# InQL RFCs

InQL uses its **own** RFC series (starting at 000), independent of the [Incan language RFCs][incan-rfcs].

**New RFC:** copy [TEMPLATE.md], name the file `NNN_short_slug.md`, pick the next number from the table (or from open issues), and open a PR. Section order and header fields follow that template. For workflow and conventions, see [Writing InQL RFCs].

| RFC            | Status      | Title                                                                                             |     |
| -------------- | ----------- | ------------------------------------------------------------------------------------------------- | --- |
| [000][rfc-000] | Planned     | Language specification — core model, naming, schema shapes, layer boundaries                      |     |
| [001][rfc-001] | In Progress | Dataset types and carriers (`DataSet[T]`, `BoundedDataSet[T]`, `UnboundedDataSet[T]`)             |     |
| [002][rfc-002] | In Progress | Apache Substrait — `Rel`-level contract, mapping catalog, binding boundaries                      |     |
| [003][rfc-003] | Planned     | `query {}` blocks — grammar, typing, Substrait lowering                                           |     |
| [004][rfc-004] | In Progress | Execution context — session, DataFusion, read/transform/write                                     |     |
| [005][rfc-005] | Blocked     | Pipe-forward relational syntax (`\|>`) — optional surface                                         |     |
| [006][rfc-006] | Blocked     | Promote unnest/explode to core Substrait lowering — blocked on upstream Substrait standardization |     |
| [007][rfc-007] | In Progress | Prism logical planning and optimization engine                                                    |     |
| [008][rfc-008] | Planned     | Optimizer boundary, statistics, cost-based optimization, and adaptive execution                   |     |
| [009][rfc-009] | Draft       | Session format handler registry (plugin-style source format registration)                         |     |
| [010][rfc-010] | Draft       | CSV dialect and interpretation contract                                                           |     |
| [011][rfc-011] | Draft       | Source discovery and parse-unit expansion                                                         |     |
| [012][rfc-012] | Draft       | Unified scalar expression surface                                                                 |     |
| [013][rfc-013] | Draft       | Function catalog program                                                                          |     |
| [014][rfc-014] | Draft       | Function registry and catalog governance                                                          |     |
| [015][rfc-015] | Draft       | Core scalar functions and operators                                                               |     |
| [016][rfc-016] | Draft       | Core aggregate functions                                                                          |     |
| [017][rfc-017] | Draft       | Aggregate modifiers                                                                               |     |
| [018][rfc-018] | Draft       | Common scalar function catalog                                                                    |     |
| [019][rfc-019] | Draft       | Window functions                                                                                  |     |
| [020][rfc-020] | Draft       | Nested data functions                                                                             |     |
| [021][rfc-021] | Draft       | Generator and table-valued functions                                                              |     |
| [022][rfc-022] | Draft       | Semi-structured and format functions                                                              |     |
| [023][rfc-023] | Draft       | Approximate and sketch functions                                                                  |     |
| [024][rfc-024] | Draft       | Function extension policy                                                                         |     |

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
[incan-rfcs]: https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs
