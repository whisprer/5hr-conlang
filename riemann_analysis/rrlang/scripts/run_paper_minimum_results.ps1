Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = "C:\github\5hr-conlang\riemann_analysis\rrlang"
Set-Location -Path $RepoRoot

$RrlangExe = Join-Path $RepoRoot "target\release\rrlang.exe"
$Summariser = Join-Path $RepoRoot "scripts\summarise_rrlang_reports_v3.py"

$MesuRegisterRoot = Join-Path $RepoRoot "testdata\datasets_canonical\mesu_register"
$ControlsRoot = Join-Path $RepoRoot "testdata\datasets_canonical\paper_minimum_controls"

$MesuOut = Join-Path $RepoRoot "outputs\mesu_register_v0_3_nulls100"
$MesuSummary = Join-Path $RepoRoot "outputs\summary_mesu_register_v0_3_nulls100"

$ControlsOut = Join-Path $RepoRoot "outputs\controls_paper_minimum_v0_3_nulls100"
$ControlsSummary = Join-Path $RepoRoot "outputs\summary_controls_paper_minimum_v0_3_nulls100"

$PaperOut = Join-Path $RepoRoot "outputs\paper_minimum_results_20260707"

$SearchRoots = @(
    $RepoRoot,
    "C:\github\5hr-conlang",
    "D:\code\5hr-conlang",
    "D:\github\5hr-conlang",
    "C:\code\5hr-conlang"
) | Where-Object { Test-Path -Path $_ } | Select-Object -Unique

function Assert-File {
    param(
        [Parameter(Mandatory=$true)][string]$Path,
        [Parameter(Mandatory=$true)][string]$Label
    )

    if (-not (Test-Path -Path $Path -PathType Leaf)) {
        throw "Missing $Label at: $Path"
    }
}

function Resolve-ProjectFile {
    param(
        [Parameter(Mandatory=$true)][string]$PreferredPath,
        [Parameter(Mandatory=$true)][string]$Label,
        [Parameter(Mandatory=$true)][string[]]$Patterns
    )

    if (Test-Path -Path $PreferredPath -PathType Leaf) {
        return (Resolve-Path -Path $PreferredPath).Path
    }

    foreach ($pattern in $Patterns) {
        $hit = Get-ChildItem -Path $SearchRoots -Recurse -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -like $pattern } |
            Select-Object -First 1

        if ($null -ne $hit) {
            Write-Host "Resolved $Label via search: $($hit.FullName)" -ForegroundColor Yellow
            return $hit.FullName
        }
    }

    throw "Could not find $Label. Tried preferred path: $PreferredPath"
}

function Get-PythonCommand {
    & py -3 --version *> $null
    if ($LASTEXITCODE -eq 0) {
        return @{
            Exe = "py"
            Prefix = @("-3")
        }
    }

    & python --version *> $null
    if ($LASTEXITCODE -eq 0) {
        return @{
            Exe = "python"
            Prefix = @()
        }
    }

    throw "Could not find Python via py -3 or python."
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory=$true)][string]$Label,
        [Parameter(Mandatory=$true)][string]$Exe,
        [Parameter(Mandatory=$true)][string[]]$Args
    )

    Write-Host ""
    Write-Host "=== $Label ===" -ForegroundColor Cyan
    Write-Host "> $Exe $($Args -join ' ')" -ForegroundColor DarkGray

    & $Exe @Args

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function New-DeterministicControlFiles {
    param(
        [Parameter(Mandatory=$true)][string]$SourceMesuUdhr,
        [Parameter(Mandatory=$true)][string]$OutDir
    )

    New-Item -Force -ItemType Directory -Path $OutDir | Out-Null

    $mesu = Get-Content -Path $SourceMesuUdhr -Raw -Encoding UTF8

    $duplicatedParts = @(
        $mesu.Trim(),
        "",
        $mesu.Trim(),
        "",
        $mesu.Trim(),
        "",
        $mesu.Trim()
    )

    $duplicated = $duplicatedParts -join [Environment]::NewLine

    Set-Content -Path (Join-Path $OutDir "control_mesu_udhr_duplicated_x4.txt") -Value $duplicated -Encoding UTF8

    $punctuation = @(".", ",", ";", ":", "!", "?")
    $sb = New-Object System.Text.StringBuilder
    $nonWhitespaceCount = 0
    $punctuationIndex = 0

    foreach ($ch in $mesu.ToCharArray()) {
        [void]$sb.Append($ch)

        if (-not [char]::IsWhiteSpace($ch)) {
            $nonWhitespaceCount += 1

            if (($nonWhitespaceCount % 5) -eq 0) {
                [void]$sb.Append($punctuation[$punctuationIndex % $punctuation.Count])
                [void]$sb.Append(" ")
                $punctuationIndex += 1
            }
        }
    }

    Set-Content -Path (Join-Path $OutDir "control_mesu_udhr_punctuation_noise_every5.txt") -Value $sb.ToString() -Encoding UTF8

    $rng = [System.Random]::new(1337)
    $bits = New-Object System.Text.StringBuilder

    for ($i = 0; $i -lt 12000; $i++) {
        [void]$bits.Append($rng.Next(0, 2))

        if ((($i + 1) % 80) -eq 0) {
            [void]$bits.Append([Environment]::NewLine)
        }
    }

    Set-Content -Path (Join-Path $OutDir "control_random_bits_seed1337_len12000.txt") -Value $bits.ToString() -Encoding UTF8
}

