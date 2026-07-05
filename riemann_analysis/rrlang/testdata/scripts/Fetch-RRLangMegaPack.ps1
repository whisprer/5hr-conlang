param(
  [ValidateSet('Core','Broad','UDHR','Wiki','Tatoeba','Flores','Controls','Gutenberg')]
  [string]$Preset = 'Core',
  [string]$Languages = '',
  [int]$WikiPagesPerLang = 20,
  [int]$TatoebaPerLang = 10000,
  [switch]$Big,
  [switch]$Huge,
  [switch]$SkipFlores,
  [switch]$SkipTatoeba,
  [switch]$SkipWiki,
  [switch]$SkipUdhr,
  [switch]$SkipControls
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Resolve-Path (Join-Path $here '..')
$py = Get-Command py -ErrorAction SilentlyContinue
if ($py) {
  $python = 'py'
  $pyArgs = @('-3')
} else {
  $python = 'python'
  $pyArgs = @()
}

if ($Big) {
  if ($WikiPagesPerLang -lt 100) { $WikiPagesPerLang = 100 }
  if ($TatoebaPerLang -lt 50000) { $TatoebaPerLang = 50000 }
}
if ($Huge) {
  if ($WikiPagesPerLang -lt 300) { $WikiPagesPerLang = 300 }
  if ($TatoebaPerLang -lt 150000) { $TatoebaPerLang = 150000 }
}

$args = @()
$args += $pyArgs
$args += @((Join-Path $here 'fetch_rrlang_mega_pack.py'))
$args += @('--root', $root.Path)
$args += @('--preset', $Preset)
$args += @('--wiki-pages-per-lang', $WikiPagesPerLang)
$args += @('--tatoeba-per-lang', $TatoebaPerLang)
if ($Languages.Trim().Length -gt 0) { $args += @('--langs', $Languages) }
if ($SkipFlores) { $args += '--skip-flores' }
if ($SkipTatoeba) { $args += '--skip-tatoeba' }
if ($SkipWiki) { $args += '--skip-wiki' }
if ($SkipUdhr) { $args += '--skip-udhr' }
if ($SkipControls) { $args += '--skip-controls' }

Write-Host "RRLANG mega fetch root: $($root.Path)" -ForegroundColor Cyan
Write-Host "$python $($args -join ' ')" -ForegroundColor DarkGray
& $python @args
