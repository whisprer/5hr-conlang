param(
    [string]$RrLangRepo = "..",
    [int]$Nulls = 100,
    [string]$Encodings = "utf8_bits,grapheme,grapheme_class,word_boundary,frequency_class"
)

$ErrorActionPreference = "Stop"
$packRoot = Split-Path -Parent $PSScriptRoot
$outDir = Join-Path $packRoot "outputs\udhr_batch"
New-Item -Force -ItemType Directory $outDir | Out-Null

function Resolve-RrLangRepo($candidate) {
    if ($candidate -and (Test-Path (Join-Path $candidate "Cargo.toml"))) {
        return (Resolve-Path $candidate).Path
    }
    # Common case: document pack is inside rrlang/testdata and the Cargo workspace is parent.
    $p = Resolve-Path $packRoot
    for ($i=0; $i -lt 5; $i++) {
        if (Test-Path (Join-Path $p "Cargo.toml")) { return $p.Path }
        $parent = Split-Path -Parent $p.Path
        if (!$parent -or $parent -eq $p.Path) { break }
        $p = Resolve-Path $parent
    }
    throw "Could not find RRLANG Cargo workspace. Pass -RrLangRepo with the folder containing Cargo.toml. You gave: $candidate"
}

$rr = Resolve-RrLangRepo $RrLangRepo
Write-Host "RRLANG repo: $rr"

$files = Get-ChildItem -Recurse -File (Join-Path $packRoot "datasets\parallel\udhr") -Filter "udhr_*_text_only.txt"
if (!$files -or $files.Count -eq 0) {
    throw "No UDHR text files found. Run scripts\Fetch-RRLangDocuments.ps1 first."
}

foreach ($f in $files) {
    $code = Split-Path -Leaf (Split-Path -Parent $f.FullName)
    $base = "udhr_$code"
    $jsonOut = Join-Path $outDir ("$base.json")
    $txtOut = Join-Path $outDir ("$base.txt")
    $hyphenPolicy = if ($code -eq "mesu") { "morpheme_boundary" } else { "punctuation" }
    Write-Host "RUN   $code $($f.FullName)"
    Push-Location $rr
    try {
        cargo run -p rrlang -- analyse `
            --input $f.FullName `
            --language $code `
            --encodings $Encodings `
            --hyphen-policy $hyphenPolicy `
            --nulls $Nulls `
            --out $jsonOut `
            --text-out $txtOut
    } finally {
        Pop-Location
    }
}

Write-Host "DONE. Reports written to $outDir"