function Get-AbsZ {
    param([object]$Value)

    $parsed = 0.0
    $ok = [double]::TryParse([string]$Value, [ref]$parsed)

    if ($ok) {
        return [math]::Abs($parsed)
    }

    return 0.0
}

function Get-TrustedSurvivors {
    param(
        [Parameter(Mandatory=$true)][string]$CsvPath
    )

    if (-not (Test-Path -Path $CsvPath -PathType Leaf)) {
        return @()
    }

    $rows = @(Import-Csv -Path $CsvPath)

    $trusted = @(
        $rows |
            Where-Object {
                $_.encoding -ne "utf8_bits" -and
                $_.event -notin @("other", "digit", "whitespace") -and
                $_.metric -ne "zeta_spectral_coherence"
            } |
            Sort-Object -Property @{
                Expression = { Get-AbsZ $_.z }
                Descending = $true
            }
    )

    return $trusted
}

function Export-TrustedSurvivors {
    param(
        [AllowEmptyCollection()][object[]]$Rows,
        [Parameter(Mandatory=$true)][string]$OutCsv
    )

    $Rows = @($Rows)
    $columns = @("language_guess", "encoding", "event", "metric", "z", "p_emp", "input")

    if ($Rows.Count -eq 0) {
        Set-Content -Path $OutCsv -Encoding UTF8 -Value ($columns -join ",")
        return
    }

    $Rows |
        Select-Object language_guess, encoding, event, metric, z, p_emp, input |
        Export-Csv -Path $OutCsv -NoTypeInformation -Encoding UTF8
}

function Export-InputSummary {
    param(
        [AllowEmptyCollection()][object[]]$Rows,
        [Parameter(Mandatory=$true)][string]$OutCsv
    )

    $Rows = @($Rows)

    if ($Rows.Count -eq 0) {
        Set-Content -Path $OutCsv -Encoding UTF8 -Value "input,trusted_survivor_count,strongest_abs_z,strongest_event,strongest_metric"
        return
    }

    $summaryRows = foreach ($group in ($Rows | Group-Object input)) {
        $groupRows = @($group.Group)

        $strongest = $groupRows |
            Sort-Object -Property @{
                Expression = { Get-AbsZ $_.z }
                Descending = $true
            } |
            Select-Object -First 1

        $strongestAbsZ = 0.0
        $strongestEvent = ""
        $strongestMetric = ""

        if ($null -ne $strongest) {
            $strongestAbsZ = Get-AbsZ $strongest.z
            $strongestEvent = [string]$strongest.event
            $strongestMetric = [string]$strongest.metric
        }

        [pscustomobject]@{
            input = $group.Name
            trusted_survivor_count = $groupRows.Count
            strongest_abs_z = $strongestAbsZ
            strongest_event = $strongestEvent
            strongest_metric = $strongestMetric
        }
    }

    $summaryRows |
        Sort-Object -Property trusted_survivor_count,strongest_abs_z -Descending |
        Export-Csv -Path $OutCsv -NoTypeInformation -Encoding UTF8
}

