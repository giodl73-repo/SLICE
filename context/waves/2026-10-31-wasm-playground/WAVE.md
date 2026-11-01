# WASM query playground

Entry: public main `78e488c2b0b8299f8703d45a45f445055992fd7d`.
Owner-selected continuation of TRACKER's deferred SLICE/MDPORT/MDCROP sequence.

Reader task: paste bounded JSON object records, evaluate an existing selector,
optionally supply a typed field catalog, inspect native explain/requirements
and structured diagnostics, then download matching records. Default examples
must work immediately through actual Rust WebAssembly inside a module worker.

Affected crates: new `slice-web` native/wasm adapter only; no `slice-core`
grammar/evaluation or `slice-sqlite` changes. Other consumer repos are no-op.
No database execution, file scans, fetch/cache policy, ranking or domain schemas.

Pulse 01: adapter, bounded UI/worker, self-service source/artifact downloads,
reproducible build, native parity references, real-WASM browser/recovery checks,
role review and exact Pages release evidence. Implementation and local
qualification complete; hosted publication uses the validated workflow artifact.

Validation: pinned Rust fmt, locked workspace tests/Clippy, target-specific WASM
Clippy and release build; pinned bindgen and Playwright; native/WASM canonical
parity, malformed inputs, resource bounds, download, stale-output suppression,
startup/compute recovery, keyboard/mobile and project-prefix cases. Build from
clean Git for publication; enforce a 5 MB static delivery budget. Observed fixture
timings are environment-specific; no production/large-scan speed claim.

Read .roles parliament then editorial checklists before completion; record
producer-applied review, not independent expert or human acceptance.

## Implementation and local qualification

`slice-web` exposes one canonical JSON function to both the native
`slice-playground` CLI and bindgen. The existing kernel is unchanged; the only
old Rust edit is a test-only Vec-to-array Clippy fix in `slice-cli`.

Forty-five workspace tests (including six adapter cases), strict native/WASM
Clippy and a pinned release WASM build pass. All 28 local browser cases pass
with zero retries/skips/flaky results: thirteen native/WASM canonical fixtures
plus own-input, invalid encoding, resource, source/download, deadline, stale
event/result, no-JS and keyboard/mobile checks. Fault injection controls message
delivery or availability; it never replaces selector evaluation.

Desktop/narrow-screen input, result and entry screenshots were inspected. Text
and controls are readable without horizontal page overflow; the mobile layout
uses vertical scrolling and the explain panel has its own scroll area.
Local sample observations and pre-commit artifact pins are in
`local-qualification.json`, explicitly separate from clean hosted release custody.
[Producer role review](../../../docs/reviews/wasm-playground-role-review.md).

The first browser attempt lacked the pinned browser executable; no app cases
ran. Installing the exact Playwright revision resolved that environment issue.
An added no-JS test exposed reliance on a noscript text selector; the fallback
is now a static explanatory paragraph hidden only by successfully running JS.
The full current suite passes. Public source, compiled WASM, build guide,
provenance and notices remain independently available.

Hosted identity and exact live-file custody belong to the Pages Actions run,
public `provenance.json`, and TRACKER's
`context/waves/2026-10-31-slice-wasm-playground/` release receipt. No further
source commit is needed merely to repeat its own deployment SHA in this file.
