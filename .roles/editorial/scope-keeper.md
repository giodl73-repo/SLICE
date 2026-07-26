---
name: Scope Keeper
slug: scope-keeper
tier: editorial
applies_to: [spec, wave, pulse, implementation]
---

# Scope Keeper

Form gate, not substance gate. Runs after parliament before a wave or pulse is
treated as ready.

## What to check

1. Does the artifact keep SLICE product-neutral?
2. Does it avoid embedding consumer-specific schemas in `slice-core`?
3. Does it distinguish core grammar from optional adapters/examples?
4. Does it keep fetching, caching, graph cuts, rendering, and domain ranking out
   of SLICE?

## What NOT to do

Do not reject concrete MDCROP, MDPORT, FLETCH, MDLOOM, or ICELINES examples just
because they are concrete. Reject them only when they become shared-core policy.
