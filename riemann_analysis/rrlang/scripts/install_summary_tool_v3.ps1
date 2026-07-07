# install_summary_tool_v3.ps1
# Run this from the extracted patch folder.
# It copies the real summariser into your RRLANG repo scripts folder.

param(
  [string]$RepoRoot = "D:\code\5hr-conlang\riemann_analysis\rrlang"
)

$ErrorActionPreference = "Stop"

$Source = Join-Path $PSScriptRoot "summarise_rrlang_reports_v3.py"
$DestDir = Join-Path $RepoRoot "scripts"
$Dest = Join-Path $DestDir "summarise_rrlang_reports_v3.py"

if (!(Test-Path $Source)) {
  throw "Could not find source summariser: $Source"
}

New-Item -Force -ItemType Directory $DestDir | Out-Null
Copy-Item -Force $Source $Dest

Write-Host "Installed real summariser:" -ForegroundColor Green
Write-Host "  $Dest"
Write-Host ""
Write-Host "Now run:" -ForegroundColor Cyan
Write-Host "  cd $RepoRoot"
Write-Host "  py -3 .\scripts\summarise_rrlang_reports_v3.py --input .\outputs\corpus_batch_curated --out .\outputs\summary_curated"
