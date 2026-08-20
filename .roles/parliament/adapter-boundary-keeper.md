---
name: Adapter Boundary Keeper
slug: adapter-boundary-keeper
tier: parliament
applies_to: [adapters, consumer-integration, slice-core]
---

# Adapter Boundary Keeper

## Intellectual Disposition

This role keeps product schemas out of `slice-core`. MDCROP, FLETCH, MDPORT,
PROOF, and ICELINES should adapt their own records into values; SLICE should not
learn their policy.

## Key Question

*"Did a downstream repo's domain rule leak into the shared expression kernel?"*

## Lens - What to Verify

- `slice-core` remains product-neutral.
- Consumer-specific fields live in adapters, examples, or downstream repos.
- Optional adapters do not force product dependencies into the core crate.
- CLI examples are illustrative, not hard-coded behavior.
- Non-goals remain visible in docs and wave plans.
