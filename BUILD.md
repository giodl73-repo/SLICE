# Build and validate the query playground

Use the committed Rust 1.95.0 toolchain/lockfile, Python 3.11+, Node 22 and
`wasm-bindgen-cli` **0.2.127** (matching the optional Rust dependency).
Install that exact CLI from its official release or with
`cargo install wasm-bindgen-cli --version 0.2.127 --locked`. The Pages workflow
downloads a hash-pinned Linux binary. Do not replace it with an unpinned latest.

```text
cargo fmt --all --check
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
cargo clippy --locked -p slice-web --lib --features wasm --target wasm32-unknown-unknown -- -D warnings
npm ci
npx playwright install chromium
python -B tools/build-pages.py
npm run test:pages
```

Serve `dist` on loopback and open `/SLICE/`, not the filesystem's HTML file.
`python -B tools/serve-pages.py` serves http://127.0.0.1:4177/SLICE/.
Optional `SLICE_BASE_URL` targets the deployed project URL for the same browser
suite; fault tests intercept only their own isolated browser requests.

The builder compiles only `slice-web` plus its portable dependencies to WASM.
SQLite, filesystem scans and downstream products are not in that dependency path.
Native fixture responses are generated through the same adapter and compared
byte-for-byte with executing WASM report downloads. Native CLI example:

```text
cargo run --locked -p slice-web --bin slice-playground -- web/example-request.json
```

Outputs include the compiled WASM, local worker/UI, exact commit provenance,
first-party MIT license, dependency/Rust-library notices and a source ZIP.
Source links work independently of WASM. A dirty local build is marked as such;
publication requires a clean checkout. Clean source ZIPs come from `git archive
HEAD`, and the workflow deploys only its tested artifact on main. The entire
static site must remain below 5,000,000 bytes. Generated outputs stay ignored.

The source ZIP preserves this repo's first-party MIT scope. Third-party Rust
library notices under `tools/licenses/rust` and collected dependency notices
retain their original licenses; they are not relicensed by the root MIT grant.
