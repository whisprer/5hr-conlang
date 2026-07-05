param(
    [string]$Only = "en,fr,de,es,ja,zh,cy,ru,ar,he,tr,sw,xh,is,la,el",
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$packRoot = Split-Path -Parent $PSScriptRoot
$cacheDir = Join-Path $packRoot "_cache"
$zipPath = Join-Path $cacheDir "udhr2.zip"
$extractDir = Join-Path $cacheDir "udhr2_extracted"
$outRoot = Join-Path $packRoot "datasets\parallel\udhr"
$logDir = Join-Path $packRoot "logs"
New-Item -Force -ItemType Directory $cacheDir, $outRoot, $logDir | Out-Null

$udhr2Url = "https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/udhr2.zip"
$udhrFallbackUrl = "https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/udhr.zip"

function Download-File($url, $dest) {
    Write-Host "FETCH $url"
    try {
        Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing -TimeoutSec 120
        return $true
    } catch {
        Write-Warning "Download failed: $url"
        Write-Warning $_.Exception.Message
        return $false
    }
}

if ($Force -or !(Test-Path $zipPath)) {
    if (!(Download-File $udhr2Url $zipPath)) {
        Write-Warning "Trying fallback NLTK udhr.zip"
        $zipPath = Join-Path $cacheDir "udhr.zip"
        if (!(Download-File $udhrFallbackUrl $zipPath)) {
            throw "Could not download UDHR corpus package from NLTK GitHub."
        }
    }
}

if ($Force -or !(Test-Path $extractDir)) {
    if (Test-Path $extractDir) { Remove-Item -Recurse -Force $extractDir }
    New-Item -Force -ItemType Directory $extractDir | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $extractDir)
}

# Language search patterns are deliberately broad because NLTK UDHR package filenames differ between udhr.zip and udhr2.zip.
$targets = @{
    "en" = @{ name="English";  patterns=@("English", "eng", "ENGLISH") }
    "fr" = @{ name="French";   patterns=@("French", "Francais", "Français", "fra", "fre") }
    "es" = @{ name="Spanish";  patterns=@("Spanish", "Espanol", "Español", "spa") }
    "de" = @{ name="German";   patterns=@("German", "Deutsch", "deu", "ger") }
    "zh" = @{ name="Chinese";  patterns=@("Chinese", "Mandarin", "Zhong", "Hanzi", "cmn", "zho", "Chinese_Mandarin") }
    "ja" = @{ name="Japanese"; patterns=@("Japanese", "Nihongo", "jpn", "Japanese_Nihongo") }
    "cy" = @{ name="Welsh";    patterns=@("Welsh", "Cymraeg", "cym", "wel") }
    "ru" = @{ name="Russian";  patterns=@("Russian", "Russky", "Russki", "rus") }
    "ar" = @{ name="Arabic";   patterns=@("Arabic", "Arab", "ara", "arb") }
    "he" = @{ name="Hebrew";   patterns=@("Hebrew", "Ivrit", "heb") }
    "tr" = @{ name="Turkish";  patterns=@("Turkish", "Turkce", "Türkçe", "tur") }
    "sw" = @{ name="Swahili";  patterns=@("Swahili", "Kiswahili", "swa", "swh") }
    "xh" = @{ name="Xhosa";    patterns=@("Xhosa", "xho") }
    "is" = @{ name="Icelandic";patterns=@("Icelandic", "Islenska", "Íslenska", "ice", "isl") }
    "la" = @{ name="Latin";    patterns=@("Latin", "Latina", "lat") }
    "el" = @{ name="Greek";    patterns=@("Greek", "Ellinika", "Hellenic", "ell", "gre") }
}

function Get-CandidateFiles {
    Get-ChildItem -Recurse -File $extractDir |
        Where-Object { $_.Name -notmatch 'README|LICENSE|\.DS_Store' -and $_.Length -gt 200 }
}

function Score-File($file, $patterns) {
    $name = $file.Name
    $path = $file.FullName
    $score = 0
    foreach ($p in $patterns) {
        if ($name -match $p) { $score += 100 }
        elseif ($path -match $p) { $score += 50 }
    }
    if ($name -match 'UTF|utf|Unicode|unicode') { $score += 10 }
    if ($name -match 'err|font|~') { $score -= 1000 }
    return $score
}

