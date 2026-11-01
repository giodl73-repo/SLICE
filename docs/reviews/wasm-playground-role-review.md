# WASM playground producer review

Method: one producer applied the repository's parliament and editorial checklists.
These are review lenses, not independent reviewers or human/publication approval.
Entry: `78e488c2b0b8299f8703d45a45f445055992fd7d`. Disposition: local candidate
passes with the scope/limits below; hosted deployment must use its own exact
Actions/provenance receipt.

| Lens | Review and evidence | Disposition |
| --- | --- | --- |
| Expression Kernel Keeper | `slice-core` source unchanged. Browser adapter delegates parse, typed explain/requirements and matching to the existing API. Six adapter tests retain direct-core parity, diagnostics, explicit row order and refusal boundaries; thirteen native/WASM fixtures include quote-boundary cases. | Pass, bounded adapter only |
| Adapter Boundary Keeper | `slice-web` depends on core/serde and optional bindgen. No SQLite, native scan, fetch/cache policy, ranking or consumer schemas enter the portable dependency path. UI presentation is in `web/`, outside the kernel. | Pass |
| Diagnostics Auditor | Original `slice.diagnostic.v1` fields/byte offsets survive. UTF-8 byte offsets map to DOM cursor units correctly. Invalid JSON/catalogs, invalid file UTF-8, limits and engine failures are separate. Rendering uses text nodes, not user HTML. | Pass |
| Performance Engineer | 1 MiB/2,000-row/24-level/50,000-node input bounds and selector/catalog bounds are explicit. The selector is parsed once and reused. Computation is in a worker with UI recovery deadlines. Post-parse limits are not allocation quotas; no large-scan or production speed claim. | Pass with limits |
| Scope Keeper | A public query front door exposes the existing kernel. No document renderer, database executor, domain policy or downstream runtime adoption was added. First-party source remains MIT; upstream legal notices retain their own scope. | Pass |
| Contract Checker | Existing grammar/operators and core source stay unchanged. Admission follows actual core token/quote boundaries, not JSON escaping. Typed catalogs validate selectors, not entire row schemas. Record order is retained; JSON formatting/key order need not be. | Pass |
| Validation Checker | Locked workspace tests, host/WASM strict Clippy and release WASM build pass. Browser cases cover actual native/WASM canonical downloads, own input, malformed input, limits, errors/recovery, keyboard/mobile, source/WASM/license routes and no-JS static access. Fault injection is labelled; it is not an algorithm substitute. | Pass, hosted release still separately evidenced |

## Findings resolved during implementation

- A timed-out/terminated worker must not be re-enabled by a queued old ready
  event. Generation fencing and a controlled late-event case protect this.
- Edits must not cancel startup or computation deadlines. Retry preserves all
  three editors; Restore example is independent. A late result cannot repopulate
  an edited draft.
- A reset during a pending local file read must release the Run button and
  discard late file text. Invalid UTF-8 must retain the previous draft.
- Core quotes do not use JSON backslash escaping, and quote characters can
  occur inside identifiers. The resource scanner now follows those actual
  lexical boundaries; nested syntax cannot hide behind a different quote rule.
- The old CLI test's unnecessary `vec!` became an array for the pinned compiler's
  strict Clippy. Production CLI/kernel behavior is unchanged.
- Clean source ZIPs and direct Rust downloads come from the exact Git commit.
  The builder checks ZIP members against build source. Dirty development builds
  are labelled; the publication job refuses them.
- Authored files normalize to LF. Scoped attributes preserve only verbatim
  upstream legal documents and their existing whitespace.

## Evidence limits

Headless desktop/narrow-screen checks are not real phone hardware acceptance.
Startup samples include browser/network/worker/UI costs and are not Rust CPU
benchmarks or SLAs. Example rows are illustrative, not policy/data admission.
Closing the tab discards drafts; no persistence is promised. Native APIs and
large corpus/database execution remain independent. Exact public release and
live artifact identity are recorded by the Pages workflow, public provenance
and TRACKER's adoption wave after publication.
