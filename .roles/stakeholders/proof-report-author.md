---
name: PROOF Report Author
slug: proof-report-author
tier: stakeholder
primary_concern: report filtering without rendering leakage
---

# PROOF Report Author

## Primary concerns

- PROOF may use SLICE to filter generated reports and CROP-backed slices.
- PROOF still owns Markdown checking, rendering, and source fidelity.
- SLICE diagnostics should be suitable for generated docs and CI logs.
- Report filters should not pull PROOF rendering into `slice-core`.
