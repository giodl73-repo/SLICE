# SLICE Invariants

## SLICE-I-01: Field Catalogs Gate Typed Evaluation

**Status:** VERIFIED

**Invariant:** A selector compiled with a field catalog validates paths and
operator/value compatibility before row evaluation.

**Why it matters:** Consumers need predictable failures for unsupported fields
instead of silent row-level false negatives.

**Test:** `cargo test -p slice-core`.

**Evidence:** `docs/specs/formalism.md`, `docs/plans/consumer-migration.md`,
and `crates/slice-core/src/lib.rs`.

## SLICE-I-02: Missing Field Semantics Stay Deliberate

**Status:** VERIFIED

**Invariant:** Missing-field behavior, including MDCROP-compatible `ne`
handling in the adapter layer, remains explicit and tested.

**Why it matters:** Predicate parity can break quietly if absence, null, and
inequality collapse into one behavior.

**Test:** `cargo test -p slice-mock-client` and `cargo test -p slice-core`.

**Evidence:** `docs/plans/consumer-migration.md`,
`docs/specs/formalism.md`, and `crates/slice-mock-client/src/lib.rs`.

## SLICE-I-03: Report Schemas Stay Stable

**Status:** VERIFIED

**Invariant:** `slice.diagnostic.v1`, `slice.explain.v1`,
`slice.parse_explain.v1`, `slice.requirements.v1`, and `slice.fold.v1` retain
their documented compatibility semantics.

**Why it matters:** Agents and downstream CLIs depend on machine-readable
diagnostics and explanations to debug selectors before scanning data.

**Test:** `cargo test -p slice-core -p slice-cli`.

**Evidence:** `docs/compatibility.md`, `crates/slice-core/src/lib.rs`, and
`crates/slice-cli/src/main.rs`.

## SLICE-I-04: Mock Client Protects Consumer Shapes

**Status:** VERIFIED

**Invariant:** The local mock client exercises MDPORT-shaped, MDCROP-like,
FLETCH-like, and ICELINES-like rows without becoming those products.

**Why it matters:** Representative fixtures catch selector drift while keeping
real schema, ranking, folding, and command policy downstream.

**Test:** `cargo run -p slice-mock-client` and
`cargo test -p slice-mock-client`.

**Evidence:** `README.md`, `docs/plans/consumer-migration.md`, and
`crates/slice-mock-client/src/lib.rs`.

## SLICE-I-05: FLETCH Rehearsal Gates Foundation Changes

**Status:** VERIFIED

**Invariant:** Foundation changes that can affect real consumers require the
affected SLICE tests plus the documented FLETCH downstream rehearsal.

**Why it matters:** A shared selector crate can break cache-index and
active-partition users even when local fixtures still pass.

**Test:** `cargo test -p slice-core -p slice-sqlite -p slice-cli -p slice-mock-client`
plus documented FLETCH tests for affected changes.

**Evidence:** `docs/compatibility.md` and
`docs/plans/consumer-migration.md`.

## SLICE-I-06: Performance Claims Require Sized Evidence

**Status:** VERIFIED

**Invariant:** SLICE cannot make large-scan, FLETCH partition, SQLite table,
production-speed, or hot-path replacement claims unless the release record names
fixture size, row width, command, selector, catalog posture, memory posture,
downstream responsibilities, measured result or smoke threshold, and review
dispositions.

**Why it matters:** Tiny fixtures can prove selector semantics while hiding
memory or latency debt that only appears in real manifests, tables, or
consumer hot paths.

**Test:** `pwsh -NoProfile -File tests/check-performance-claim-boundary.ps1`.

**Evidence:** `SLICE-PF-06`, `docs/performance-claim-boundary.md`,
`docs/compatibility.md`, `.roles/ROLE.md`, and
`tests/check-performance-claim-boundary.ps1`.
