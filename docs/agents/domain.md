# Domain Docs

How engineering skills consume this repository's domain documentation.

## Before exploring

- Read `CONTEXT.md` at the repository root when it exists.
- Read relevant ADRs under `docs/adr/` when they exist.
- Proceed silently when either location is absent. Domain-modeling skills create these files lazily when terminology or decisions are resolved.

## Layout

This is a single-context repository:

/
├── CONTEXT.md
├── docs/adr/
└── src/

## Vocabulary

Use domain terms as defined in `CONTEXT.md`. If a needed concept is absent, reconsider whether it belongs to the project or note the gap for domain modeling.

## ADR conflicts

Explicitly surface output that contradicts an existing ADR rather than silently overriding the decision.
