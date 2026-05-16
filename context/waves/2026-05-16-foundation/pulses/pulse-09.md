# Pulse 09: First consumer adoption

## Goal

Record the first real downstream SLICE adoption.

## Outcome

CROP commit `02006c9` adopted `slice-core` for `crop.view.v1`
`frontmatter_query` parsing and evaluation.

## Boundary

CROP derives its own field catalog, materializes Markdown frontmatter into row
values, preserves missing-field `ne`, and keeps view recipes, graph selection,
status policy, prefix caches, and rendering local to CROP.

## Validation

Validated in CROP:

- `cargo fmt --check`
- `cargo test --workspace`
- `cargo run -p crop-cli -- view --file examples\proof-fixture\proof-ready-view.json`
- `git diff --check`

## Status

Done.
