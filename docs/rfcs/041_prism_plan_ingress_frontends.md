# InQL RFC 041: Prism plan ingress and external client frontends

- **Status:** Draft
- **Created:** 2026-05-30
- **Author(s):** Danny Meijer (@dannymeijer)
- **Related:**
  - InQL RFC 000 (core language model and layer boundaries)
  - InQL RFC 004 (execution context)
  - InQL RFC 007 (Prism logical planning and optimization engine)
  - InQL RFC 012 (unified scalar expression surface)
  - InQL RFC 013 (function catalog program)
  - InQL RFC 027 (relational evidence program)
  - InQL RFC 028 (semantic identity and target model)
  - InQL RFC 029 (typed metadata attachments)
  - InQL RFC 030 (Prism lineage graph)
  - InQL RFC 031 (local inspection APIs and artifacts)
  - InQL RFC 033 (adapter requirements and coverage)
  - InQL RFC 040 (interoperability semantic profiles)
- **Issue:** —
- **RFC PR:** —
- **Written against:** Incan v0.3-era InQL
- **Shipped in:** —

## Summary

This RFC defines Prism plan ingress and external client frontends for InQL. A frontend receives an external authoring or client protocol such as Spark Connect, SQL, or another unresolved relational plan surface, decodes it into a Prism-owned unresolved ingress plan, and asks Prism to analyze that plan into ordinary InQL relational semantics. The frontend may preserve client-origin evidence, client-session evidence, protocol diagnostics, and ingress coverage records, but it must not make the external protocol, Spark, Substrait, DataFusion, or any backend adapter the semantic owner of the plan.

## Motivation

InQL should be able to interoperate with established client APIs without pretending that those APIs own InQL's semantics. Spark Connect is the clearest pressure: a PySpark client can submit plan-shaped calls over a protocol boundary, and those calls may depend on client session state such as configuration, current catalog, temporary views, or function aliases. InQL should not route those calls through Spark just to recover meaning later. Prism should receive an unresolved representation, resolve names and functions, apply InQL semantic rules under an explicit profile and session context, and then continue through the normal planning, evidence, Substrait, and execution paths.

Without a first-class ingress contract, external API support will be squeezed into session adapters, backend adapters, Substrait metadata, or compatibility flags. That would hide the real boundary. Execution adapters run plans. Plan ingress frontends receive external plan requests and let Prism create InQL plans.

## Goals

- Define plan ingress frontends as distinct from execution session adapters.
- Define a Prism-owned unresolved ingress plan model for external client plans.
- Preserve client-origin evidence without treating external node identifiers as InQL semantic identities.
- Represent client session state that can affect Prism analysis.
- Allow Spark Connect-style clients to submit supported relation, expression, and command calls that Prism analyzes directly.
- Require structured unsupported-feature diagnostics and ingress coverage records.
- Connect ingress analysis to semantic profiles for client protocol behavior such as case sensitivity, function aliases, coercion, temporal behavior, and null ordering.
- Keep DataFusion and other execution backends behind the existing execution adapter boundary.

## Non-Goals

- Defining full Spark API parity.
- Defining every Spark Connect protobuf message, gRPC service method, or streaming transport detail.
- Making Spark, PySpark, Spark Connect, DataFusion, or Substrait the source of InQL relational meaning.
- Defining SQL transpilation as the internal planning model.
- Reimplementing every external client session lifecycle rule.
- Defining a hosted compatibility service or global conformance registry.
- Requiring every InQL deployment to expose an external client protocol.

## Guide-level explanation (how authors think about it)

An external client frontend lets an existing tool submit relational intent while InQL keeps Prism as the planner:

```incan
from pub::inql.frontends.spark import spark_connect_frontend
from pub::inql.session import datafusion_session

frontend = spark_connect_frontend(session_factory=datafusion_session)
frontend.serve("127.0.0.1:15002")
```

A PySpark client may send a supported Spark Connect plan to that endpoint. The frontend decodes the request into a Prism ingress plan, Prism analyzes it, and the selected InQL session executes the resulting plan through the normal adapter path.

The important user model is not that InQL becomes Spark internally. The model is that InQL can speak a client protocol at the edge while keeping Prism responsible for names, functions, types, lineage, diagnostics, evidence, and execution requirements.

## Reference-level explanation (precise rules)

InQL must define a plan ingress frontend boundary. A frontend may parse, decode, authenticate, route, frame client requests, and maintain client session state. It must not resolve relational semantics by delegating to an external engine when Prism can represent the requested plan.

An ingress frontend must produce a Prism-owned unresolved ingress plan or a structured unsupported diagnostic. An unresolved ingress plan must represent at least:

- ingress request identity
- ingress frontend identity
- client session identity when the request is session-scoped
- client protocol and protocol version when available
- requested semantic profile when available
- unresolved relation nodes
- unresolved expression nodes
- unresolved command nodes when the protocol includes commands
- client-origin references
- diagnostics
- ingress coverage records

Prism analysis of an ingress plan must produce ordinary Prism semantic targets, relation plans, expression plans, diagnostics, lineage inputs, adapter requirements, and inspection evidence. External client node identifiers may be preserved as origin references, but they must not replace Prism semantic target identities.

Client session state that can affect analysis must be represented as client session targets, typed attachments, profile evidence, or analyzer binding references. This includes current catalog or namespace, temporary relation names, session variables, function registrations, case sensitivity, ANSI or compatibility modes, timezone settings, and other frontend-specific state that changes name resolution, type coercion, function lookup, or expression semantics.

