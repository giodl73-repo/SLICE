# SLICE Formalism

SLICE is not "ICELINES query, but generic." It is the lower reusable shape that
ICELINES-style query engines keep rebuilding: a typed selector IR, a sequence of
lowering and validation passes, and an evaluation context that lets product
repos keep their own domain semantics.

## Why ICELINES matters

ICELINES query is already a real example of the pattern:

1. The user-facing surface is domain-first: `icelines query leaders`, `player`,
   `compare`, `career`, and windowed leaders.
2. Flags and filters become a typed query plan with boolean composition and typed
   atoms.
3. The plan computes requirements before execution, such as which reports,
   boxscores, date windows, or career records are needed.
4. A provider/context boundary supplies data without letting the lower query
   crate own fetching.
5. The executor evaluates the typed plan against domain views and preserves
   product-specific missing-data and strict-mode policy.

SLICE should make that pipeline easier to reuse. It should not make users type
raw SLICE for every ICELINES query.

## The core model

SLICE owns four product-neutral concepts.

| Concept | Meaning | Product-owned counterpart |
|---|---|---|
| `Expression` | Boolean tree of selector clauses. | ICELINES `QueryPlan`, CROP view predicate, FLETCH manifest slice. |
| `Path` | Stable field path into an adapter-provided value. | `metadata.tags`, `player.bio.age`, `manifest.entries[].verified`. |
| `Predicate` | Typed operator plus typed literal or set. | Numeric threshold, string equality, membership, range, pattern. |
| `EvalContext` | Read-only evaluation settings and provider hooks. | ICELINES season/today/provider, CROP view root, FLETCH manifest scope. |

The invariant: `slice-core` can parse, type-check, explain, and evaluate these
concepts without knowing hockey, graph cuts, Pebble schema policy, cache
freshness, or Markdown rendering.

## Pass pipeline

SLICE should evolve toward an explicit pass pipeline:

1. **Parse**: source text or structured fragments become raw syntax.
2. **Normalize**: aliases, case policy, and punctuation variants become a stable
   internal form.
3. **Resolve**: consumer adapters map paths to known fields and value types.
4. **Type-check**: operator/value compatibility is checked before execution.
5. **Plan**: consumers can collect requirements, cost hints, and strict-mode
   blockers from the resolved expression.
6. **Evaluate**: a compiled selector is reused across many artifact rows.
7. **Explain**: the same resolved form can render a machine-readable reason,
   requirements summary, and diagnostics.

Only the generic pass machinery belongs in SLICE. Consumer repos own path
catalogs, aliases, requirements, and policy.

## Typed IR target

The target IR should be shape-by-construction:

```text
Expression
  = All(Vec<Expression>)
  | Any(Vec<Expression>)
  | Not(Box<Expression>)
  | Clause {
      path: ResolvedPath,
      predicate: Predicate,
    }

Predicate
  = Compare(CompareOp, Literal)
  | Member(MemberOp, Vec<Literal>)
  | Range { min: Bound, max: Bound }
  | Pattern(PatternOp, Pattern)
  | Exists
```

Invalid combinations should fail before evaluation. Examples:

- `metadata.tags has 'context'` is valid when `metadata.tags` is an array,
  object, or string-like field.
- `player.age contains 'young'` should fail type-check because age is numeric.
- `manifest.verified between true and false` should fail type-check because
  booleans do not have an ordered range.

## ICELINES mapping

ICELINES should keep its user-facing commands and domain aliases. SLICE helps by
providing a reusable lower layer under pieces of the query engine.

| ICELINES surface | Stays in ICELINES | Potential SLICE responsibility |
|---|---|---|
| `--pos C`, `--age-max 23`, `--nationality SWE` | Hockey-friendly flags and aliases. | Lowered clauses over resolved bio fields. |
| `--sort pts-pace`, `--percentiles`, `--top 20` | Ranking, display, metric catalog, pagination. | Possibly parse generic order/limit later, but not first. |
| `--situation 5v5`, `--score-state close` | Hockey-specific axes and data availability. | Generic typed enum/string predicates after ICELINES resolves them. |
| `--filter "g.last10g>=5"` | Hockey stat IDs, window semantics, current-team/career policy. | Boolean tree, predicate shapes, diagnostics, explain framework. |
| `QueryPlan::requirements()` | ICELINES knows reports, boxscores, career records, strict policy. | Generic pass hook for collecting requirements from resolved clauses. |
| `DataProvider` / `EvalCtx` | ICELINES owns data loading and time/season context. | Generic evaluation context pattern, no fetching. |

Concrete example:

```bash
icelines query leaders --pos C --age-max 23 --nationality SWE --ppg-min 0.80 --sort pts-pace
```

ICELINES should still parse that as an ICELINES command. Internally it can lower
the filter portion into a SLICE-shaped expression:

```text
All(
  Clause(player.position, Compare(eq, "C")),
  Clause(player.age, Compare(le, 23)),
  Clause(player.nationality, Compare(eq, "SWE")),
  Clause(stats.ppg, Compare(ge, 0.80)),
)
```

ICELINES still owns `pts-pace`, the stat catalog, player views, percentiles,
formatting, and missing-data policy.

## What gets easier

SLICE makes repeated query work easier by giving repos a common set of building
blocks:

- one parser/evaluator for simple metadata predicates;
- one typed predicate model for equality, membership, ranges, and patterns;
- one diagnostics shape for parse/type/evaluation failures;
- one explain shape agents can read;
- one compiled selector that can run over JSON, Pebble sections, CROP units,
  FLETCH manifests, and ICELINES adapter rows;
- one place to benchmark selector performance.

For ICELINES specifically, SLICE can eventually reduce duplicated parsing and
diagnostic work while leaving the advanced hockey logic in `icelines-query`.

## Near-term adoption rule

Do not start by moving ICELINES wholesale onto SLICE. Start with the smallest
shared subset:

1. Define a path catalog API in SLICE.
2. Add range and set predicates to `slice-core`.
3. Prove CROP/Pebble/FLETCH fixtures.
4. Add an ICELINES adapter experiment for simple bio/stat filters only.
5. Keep windows, career aggregation, similarity search, sorting, and data
   requirements in ICELINES until the shared abstraction earns them.

This keeps SLICE low-layer while still making the hard-won ICELINES query
architecture reusable.
