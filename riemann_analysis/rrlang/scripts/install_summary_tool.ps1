# install_summary_tool.ps1
    # Run from the rrlang repo root:
    #   powershell -ExecutionPolicy Bypass -File .\install_summary_tool.ps1

    $ErrorActionPreference = "Stop"

    $ScriptDir = Join-Path (Get-Location) "scripts"
    New-Item -Force -ItemType Directory $ScriptDir | Out-Null

    $Source = Join-Path $PSScriptRoot "summarise_rrlang_reports_v2.py"
    $Dest = Join-Path $ScriptDir "summarise_rrlang_reports_v2.py"

    Copy-Item -Force $Source $Dest

    Write-Host "Installed:" -ForegroundColor Green
    Write-Host "  $Dest"
    Write-Host ""
    Write-Host "Test command:" -ForegroundColor Cyan
    Write-Host "  py -3 .\scripts\summarise_rrlang_reports_v2.py --input .\outputs\corpus_batch_curated --out .\outputs\summary_curated"
    