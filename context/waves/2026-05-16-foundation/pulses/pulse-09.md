# Pulse 09: First consumer adoption

## Goal

Record the first real downstream SLICE adoption.

## Outcome

MDCROP commit `02006c9` adopted `slice-core` for `mdcrop.view.v1`
`frontmatter_query` parsing and evaluation.

## Boundary

MDCROP derives its own field catalog, materializes Markdown frontmatter into row
values, preserves missing-field `ne`, and keeps view recipes, graph selection,
status policy, prefix caches, and rendering local to MDCROP.

## Validation

Validated in MDCROP:

- `cargo fmt --check`
- `cargo test --workspace`
- `cargo run -p mdcrop-cli -- view --file examples\proof-fixture\proof-ready-view.json`
- `git diff --check`

## Status

Done.
