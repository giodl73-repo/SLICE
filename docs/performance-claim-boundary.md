# Performance claim boundary

SLICE's parser/evaluator contracts and fixture parity are separate from any
large-scan or production performance claim. Name corpus size, row width,
expression/catalog, environment, memory posture and consumer responsibilities
before making such a claim. Native scans, SQLite execution and product ranking
remain outside the browser adapter.

The playground limits JSON to 1 MiB, 2,000 object rows, 24 levels and 50,000 nodes;
selectors to 2 KiB, 16 nesting levels, 128 words and 16 unquoted `not` tokens;
catalogs to 16 KiB and 128 fields. These are conservative admission budgets,
not a new grammar or a throughput guarantee. Startup and request deadlines
recover the UI; they are not interruptible native computation quotas.

Record browser readiness/request observations with exact artifact, browser,
fixture and environment. Do not interpret end-to-end worker/CDN timings as
pure Rust CPU benchmarks, an SLA, or evidence of native corpus-scale adoption.
