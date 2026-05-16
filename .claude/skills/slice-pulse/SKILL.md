---
name: slice-pulse
description: Execute the next SLICE wave pulse with docs, implementation, validation, and commit-ready updates.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - Bash
---

# SLICE Pulse

Use this skill for SLICE development pulses.

## Workflow

1. Read `context/waves/PHASES.md`.
2. Read the active wave `WAVE.md`.
3. Read the target pulse under `pulses/`.
4. Implement the smallest complete selector/expression slice.
5. Keep shared kernels free of consumer-specific behavior.
6. Update docs and wave/pulse status.
7. Run validation commands and `git diff --check`.
