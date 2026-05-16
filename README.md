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
- ranges: `field between 1 and 10`;
- membership: `field in ['CROP', 'PROOF']`, `field not in ['blocked']`;
- null queries: `field is null`, `field is not null`;
- containment: `field has 'value'` for arrays, strings, and object keys;
- array/string quantifiers: `field has any ['runtime', 'selector']`,
  `field has all ['slice', 'runtime']`;
- substring/object membership: `field contains 'value'`;
- string prefixes and suffixes: `field starts_with 'docs/'`,
  `field ends_with '.md'`;
- boolean composition with `and`, `or`, `not`, and parentheses;
- optional typed field catalogs before evaluation, including CLI `--catalog`
  JSON files;
- CLI adapters for JSON arrays, JSONL rows, and Markdown tables;
- CLI projection with `--fields` for emitting only selected row fields;
- CLI result shaping with `--sort-by`, `--desc`, `--offset`, `--limit`, and
  `--count`.

```bash
slice eval --expr "metadata.tags has 'context' and metadata.status eq 'ready'" --input examples/pebble.json
```

Set and null operators keep repo/status queries compact:

```bash
slice eval --markdown-table --expr "slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] and tracker is not null" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Boolean grouping supports OData-style row predicates:

```bash
slice eval --markdown-table --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Markdown planning tables can be selected directly and emitted as JSONL rows:

```bash
slice eval --markdown-table --expr "status eq '[~]'" --input ../TRACKER/dependency-systems/slice-usage.md
```

Use `--fields` to project matching rows for scripts and planning reports:

```bash
slice eval --markdown-table --expr "tracker eq '[x]'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker,notes
```

Use `--catalog` to type-check CLI selectors before evaluating rows:

```bash
slice eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker eq '[x]'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Use result-shaping flags for OData-style planning queries:

```bash
slice eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../TRACKER/dependency-systems/slice-usage.md --sort-by slice_layer --limit 2 --fields slice_layer,tracker
```

## Formalism

SLICE is a typed selector pipeline: parse source syntax, normalize it, resolve
paths through a consumer-owned field catalog, type-check predicates, plan any
consumer-owned requirements, evaluate over adapter-provided values, and explain
the result. The detailed model is in
[`docs/specs/formalism.md`](docs/specs/formalism.md).

The first adoption path is documented in
[`docs/plans/consumer-migration.md`](docs/plans/consumer-migration.md).

## Mock client

`slice-mock-client` is the local downstream validation harness. It runs the
current selector contract over Pebble-shaped metadata, CROP-like evidence units,
FLETCH-like active partitions, and ICELINES-like player rows. For FLETCH, SLICE
selects rows and the mock client performs the downstream fold into quiver
candidates, preserving the layer boundary.

The mock client also includes a CROP frontmatter-query parity adapter. It derives
a field catalog from the query, materializes array-like frontmatter strings for
`has`, and preserves CROP's current behavior that a missing field satisfies
`ne`.

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
let requirements = selector.requirements();
```

`explain` is machine-readable, so downstream CLIs and agents can show which
fields, operators, and typed literals a selector depends on.
`requirements` is the deduplicated field list an adapter must materialize before
evaluation.

Errors also expose `diagnostic()` as `slice.diagnostic.v1`, including the error
kind, message, byte offset, and catalog/type details when available.

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
cargo run -p slice-cli -- eval --markdown-table --expr "status eq '[~]'" --input ../TRACKER/dependency-systems/slice-usage.md
cargo run -p slice-cli -- eval --markdown-table --expr "tracker eq '[x]'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --expr "slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] and tracker is not null" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker eq '[x]'" --input ../TRACKER/dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../TRACKER/dependency-systems/slice-usage.md --sort-by slice_layer --limit 2 --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../TRACKER/dependency-systems/slice-usage.md --count
cargo run -p slice-mock-client
```

## License

[MIT](LICENSE) - © 2026 Gio Della-Libera.
