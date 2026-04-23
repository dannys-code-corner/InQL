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
[incan-rfcs]: https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs
