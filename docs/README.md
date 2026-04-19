# InQL documentation

This directory holds the public documentation for the InQL project.

Use the docs tree like this:

- **Language reference:** current package/API contracts under [language/reference/][language-reference]
- **Language explanation:** conceptual guidance and usage framing under [language/explanation/][language-explanation]
- **Architecture:** repository and system boundaries in [architecture.md][architecture]
- **RFCs:** design records and normative proposals in [rfcs/][rfcs]
- **Release notes:** shipped and user-visible changes in [release_notes/][release-notes]
- **Contributing:** contributor workflow, especially RFC process, in [contributing/][contributing]

## Recommended reading paths

### Learn the current package surface

1. [Language overview][language-overview]
2. [Dataset types (Explanation)][dataset-explanation]
3. [Execution context (Explanation)][execution-explanation]
4. [Dataset types (Reference)][dataset-reference]
5. [Execution context (Reference)][execution-reference]

### Understand the system design

1. [InQL architecture][architecture]
2. [RFC index][rfcs-index]
3. Relevant RFCs for deeper normative context

### Work on the spec

1. [RFC index][rfcs-index]
2. [How to write an RFC][writing-rfcs]

> Note: When a standalone docs site is added, `docs/` remains the content root. The structure here should already follow the same content model used in Incan: reference, explanation, architecture/contributing, RFCs, and release notes.

<!-- References -->
[language-reference]: language/reference/
[language-explanation]: language/explanation/
[architecture]: architecture.md
[rfcs]: rfcs/README.md
[release-notes]: release_notes/
[contributing]: contributing/
[language-overview]: language/README.md
[dataset-explanation]: language/explanation/dataset_types.md
[execution-explanation]: language/explanation/execution_context.md
[dataset-reference]: language/reference/dataset_types.md
[execution-reference]: language/reference/execution_context.md
[rfcs-index]: rfcs/README.md
[writing-rfcs]: contributing/writing_rfcs.md