function Read-TextBestEffort($path) {
    # UDHR2 should be UTF-8, but fallback udhr.zip has mixed legacy encodings. Keep this simple and robust enough for first-pass corpus work.
    try { return [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8) } catch {}
    try { return [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::Default) } catch {}
    return (Get-Content -Raw $path)
}

function Clean-UdhrText($text) {
    $text = $text -replace "`r`n", "`n"
    $text = $text -replace "`r", "`n"
    # Many UDHR-in-Unicode files have a short metadata header followed by ---.
    $marker = "---"
    $idx = $text.IndexOf($marker)
    if ($idx -ge 0 -and $idx -lt 1000) {
        $text = $text.Substring($idx + $marker.Length)
    }
    $lines = $text -split "`n"
    $kept = New-Object System.Collections.Generic.List[string]
    foreach ($line in $lines) {
        $l = $line.TrimEnd()
        if ($l -match '^\s*$') { $kept.Add(""); continue }
        if ($l -match '^\s*(©|This plain text version prepared|Universal Declaration of Human Rights -)') { continue }
        $kept.Add($l)
    }
    $joined = ($kept -join "`n").Trim() + "`n"
    return $joined
}

$wanted = $Only.Split(',') | ForEach-Object { $_.Trim().ToLowerInvariant() } | Where-Object { $_ }
$allCandidates = @(Get-CandidateFiles)
Write-Host "UDHR package files found: $($allCandidates.Count)"

foreach ($code in $wanted) {
    if (!$targets.ContainsKey($code)) {
        Write-Warning "Unknown language code in -Only: $code"
        continue
    }
    $t = $targets[$code]
    $scored = foreach ($f in $allCandidates) {
        $s = Score-File $f $t.patterns
        if ($s -gt 0) { [PSCustomObject]@{ Score=$s; File=$f } }
    }
    $best = $scored | Sort-Object Score, @{Expression={$_.File.Length}; Descending=$true} -Descending | Select-Object -First 1
    if (!$best) {
        Write-Host "MISS  $code $($t.name)"
        continue
    }

    $langDir = Join-Path $outRoot $code
    New-Item -Force -ItemType Directory $langDir | Out-Null
    $rawOut = Join-Path $langDir ("udhr_{0}_raw.txt" -f $code)
    $cleanOut = Join-Path $langDir ("udhr_{0}_text_only.txt" -f $code)
    $sourceOut = Join-Path $langDir "source.json"

    $text = Read-TextBestEffort $best.File.FullName
    [System.IO.File]::WriteAllText($rawOut, $text, [System.Text.Encoding]::UTF8)
    $clean = Clean-UdhrText $text
    [System.IO.File]::WriteAllText($cleanOut, $clean, [System.Text.Encoding]::UTF8)

    $src = [PSCustomObject]@{
        language_code = $code
        language = $t.name
        source_package = "NLTK udhr2/udhr corpus archive"
        source_url = $udhr2Url
        fallback_url = $udhrFallbackUrl
        extracted_file = $best.File.FullName
        score = $best.Score
        cleaned_output = $cleanOut
        note = "Generated by RRLANG document-pack Fetch-RRLangDocuments.ps1"
    } | ConvertTo-Json -Depth 4
    [System.IO.File]::WriteAllText($sourceOut, $src, [System.Text.Encoding]::UTF8)
    Write-Host "OK    $code $($t.name) <- $($best.File.Name)"
}

# Controls remain useful and cheap.
$controls = Join-Path $packRoot "datasets\controls"
New-Item -Force -ItemType Directory $controls | Out-Null
$rng = New-Object System.Random 18427
$bits = -join (1..10000 | ForEach-Object { if ($rng.NextDouble() -lt 0.5) { '0' } else { '1' } })
[System.IO.File]::WriteAllText((Join-Path $controls "random_bits_10k.txt"), $bits + "`n", [System.Text.Encoding]::UTF8)
Write-Host "controls written to $controls"
