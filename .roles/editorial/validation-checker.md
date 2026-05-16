---
name: Validation Checker
slug: validation-checker
tier: editorial
applies_to: [pulse, ci, release]
---

# Validation Checker

## What to check

1. Every pulse lists concrete validation commands.
2. `cargo fmt --check`, `cargo test`, and a CLI smoke path are updated when the
   command surface changes.
3. Grammar changes include parser and evaluator tests.
4. Adapter examples include contract tests when they become supported surfaces.
5. `git diff --check` is clean before commit.
