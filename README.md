# SLICE

SLICE is a low-layer Rust query and expression language for selecting typed
artifacts across the portfolio. It gives FLETCH, MDCROP, MDPORT, PROOF, and
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
- membership: `field in ['MDCROP', 'PROOF']`, `field not in ['blocked']`;
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
- machine-readable parse and typed explain reports with expression trees;
- SQLite and OData fold plans with per-source predicates, parameters, requirements,
  residual filters, and diagnostics;
- CLI adapters for JSON arrays, JSONL rows, and Markdown tables;
- CLI projection with `--fields` for emitting only selected row fields;
- CLI result shaping with `--sort-by`, `--desc`, `--offset`, `--limit`, and
  `--count`.

```bash
slice eval --expr "metadata.tags has 'context' and metadata.status eq 'ready'" --input examples/mdport.json
```

Set and null operators keep repo/status queries compact:

```bash
slice eval --markdown-table --expr "slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] and tracker is not null" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Boolean grouping supports OData-style row predicates:

```bash
slice eval --markdown-table --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Markdown planning tables can be selected directly and emitted as JSONL rows:

```bash
slice eval --markdown-table --expr "status eq '[~]'" --input ../../../dependency-systems/slice-usage.md
```

Use `--fields` to project matching rows for scripts and planning reports:

```bash
slice eval --markdown-table --expr "tracker eq '[x]'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker,notes
```

Use `--catalog` to type-check CLI selectors before evaluating rows:

```bash
slice eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker eq '[x]'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
```

Use result-shaping flags for OData-style planning queries:

```bash
slice eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../../../dependency-systems/slice-usage.md --sort-by slice_layer --limit 2 --fields slice_layer,tracker
```

Use `slice explain` to inspect parse trees, typed fields, requirements, and
diagnostics without scanning input rows:

```bash
slice explain --catalog examples/tracker-slice-usage-catalog.json --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'"
```

Use `slice plan` to inspect which predicates can be pushed into SQLite or OData
before a consumer executes a query. Multi-source `and` branches fold
independently so a consumer can attach each predicate to its own side of a join:

```bash
slice plan --backend sqlite --catalog examples/icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'"
```

```bash
slice plan --backend odata --catalog examples/icelines-odata-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'"
```

SQLite-backed CLIs can also use the optional SQLite workbench commands. These
commands stay outside `slice-core`: they inspect real database files, emit draft
fold catalogs, validate consumer-supplied mappings, and run read-only per-source
smoke queries for folded predicates.

```bash
slice sqlite inspect --db path\to\icelines.sqlite
slice sqlite plan --db path\to\icelines.sqlite --catalog examples/icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8"
```

## Formalism

SLICE is a typed selector pipeline: parse source syntax, normalize it, resolve
paths through a consumer-owned field catalog, type-check predicates, plan any
consumer-owned requirements, fold supported predicates into backend plans,
evaluate residuals over adapter-provided values, and explain the result. The
detailed model is in
[`docs/specs/formalism.md`](docs/specs/formalism.md).

The first adoption path is documented in
[`docs/plans/consumer-migration.md`](docs/plans/consumer-migration.md).

The protected pre-1.0 contracts, versioning rules, and mandatory FLETCH
downstream rehearsal are defined in
[`docs/compatibility.md`](docs/compatibility.md).

Performance wording is also gated. The
[`performance claim boundary`](docs/performance-claim-boundary.md) makes
`SLICE-PF-06` explicit: SLICE must not be described as fast enough for large
artifact scans, FLETCH partitions, SQLite tables, or hot consumer paths unless
the claim names fixture size, row width, command, selector, catalog posture,
memory posture, downstream responsibilities, and review dispositions. The
current guard is a 1,000-row JSONL smoke check, not a benchmark.

## Mock client

`slice-mock-client` is the local downstream validation harness. It runs the
current selector contract over Mdport-shaped metadata, MDCROP-like evidence units,
FLETCH-like active partitions, and ICELINES-like player rows. For FLETCH, SLICE
selects rows and the mock client performs the downstream fold into quiver
candidates, preserving the layer boundary.

The mock client also includes a MDCROP frontmatter-query parity adapter. It derives
a field catalog from the query, materializes array-like frontmatter strings for
`has`, and preserves MDCROP's current behavior that a missing field satisfies
`ne`.

For ICELINES-style storage, the mock client creates an in-memory SQLite database,
folds player predicates to the `players` source, folds stat predicates to the
`stats` source, runs the consumer-owned join, and then applies the residual
SLICE filter locally.

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
let _parse_explain = slice_core::parse("metadata.status eq 'ready'")?.explain_parse();

let mut fold_catalog = slice_core::FoldCatalog::new();
fold_catalog
    .insert_sqlite("player.position", slice_core::ValueType::String, "players", "players.position")
    .insert_sqlite("stats.ppg", slice_core::ValueType::Number, "stats", "stats.ppg");
let fold_plan = slice_core::parse("player.position eq 'C' and stats.ppg ge 0.8")?
    .plan_sqlite(&fold_catalog)?;
```

`explain` is machine-readable, so downstream CLIs and agents can show which
fields, operators, typed literals, and boolean tree a selector depends on.
`requirements` is the deduplicated field list an adapter must materialize before
evaluation.

Errors also expose `diagnostic()` as `slice.diagnostic.v1`, including the error
kind, message, byte offset, and catalog/type details when available.

## Non-goals

- SLICE does not fetch data, build corpora, or cache artifacts.
- SLICE does not own MDCROP graph cuts, MDPORT schema design, FLETCH manifests, or
  product-specific query surfaces.
- `slice-core` does not execute SQL or infer joins; the optional SQLite CLI layer
  can run read-only inspection and per-source smoke queries, while consumers own
  schemas, joins, execution, auth, and ranking.
- SLICE is not a general programming language; it is a portable selector and
  expression kernel.

## Validation

```bash
cargo fmt --check
cargo test
cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/mdport.json
cargo run -p slice-cli -- eval --markdown-table --expr "status eq '[~]'" --input ../../../dependency-systems/slice-usage.md
cargo run -p slice-cli -- eval --markdown-table --expr "tracker eq '[x]'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --expr "slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] and tracker is not null" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker eq '[x]'" --input ../../../dependency-systems/slice-usage.md --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../../../dependency-systems/slice-usage.md --sort-by slice_layer --limit 2 --fields slice_layer,tracker
cargo run -p slice-cli -- eval --markdown-table --catalog examples/tracker-slice-usage-catalog.json --expr "tracker is not null" --input ../../../dependency-systems/slice-usage.md --count
cargo run -p slice-cli -- explain --catalog examples/tracker-slice-usage-catalog.json --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'"
cargo run -p slice-cli -- plan --backend sqlite --catalog examples/icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'"
cargo run -p slice-cli -- plan --backend odata --catalog examples/icelines-odata-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'"
cargo run -p slice-cli -- sqlite inspect --db path\to\icelines.sqlite
cargo run -p slice-cli -- sqlite plan --db path\to\icelines.sqlite --catalog examples/icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8"
cargo run -p slice-mock-client
```

## License

[MIT](LICENSE) - © 2026 Gio Della-Libera.
