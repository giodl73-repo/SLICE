# Pulse 01: Workspace foundation

## Goal

Create the repository foundation and first tested SLICE selector contract.

## Changes

- Add Rust workspace with `slice-core` and `slice-cli`.
- Add README, product plan, license, wave docs, and repo skills.
- Add first minimal expression grammar and evaluator.
- Add a CLI smoke command over a Mdport-shaped JSON fixture.
- Register SLICE in TRACKER as a low-layer shared dependency.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/mdport.json`
- `git diff --check`

## Status

Done.
