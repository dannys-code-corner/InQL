# InQL RFCs

InQL uses its **own** RFC series (starting at 000), independent of the [Incan language RFCs][incan-rfcs].

**New RFC:** copy [TEMPLATE.md], name the file `NNN_short_slug.md`, pick the next number from the table (or from open issues), and open a PR. Section order and header fields follow that template. For workflow and conventions, see [Writing InQL RFCs].

| RFC            | Status      | Title                                                                                                                                                                                                      |     |
| -------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- |
| [000][rfc-000] | Planned     | Language specification — core model, naming, schema shapes, layer boundaries                                                                                                                               |     |
| [001][rfc-001] | Implemented | Dataset types and carriers (`DataSet[T]`, `BoundedDataSet[T]`, `UnboundedDataSet[T]`) — library package is **contract-complete** (types, `Self` method surface, `functions` imports); execution in RFC 004 |     |
| [002][rfc-002] | Planned     | Apache Substrait — `Rel`-level contract, mapping catalog, binding boundaries                                                                                                                               |     |
| [003][rfc-003] | Planned     | `query {}` blocks — grammar, typing, Substrait lowering                                                                                                                                                    |     |
| [004][rfc-004] | Planned     | Execution context — session, DataFusion, read/transform/write                                                                                                                                              |     |
| [005][rfc-005] | Blocked     | Pipe-forward relational syntax (`\                                                    | >`) — optional surface                                                                                             |     |

<!-- TODO: #7: auto populate this table (like how we do in incan) -->

**Order:** [RFC 000][rfc-000] is the foundational language specification. [RFC 001][rfc-001] defines the dataset type hierarchy. [RFC 002][rfc-002] defines the Substrait interchange contract. [RFC 003][rfc-003] defines the `query {}` surface that lowers to Substrait per RFC 002 over carriers from RFC 001. [RFC 004][rfc-004] completes the end-to-end story: session, read, execute, write. [RFC 005][rfc-005] specifies optional pipe-forward syntax outside the RFC 000–004 milestone and currently blocked on Incan RFC 040.

**v0.1 scope:** RFCs 000–004. When all five are resolved (Draft → Planned → Implemented), InQL v0.1 is complete: authors can read data, write typed queries, lower to Substrait, execute through DataFusion, and write results.

New RFCs should follow [TEMPLATE.md] (aligned with Incan’s RFC structure, adapted for InQL).

<!-- Link references (single place for targets) -->

[TEMPLATE.md]: TEMPLATE.md
[Writing InQL RFCs]: ../contributing/writing_rfcs.md
[rfc-000]: 000_inql_syntax.md
[rfc-001]: 001_inql_dataset.md
[rfc-002]: 002_apache_substrait_integration.md
[rfc-003]: 003_inql_query_blocks.md
[rfc-004]: 004_inql_execution_context.md
[rfc-005]: 005_inql_pipe_forward.md
[incan-rfcs]: https://github.com/dannys-code-corner/incan/tree/main/workspaces/docs-site/docs/RFCs