Analyzer bindings that depend on client session state must reference the relevant client session target or profile assessment. A later inspection should be able to explain that a relation, function, alias, identifier, or coercion resolved under a specific client session context without treating that client session as the source of relational meaning.

Ingress frontends must distinguish at least the following coverage states for protocol features:

- supported: Prism can represent and analyze the feature under the selected profile
- partially_supported: Prism can represent the feature only under recorded restrictions
- unsupported: Prism cannot represent or analyze the feature
- unknown: InQL cannot determine support for the feature

Unsupported and unknown ingress features must be reported before execution when they affect plan semantics. They must not be silently lowered to backend-specific behavior.

Spark Connect compatibility must be modeled as a frontend protocol profile plus ingress coverage, not as a backend adapter capability alone. A Spark Connect frontend may accept supported relation and expression calls, reject unsupported calls, and report coverage for omitted protocol areas. It must not require Spark's engine to produce the semantic plan that Prism will execute.

Semantic profiles may affect ingress analysis. Profile dimensions may control identifier resolution, case sensitivity, function aliases, type coercion, null and NaN behavior, timestamp semantics, decimal behavior, ANSI mode, and other compatibility-sensitive rules. Profile evidence must be explicit when the frontend uses those rules.

Commands that do not describe relational computation, such as session configuration, catalog inspection, temporary view registration, cache control, or client lifecycle operations, must be represented as command ingress nodes, mapped to explicit InQL client session or execution session behavior, or rejected with structured diagnostics. Accepted commands that mutate client session state must produce client session evidence. They must not be disguised as ordinary relational plan nodes.

Plan ingress evidence must be inspectable. Tools must be able to see which frontend produced a plan, which client session context affected analysis, which client protocol features were used, which features were unsupported or partially supported, which profile governed analysis, and how client-origin references map to Prism targets.

## Design details

### Syntax

This RFC introduces no InQL authoring syntax. Frontends are package or service APIs around Prism and Session.

### Semantics

Plan ingress is a semantic boundary before Prism analysis. It is not an execution boundary. The frontend receives external input, Prism owns analysis, and Session plus execution adapters own execution.

The unresolved ingress model should be general enough for Spark Connect, SQL parsers, notebook clients, and future external plan protocols, but each frontend must declare its own protocol coverage, session-state model, and profile assumptions.

### Interaction with other InQL surfaces

Method chains, `query {}` blocks, SQL frontends, and Spark Connect frontends may all produce Prism plans. Equivalent relational intent should converge on comparable Prism targets after analysis, subject to explicit profile differences.

Prism lineage must preserve client-origin relationships where available. Those relationships explain where a Prism target came from, but they are not value lineage by themselves.

Inspection APIs and artifacts must expose ingress diagnostics, origin mappings, selected profiles, relevant client session targets, and ingress coverage when a plan came through a frontend.

Adapter requirements and coverage remain execution-facing evidence. Ingress coverage is frontend-facing evidence. A Spark Connect relation node being accepted by a frontend is not the same as DataFusion being able to execute the resulting plan.

### Compatibility / migration

Existing InQL plans and sessions remain valid without ingress evidence. Frontends are additive. Tools that require client-origin evidence must report missing ingress evidence as unsupported or unknown rather than inferring it from display names or backend plans.

## Alternatives considered

- **Use Spark as the planner.** Rejected because that would make Spark the semantic owner and reduce Prism to an execution bridge.
- **Translate Spark Connect directly to Substrait.** Rejected because Substrait is an interchange boundary, not the full InQL semantic analysis and evidence model.
- **Treat Spark Connect support as a backend adapter.** Rejected because receiving client plan calls and executing an analyzed plan are different boundaries.
- **Only support InQL-native authoring.** Rejected because external client protocols are valuable when they feed Prism honestly instead of bypassing it.
- **Accept unsupported calls and hope the backend handles them.** Rejected because unknown frontend semantics must be visible before execution.

## Drawbacks

- Frontends require protocol-specific maintenance.
- Spark compatibility pressure can pull non-portable semantics into the system unless profiles and coverage stay explicit.
- The unresolved ingress model adds another planning representation before Prism's analyzed plan.
- Early frontend support will be partial, which requires clear diagnostics and compatibility documentation.

## Layers affected

- **InQL specification** — plan ingress, frontend coverage, origin mapping, and Prism analysis boundaries become normative vocabulary.
- **InQL library package** — frontend APIs, unresolved ingress plan records, diagnostics, and inspection records must be exposed through public modules where implemented.
- **Execution / interchange** — Session and backend adapters execute Prism-owned plans and may report adapter coverage, but they do not own ingress semantics.
- **Documentation** — docs must distinguish external client protocol support from Spark engine compatibility, Substrait interchange, and DataFusion execution.

## Unresolved questions

- Which Spark Connect relation and expression nodes belong in the first supported ingress slice?
- Should ingress frontends live in the core package or optional integration packages?
- What is the minimum common unresolved ingress model shared by Spark Connect, SQL, and future client protocols?
- Which client session fields are required in the first Spark Connect-compatible profile without importing Spark's session model wholesale?

<!-- When every question is resolved, rename this section to **Design Decisions**, group answers under ### Resolved, and remove this comment. -->
