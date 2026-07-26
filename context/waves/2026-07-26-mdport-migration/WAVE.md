---
wave: mdport-migration
date_open: 2026-07-26
status: done
source_request: "Rename PEBBLE and pebble.v1 to MDPORT and mdport.v1."
---

# Wave: MDPORT migration

SLICE examples, fixtures, adapter language, and review roles now use MDPORT and
`mdport.v1`. SLICE remains a product-neutral selector kernel and does not own the
MDPORT schema.

Validation: `cargo test`.
