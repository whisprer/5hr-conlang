# apply_rrlang_v0_3_operability_patch.ps1
# Run this from the root of your existing rrlang repo:
#   cd D:\code\5hr-conlang\riemann_analysis\rrlang
#   powershell -ExecutionPolicy Bypass -File C:\path\to\patch\apply_rrlang_v0_3_operability_patch.ps1

$ErrorActionPreference = "Stop"

$Repo = Get-Location
$PatchRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$ReplacementRoot = Join-Path $PatchRoot "replacement_files"

if (-not (Test-Path (Join-Path $Repo "Cargo.toml"))) {
  throw "Run this from the rrlang repo root. Cargo.toml was not found in $Repo"
}

$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$BackupRoot = Join-Path $Repo "backup_rrlang_v0_2_before_v0_3_$Stamp"
New-Item -Force -ItemType Directory $BackupRoot | Out-Null

$Files = @(
  "Cargo.toml",
  "README.md",
  "crates\rrlang-core\src\types.rs",
  "crates\rrlang-core\src\config.rs",
  "crates\rrlang-core\src\pipeline.rs",
  "crates\rrlang-core\src\report.rs",
  "crates\rrlang-cli\src\main.rs"
)

foreach ($Rel in $Files) {
  $Src = Join-Path $ReplacementRoot $Rel
  $Dst = Join-Path $Repo $Rel
  $Bak = Join-Path $BackupRoot $Rel

  if (-not (Test-Path $Src)) {
    throw "Missing replacement file: $Src"
  }

  if (Test-Path $Dst) {
    New-Item -Force -ItemType Directory (Split-Path $Bak) | Out-Null
    Copy-Item -Force $Dst $Bak
  }

  New-Item -Force -ItemType Directory (Split-Path $Dst) | Out-Null
  Copy-Item -Force $Src $Dst
  Write-Host "patched $Rel" -ForegroundColor Green
}

Write-Host ""
Write-Host "Backup saved to:" -ForegroundColor Cyan
Write-Host "  $BackupRoot"
Write-Host ""
Write-Host "Now run:" -ForegroundColor Cyan
Write-Host "  cargo build --release -p rrlang"
Write-Host "  .\target\release\rrlang.exe --help"
