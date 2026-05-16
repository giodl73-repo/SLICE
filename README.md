# SLICE

SLICE is a low-layer Rust query and expression language for selecting typed
artifacts across the portfolio. It gives FLETCH, CROP, PEBBLE, PROOF, and
domain CLIs one reusable predicate kernel instead of many incompatible filter
grammars.

**Series:** Tools & Infrastructure.

SLICE is product-neutral: it parses and evaluates selectors over structured
values, while downstream repos own their schemas, domain names, ranking policy,
and user-facing commands.

## First contract

The first contract is intentionally small:

- dotted field paths, such as `metadata.status`;
- equality and inequality: `field eq 'value'`, `field ne 'value'`;
- numeric comparisons: `field gt 1`, `field ge 1`, `field lt 1`,
  `field le 1`;
- containment: `field has 'value'` for arrays, strings, and object keys;
- substring/object membership: `field contains 'value'`;
- conjunctions with `and`;
- optional typed field catalogs before evaluation.

```bash
slice eval --expr "metadata.tags has 'context' and metadata.status eq 'ready'" --input examples/pebble.json
```

## Formalism

SLICE is a typed selector pipeline: parse source syntax, normalize it, resolve
paths through a consumer-owned field catalog, type-check predicates, plan any
consumer-owned requirements, evaluate over adapter-provided values, and explain
the result. The detailed model is in
[`docs/specs/formalism.md`](docs/specs/formalism.md).

## Mock client

`slice-mock-client` is the local downstream validation harness. It runs the
current selector contract over Pebble-shaped metadata, CROP-like evidence units,
FLETCH-like active partitions, and ICELINES-like player rows. For FLETCH, SLICE
selects rows and the mock client performs the downstream fold into quiver
candidates, preserving the layer boundary.

```bash
cargo run -p slice-mock-client
```

## Rust

```rust
let expr = slice_core::parse("metadata.status eq 'ready'")?;
let ok = expr.matches(&serde_json::json!({
    "metadata": { "status": "ready" }
}));

let mut catalog = slice_core::FieldCatalog::new();
catalog
    .insert("metadata.status", slice_core::ValueType::String)
    .insert("stats.ppg", slice_core::ValueType::Number);
let selector = slice_core::compile("metadata.status eq 'ready' and stats.ppg ge 0.8", &catalog)?;
let explain = selector.explain();
```

`explain` is machine-readable, so downstream CLIs and agents can show which
fields, operators, and typed literals a selector depends on.

## Non-goals

- SLICE does not fetch data, build corpora, or cache artifacts.
- SLICE does not own CROP graph cuts, PEBBLE schema design, FLETCH manifests, or
  product-specific query surfaces.
- SLICE is not a general programming language; it is a portable selector and
  expression kernel.

## Validation

```bash
cargo fmt --check
cargo test
cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/pebble.json
cargo run -p slice-mock-client
```

## License

[MIT](LICENSE) - © 2026 Gio Della-Libera.
