param(
  [string]$RrLangRepo = '..',
  [string]$DatasetRoot = '.',
  [int]$Nulls = 100,
  [string]$Glob = '*.txt',
  [string]$OutDir = '.\outputs\corpus_batch',
  [string]$HyphenPolicy = 'punctuation'
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path $DatasetRoot
$repo = Resolve-Path $RrLangRepo
New-Item -Force -ItemType Directory $OutDir | Out-Null

$files = Get-ChildItem -Path (Join-Path $root 'datasets') -Recurse -Filter $Glob | Where-Object { $_.FullName -notmatch '\\_cache\\' -and $_.Length -gt 0 }
Write-Host "Found $($files.Count) text files" -ForegroundColor Cyan

Push-Location $repo
try {
  foreach ($f in $files) {
    $rel = $f.FullName.Substring($root.Path.Length).TrimStart('\','/')
    $safe = ($rel -replace '[:\\/ ]+', '_' -replace '[^A-Za-z0-9_.-]', '_')
    $jsonOut = Join-Path (Resolve-Path $OutDir) ($safe + '.json')
    $txtOut = Join-Path (Resolve-Path $OutDir) ($safe + '.txt')
    Write-Host "ANALYSE $rel" -ForegroundColor Green
    cargo run -p rrlang -- analyse `
      --input $f.FullName `
      --language Corpus `
      --hyphen-policy $HyphenPolicy `
      --nulls $Nulls `
      --out $jsonOut `
      --text-out $txtOut
  }
} finally {
  Pop-Location
}
