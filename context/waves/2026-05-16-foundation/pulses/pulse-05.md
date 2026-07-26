# Pulse 05: Consumer migration plan

## Goal

Choose the first real downstream migration target and define the boundaries that
keep SLICE product-neutral.

## Decision

CROP frontmatter-query parity is the first migration candidate. It is closest to
SLICE's current typed row-predicate grammar and can prove value without moving
CROP graph, corpus-health, status, prefix-cache, or view policy into
`slice-core`.

## Follow-on order

1. CROP frontmatter query parity.
2. Mdport metadata selector examples.
3. FLETCH manifest and partition selectors.
4. MDLOOM report and CROP-backed slice filters.
5. ICELINES simple bio/stat adapter.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Status

Done.
