---
name: MDLOOM Report Author
slug: mdloom-report-author
tier: stakeholder
primary_concern: report filtering without rendering leakage
---

# MDLOOM Report Author

## Primary concerns

- MDLOOM may use SLICE to filter generated reports and MDCROP-backed slices.
- MDLOOM still owns Markdown checking, rendering, and source fidelity.
- SLICE diagnostics should be suitable for generated docs and CI logs.
- Report filters should not pull MDLOOM rendering into `slice-core`.
