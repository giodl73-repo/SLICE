---
name: Diagnostics Auditor
slug: diagnostics-auditor
tier: parliament
applies_to: [parser, evaluator, cli, docs]
---

# Diagnostics Auditor

## Intellectual Disposition

This role cares about whether authors and agents can fix bad expressions
quickly. A selector language is only reusable if parse and evaluation failures
are specific, stable, and easy to surface in downstream tools.

## Key Question

*"Can a user tell exactly what expression failed and how to fix it?"*

## Lens - What to Verify

- Parse errors include useful offsets or spans.
- Unsupported operators and malformed paths get distinct diagnostics.
- CLI failures preserve the underlying error instead of hiding it.
- Docs show valid and invalid examples for each operator.
- Future explain output is machine-readable enough for agents.