function Convert-RowsToMarkdown {
    param(
        [AllowEmptyCollection()][object[]]$Rows,
        [Parameter(Mandatory=$true)][string[]]$Columns,
        [int]$MaxRows = 20
    )

    $Rows = @($Rows)

    if ($Rows.Count -eq 0) {
        return "_No trusted survivors after filtering._"
    }

    $limited = @($Rows | Select-Object -First $MaxRows)

    $lines = New-Object System.Collections.Generic.List[string]

    $header = "| " + ($Columns -join " | ") + " |"
    $divider = "| " + (($Columns | ForEach-Object { "---" }) -join " | ") + " |"

    $lines.Add($header)
    $lines.Add($divider)

    foreach ($row in $limited) {
        $cells = foreach ($col in $Columns) {
            $value = [string]$row.$col
            $value = $value.Replace("|", "\|")

            if ($value.Length -gt 120) {
                $value = $value.Substring(0, 117) + "..."
            }

            $value
        }

        $lines.Add("| " + ($cells -join " | ") + " |")
    }

    return ($lines -join [Environment]::NewLine)
}

Write-Host "RRLANG paper-minimum results pipeline v2" -ForegroundColor Green
Write-Host "Repo: $RepoRoot"

Assert-File -Path $RrlangExe -Label "RRLANG release binary"

