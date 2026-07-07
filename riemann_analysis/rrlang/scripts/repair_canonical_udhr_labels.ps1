# repair_canonical_udhr_labels.ps1
# Repairs language_guess/corpus_family labels in a canonical UDHR summary folder.
# Run from the RRLANG repo root, for example:
#   powershell -ExecutionPolicy Bypass -File .\scripts\repair_canonical_udhr_labels.ps1 `
#     -InputSummary .\outputs\summary_canonical_udhr_v0_3_nulls100 `
#     -OutputSummary .\outputs\summary_canonical_udhr_v0_3_nulls100_labeled

param(
  [string]$InputSummary = ".\outputs\summary_canonical_udhr_v0_3_nulls100",
  [string]$OutputSummary = ".\outputs\summary_canonical_udhr_v0_3_nulls100_labeled"
)

$ErrorActionPreference = "Stop"

$InputSummaryPath = Resolve-Path $InputSummary
New-Item -Force -ItemType Directory $OutputSummary | Out-Null
$OutputSummaryPath = Resolve-Path $OutputSummary

function Get-RRLangCanonicalLanguage($row) {
  $parts = @()

  foreach ($field in @("report_file", "input", "raw_metric_json", "raw_alert")) {
    if ($row.PSObject.Properties.Name -contains $field) {
      $value = [string]$row.$field
      if ($value) { $parts += $value }
    }
  }

  $text = ($parts -join " ").ToLower() -replace '\\','/'
  $name = ""

  if ($row.PSObject.Properties.Name -contains "report_file") {
    $name = [System.IO.Path]::GetFileNameWithoutExtension([string]$row.report_file).ToLower()
  }

  foreach ($candidate in @($name, $text)) {
    if ($candidate -match '(?<lang>mesu|[a-z]{2,3})_udhr_(mesu|[a-z]{2,3})_canonical') {
      return $Matches.lang
    }
    if ($candidate -match '/(?<lang>mesu|[a-z]{2,3})/udhr_(mesu|[a-z]{2,3})_canonical\.txt') {
      return $Matches.lang
    }
    if ($candidate -match '/parallel/udhr/(?<lang>mesu|[a-z]{2,3})/') {
      return $Matches.lang
    }
  }

  return $null
}

function Set-PropertyIfPresent($row, [string]$name, $value) {
  if ($row.PSObject.Properties.Name -contains $name) {
    $row.$name = $value
  }
}

$csvFilesToRepair = @(
  "report_index.csv",
  "metrics_flat.csv",
  "markov2_survivors.csv",
  "alert_events.csv",
  "top_deviations.csv"
)

foreach ($file in $csvFilesToRepair) {
  $inFile = Join-Path $InputSummaryPath $file
  $outFile = Join-Path $OutputSummaryPath $file

  if (-not (Test-Path $inFile)) {
    Write-Host "MISS $file" -ForegroundColor Yellow
    continue
  }

  $rows = @(Import-Csv $inFile)

  foreach ($row in $rows) {
    $lang = Get-RRLangCanonicalLanguage $row
    if ($lang) {
      Set-PropertyIfPresent $row "language_guess" $lang
      Set-PropertyIfPresent $row "corpus_family" "parallel_udhr"
      Set-PropertyIfPresent $row "source" "udhr"
      Set-PropertyIfPresent $row "source_guess" "udhr"
    }
  }

  $rows | Export-Csv $outFile -NoTypeInformation -Encoding UTF8
  Write-Host "WROTE $outFile ($($rows.Count) rows)" -ForegroundColor Green
}

# Rebuild alert_summary.csv from repaired alert_events.csv.
$alertEvents = Join-Path $OutputSummaryPath "alert_events.csv"
$alertSummary = Join-Path $OutputSummaryPath "alert_summary.csv"

if (Test-Path $alertEvents) {
  $alerts = @(Import-Csv $alertEvents)
  $summary = $alerts |
    Group-Object report_group, corpus_family, language_guess, severity, code |
    ForEach-Object {
      $first = $_.Group[0]
      [pscustomobject]@{
        report_group = $first.report_group
        corpus_family = $first.corpus_family
        language_guess = $first.language_guess
        severity = $first.severity
        code = $first.code
        count = $_.Count
      }
    } |
    Sort-Object corpus_family, language_guess, severity, code

  $summary | Export-Csv $alertSummary -NoTypeInformation -Encoding UTF8
  Write-Host "WROTE $alertSummary ($($summary.Count) rows)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done. Repaired summary folder:" -ForegroundColor Cyan
Write-Host "  $OutputSummaryPath"
