$ErrorActionPreference = 'Stop'

function Assert-Contains {
  param(
    [string]$Path,
    [string]$Needle
  )

  $text = Get-Content -Raw -LiteralPath $Path
  if ($text.IndexOf($Needle, [StringComparison]::Ordinal) -lt 0) {
    throw "Missing expected text in ${Path}: ${Needle}"
  }
}

Assert-Contains 'README.md' 'performance claim boundary'
Assert-Contains 'README.md' 'SLICE-PF-06'
Assert-Contains 'README.md' '1,000-row JSONL smoke'

Assert-Contains 'PRODUCT_PLAN.md' 'performance claim boundary'
Assert-Contains 'PRODUCT_PLAN.md' 'hot-path replacement'

Assert-Contains 'docs/compatibility.md' 'performance claim boundary'
Assert-Contains 'docs/compatibility.md' 'SLICE-PF-06'

Assert-Contains 'docs/performance-claim-boundary.md' '`SLICE-PF-06`'
Assert-Contains 'docs/performance-claim-boundary.md' 'fixture size and row width'
Assert-Contains 'docs/performance-claim-boundary.md' 'If any field is missing'
Assert-Contains 'docs/performance-claim-boundary.md' 'Performance Engineer, Adapter Boundary Keeper, Contract Checker, and'

Assert-Contains 'docs/reviews/formalism-role-review.md' 'performance claim boundary'
Assert-Contains 'docs/reviews/slice-plan-role-review.md' 'performance claim boundary'
Assert-Contains 'docs/plans/consumer-migration.md' 'performance claim boundary'

Assert-Contains '.roles/ROLE.md' '## PITFALL gates'
Assert-Contains '.roles/ROLE.md' '`SLICE-PF-06`'
Assert-Contains '.roles/ROLE.md' 'Performance Engineer; Adapter Boundary Keeper; Contract Checker; Validation Checker'

Assert-Contains '.pitfall/slice-pitfalls.md' '**Status:** MITIGATED'
Assert-Contains '.pitfall/slice-pitfalls.md' 'tests/check-performance-claim-boundary.ps1'
Assert-Contains '.pitfall/slice-invariants.md' 'SLICE-I-06'
Assert-Contains '.pitfall/slice-invariants.md' 'Performance Claims Require Sized Evidence'
Assert-Contains '.pitfall/slice-invariants.md' 'SLICE-PF-06'

$tmp = Join-Path $env:TEMP 'slice-performance-claim-boundary.jsonl'
try {
  1..1000 | ForEach-Object {
    if ($_ % 4 -eq 0) {
      '{"metadata":{"status":"ready"},"metrics":{"score":' + $_ + '},"partition":{"name":"p' + ($_ % 10) + '"}}'
    } else {
      '{"metadata":{"status":"draft"},"metrics":{"score":' + $_ + '},"partition":{"name":"p' + ($_ % 10) + '"}}'
    }
  } | Set-Content -Encoding utf8 -LiteralPath $tmp

  $cargo = if ($env:CARGO) { $env:CARGO } else { 'C:\Users\giodl\.cargo\bin\cargo.exe' }
  $output = & $cargo run -q -p slice-cli -- eval --jsonl --expr "metadata.status eq 'ready' and metrics.score ge 500" --input $tmp --count
  if ($LASTEXITCODE -ne 0) {
    throw "slice eval JSONL smoke failed with exit code ${LASTEXITCODE}"
  }
  $countLine = $output | Where-Object { $_ -eq '{"count":126}' } | Select-Object -First 1
  if ($countLine -ne '{"count":126}') {
    throw "Unexpected JSONL smoke count: ${output}"
  }
}
finally {
  if (Test-Path -LiteralPath $tmp) {
    Remove-Item -LiteralPath $tmp
  }
}

Write-Host 'SLICE performance claim boundary check passed.'