if (-not (Test-Path -Path $Summariser -PathType Leaf)) {
    $Summariser = Resolve-ProjectFile `
        -PreferredPath $Summariser `
        -Label "RRLANG summariser script" `
        -Patterns @("summarise_rrlang_reports_v3.py", "*summarise*rrlang*reports*.py")
}

$MesuUdhrSource = Resolve-ProjectFile `
    -PreferredPath (Join-Path $RepoRoot "testdata\datasets_canonical\parallel\udhr\mesu\udhr_mesu_canonical.txt") `
    -Label "canonical Mesu UDHR source" `
    -Patterns @("udhr_mesu_canonical.txt", "*mesu*udhr*.txt", "*udhr*mesu*.txt")

$ThomasMesuSource = Resolve-ProjectFile `
    -PreferredPath (Join-Path $RepoRoot "testdata\datasets\private_parallel\dylan_thomas\prepared\thomas_mesu_text_only.txt") `
    -Label "Thomas Mesu text-only source" `
    -Patterns @("thomas_mesu_text_only.txt", "*thomas*mesu*.txt", "*dylan*mesu*.txt")

$BashoMesuSource = Resolve-ProjectFile `
    -PreferredPath (Join-Path $RepoRoot "testdata\datasets\private_parallel\basho\prepared\basho_mesu_text_only.txt") `
    -Label "Basho Mesu text-only source" `
    -Patterns @("basho_mesu_text_only.txt", "*basho*mesu*.txt")

$Python = Get-PythonCommand

New-Item -Force -ItemType Directory -Path $MesuRegisterRoot | Out-Null
New-Item -Force -ItemType Directory -Path $ControlsRoot | Out-Null
New-Item -Force -ItemType Directory -Path $PaperOut | Out-Null

Copy-Item -Force -Path $MesuUdhrSource -Destination (Join-Path $MesuRegisterRoot "mesu_udhr_canonical.txt")
Copy-Item -Force -Path $ThomasMesuSource -Destination (Join-Path $MesuRegisterRoot "mesu_thomas_text_only.txt")
Copy-Item -Force -Path $BashoMesuSource -Destination (Join-Path $MesuRegisterRoot "mesu_basho_text_only.txt")

New-DeterministicControlFiles -SourceMesuUdhr $MesuUdhrSource -OutDir $ControlsRoot

Invoke-Checked `
    -Label "Mesu register batch" `
    -Exe $RrlangExe `
    -Args @(
        "batch",
        "--dataset-root", $MesuRegisterRoot,
        "--out-dir", $MesuOut,
        "--language", "MesuRegister",
        "--linguistic-profile",
        "--nulls", "100",
        "--skip-existing"
    )

$mesuSummaryArgs = @()
$mesuSummaryArgs += $Python.Prefix
$mesuSummaryArgs += $Summariser
$mesuSummaryArgs += "--input"
$mesuSummaryArgs += $MesuOut
$mesuSummaryArgs += "--out"
$mesuSummaryArgs += $MesuSummary

Invoke-Checked `
    -Label "Summarise Mesu register batch" `
    -Exe $Python.Exe `
    -Args $mesuSummaryArgs

Invoke-Checked `
    -Label "Paper-minimum controls batch" `
    -Exe $RrlangExe `
    -Args @(
        "batch",
        "--dataset-root", $ControlsRoot,
        "--out-dir", $ControlsOut,
        "--language", "PaperMinimumControls",
        "--linguistic-profile",
        "--nulls", "100",
        "--skip-existing"
    )

$controlsSummaryArgs = @()
$controlsSummaryArgs += $Python.Prefix
$controlsSummaryArgs += $Summariser
$controlsSummaryArgs += "--input"
$controlsSummaryArgs += $ControlsOut
$controlsSummaryArgs += "--out"
$controlsSummaryArgs += $ControlsSummary

Invoke-Checked `
    -Label "Summarise paper-minimum controls batch" `
    -Exe $Python.Exe `
    -Args $controlsSummaryArgs

$MesuSurvivorCsv = Join-Path $MesuSummary "markov2_survivors.csv"
$ControlsSurvivorCsv = Join-Path $ControlsSummary "markov2_survivors.csv"

$MesuTrusted = @(Get-TrustedSurvivors -CsvPath $MesuSurvivorCsv)
$ControlsTrusted = @(Get-TrustedSurvivors -CsvPath $ControlsSurvivorCsv)

$MesuTrustedCsv = Join-Path $PaperOut "mesu_register_trusted_survivors.csv"
$ControlsTrustedCsv = Join-Path $PaperOut "controls_trusted_survivors.csv"
$MesuInputSummaryCsv = Join-Path $PaperOut "mesu_register_input_summary.csv"
$ControlsInputSummaryCsv = Join-Path $PaperOut "controls_input_summary.csv"

Export-TrustedSurvivors -Rows $MesuTrusted -OutCsv $MesuTrustedCsv
Export-TrustedSurvivors -Rows $ControlsTrusted -OutCsv $ControlsTrustedCsv
Export-InputSummary -Rows $MesuTrusted -OutCsv $MesuInputSummaryCsv
Export-InputSummary -Rows $ControlsTrusted -OutCsv $ControlsInputSummaryCsv

$CanonicalMainCsv = Join-Path $RepoRoot "outputs\final_tables\canonical_udhr_v0_3\final_comparison_table.csv"
$CanonicalNoPuncCsv = Join-Path $RepoRoot "outputs\final_tables\canonical_udhr_v0_3_no_punctuation\final_comparison_table.csv"

$CanonicalMesuMain = $null
$CanonicalMesuNoPunc = $null

if (Test-Path -Path $CanonicalMainCsv -PathType Leaf) {
    Copy-Item -Force -Path $CanonicalMainCsv -Destination (Join-Path $PaperOut "canonical_udhr_final_comparison_table.csv")
    $CanonicalMesuMain = @(Import-Csv -Path $CanonicalMainCsv | Where-Object { $_.language_guess -eq "mesu" }) | Select-Object -First 1
}

if (Test-Path -Path $CanonicalNoPuncCsv -PathType Leaf) {
    Copy-Item -Force -Path $CanonicalNoPuncCsv -Destination (Join-Path $PaperOut "canonical_udhr_no_punctuation_final_comparison_table.csv")
    $CanonicalMesuNoPunc = @(Import-Csv -Path $CanonicalNoPuncCsv | Where-Object { $_.language_guess -eq "mesu" }) | Select-Object -First 1
}

$MesuInputSummaryRows = @()
if (Test-Path -Path $MesuInputSummaryCsv -PathType Leaf) {
    $MesuInputSummaryRows = @(Import-Csv -Path $MesuInputSummaryCsv)
}

$ControlsInputSummaryRows = @()
if (Test-Path -Path $ControlsInputSummaryCsv -PathType Leaf) {
    $ControlsInputSummaryRows = @(Import-Csv -Path $ControlsInputSummaryCsv)
}

$ReportPath = Join-Path $PaperOut "paper_minimum_results_report.md"

$report = New-Object System.Collections.Generic.List[string]

$report.Add("# RRLANG Paper-Minimum Results Report")
$report.Add("")
$report.Add("Generated: 2026-07-07")
$report.Add("")
$report.Add("## Scope")
$report.Add("")
$report.Add("This report contains the minimum additional empirical outputs needed before drafting the RRLANG pilot/methods paper:")
$report.Add("")
$report.Add("1. Mesu register comparison: Mesu UDHR vs Thomas Mesu vs Basho Mesu.")
$report.Add("2. Deterministic controls: duplicated Mesu UDHR, punctuation-noise Mesu UDHR, and random bits.")
$report.Add("")
$report.Add("All new runs use RRLANG v0.3, linguistic-profile mode, Markov-2 survivor summaries, and 100 null samples.")
$report.Add("")
$report.Add("## Canonical UDHR calibration")
$report.Add("")

if ($null -ne $CanonicalMesuMain) {
    $report.Add("Canonical UDHR Mesu row, punctuation allowed:")
    $report.Add("")
    $report.Add("| language_guess | trusted_survivor_count | strongest_abs_z | strongest_event | strongest_metric | profile |")
    $report.Add("| --- | ---: | ---: | --- | --- | --- |")
    $report.Add("| $($CanonicalMesuMain.language_guess) | $($CanonicalMesuMain.trusted_survivor_count) | $($CanonicalMesuMain.strongest_abs_z) | $($CanonicalMesuMain.strongest_event) | $($CanonicalMesuMain.strongest_metric) | $($CanonicalMesuMain.profile) |")
    $report.Add("")
} else {
    $report.Add("_Canonical punctuation-allowed Mesu row not found in copied final table._")
    $report.Add("")
}

if ($null -ne $CanonicalMesuNoPunc) {
    $report.Add("Canonical UDHR Mesu row, punctuation excluded:")
    $report.Add("")
    $report.Add("| language_guess | trusted_survivor_count | strongest_abs_z | strongest_event | strongest_metric | profile |")
    $report.Add("| --- | ---: | ---: | --- | --- | --- |")
    $report.Add("| $($CanonicalMesuNoPunc.language_guess) | $($CanonicalMesuNoPunc.trusted_survivor_count) | $($CanonicalMesuNoPunc.strongest_abs_z) | $($CanonicalMesuNoPunc.strongest_event) | $($CanonicalMesuNoPunc.strongest_metric) | $($CanonicalMesuNoPunc.profile) |")
    $report.Add("")
} else {
    $report.Add("_Canonical no-punctuation Mesu row not found in copied final table._")
    $report.Add("")
}

$report.Add("## Mesu register input summary")
$report.Add("")
$report.Add((Convert-RowsToMarkdown -Rows $MesuInputSummaryRows -Columns @("input", "trusted_survivor_count", "strongest_abs_z", "strongest_event", "strongest_metric") -MaxRows 20))
$report.Add("")
$report.Add("## Mesu register top trusted survivors")
$report.Add("")
$report.Add((Convert-RowsToMarkdown -Rows $MesuTrusted -Columns @("language_guess", "encoding", "event", "metric", "z", "p_emp", "input") -MaxRows 30))
$report.Add("")
$report.Add("## Controls input summary")
$report.Add("")
$report.Add((Convert-RowsToMarkdown -Rows $ControlsInputSummaryRows -Columns @("input", "trusted_survivor_count", "strongest_abs_z", "strongest_event", "strongest_metric") -MaxRows 20))
$report.Add("")
$report.Add("## Controls top trusted survivors")
$report.Add("")
$report.Add((Convert-RowsToMarkdown -Rows $ControlsTrusted -Columns @("language_guess", "encoding", "event", "metric", "z", "p_emp", "input") -MaxRows 30))
$report.Add("")
$report.Add("## Generated files")
$report.Add("")
$report.Add("- mesu_register_trusted_survivors.csv")
$report.Add("- mesu_register_input_summary.csv")
$report.Add("- controls_trusted_survivors.csv")
$report.Add("- controls_input_summary.csv")
$report.Add("- canonical_udhr_final_comparison_table.csv, if available")
$report.Add("- canonical_udhr_no_punctuation_final_comparison_table.csv, if available")
$report.Add("")
$report.Add("## Paper-readiness interpretation rule")
$report.Add("")
$report.Add("Use the canonical UDHR result as calibration. Use Mesu register as the only Mesu-positive/negative register test. Use controls only as artefact sanity checks. Do not add more broad corpus results before drafting paper v0.1.")

Set-Content -Path $ReportPath -Encoding UTF8 -Value ($report -join [Environment]::NewLine)

Write-Host ""
Write-Host "DONE: paper-minimum pipeline completed." -ForegroundColor Green
Write-Host ""
Write-Host "Key output folder:"
Write-Host "  $PaperOut"
Write-Host ""
Write-Host "Main report:"
Write-Host "  $ReportPath"
Write-Host ""
Write-Host "Paste this report back into the chat:"
Write-Host "  Get-Content -Path `"$ReportPath`" -Raw"
Write-Host ""
Write-Host "Preview:"
Write-Host ""

Get-Content -Path $ReportPath -Raw

