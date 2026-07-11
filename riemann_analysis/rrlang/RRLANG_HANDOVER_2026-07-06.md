# RRLANG / Riemann-Resonant Linguistics — Handover Checkpoint

**Checkpoint date:** 2026-07-06  
**Project state:** safe checkpoint, v0.3 operability working, canonical UDHR comparison complete  
**Repo path on user machine:** `D:\code\5hr-conlang\riemann_analysis\rrlang`

---

## 1. Purpose of this handover

This handover exists because the conversation has become long and dense. Use this document to continue cleanly in a fresh chat without relying on fragile conversational memory.

The project is **RRLANG**, a Rust CLI/library research instrument for measuring positional structure in language corpora using event maps, gap/run metrics, prime-position metrics, zeta-like diagnostics, and null-model comparisons.

Important framing:

- RRLANG is a **measurement instrument**, not an oracle.
- It does **not** prove “alien language,” “AI language,” “Riemann magic,” or hidden code.
- It reports:
  - what was measured,
  - which encoding layer was used,
  - which null models were survived,
  - which interpretation level is permissible,
  - and which interpretations are forbidden.

---

## 2. Current high-level project status

### Completed

- Built RRLANG MVP v0.1.
- Built RRLANG v0.2 with:
  - `bit_text` encoder,
  - Markov-1 null,
  - Markov-2 null,
  - gap-order shuffle,
  - hyphen-policy handling.
- Built local document/corpus pack tooling.
- Fetched / prepared multilingual UDHR files.
- Ran early broad/curated batches.
- Diagnosed Tatoeba/Wikipedia files as too large for v0.2 whole-file runs.
- Built RRLANG v0.3 operability patch.
- Applied v0.3 patch successfully.
- Built release binary.
- Confirmed new `batch` command works.
- Ran clean canonical UDHR batch:
  - 17/17 inputs completed,
  - 0 failed,
  - `--linguistic-profile`,
  - raw UTF-8 skipped,
  - `--nulls 100`.
- Generated summary CSVs.
- Repaired language labels.
- Built final canonical UDHR comparison tables.
- Backed up / committed latest milestone successfully.

### Current state

The project is now ready for the next empirical stage:

1. Mesu register comparison:
   - Mesu UDHR vs Mesu Thomas vs Mesu Basho.
2. v0.3 controls comparison:
   - random bits,
   - duplicated Mesu UDHR,
   - punctuation-noise Mesu UDHR.
3. Combined paper-results table.
4. Paper draft v0.1.

---

## 3. Repository and important paths

### Repo root

```powershell
D:\code\5hr-conlang\riemann_analysis\rrlang
```

### Important dataset roots

```text
testdata\datasets\
testdata\datasets_canonical\
testdata\datasets_canonical\parallel\udhr\
```

### Important output folders

```text
outputs\canonical_udhr_v0_3_nulls100
outputs\summary_canonical_udhr_v0_3_nulls100
outputs\summary_canonical_udhr_v0_3_nulls100_labeled
outputs\final_tables\canonical_udhr_v0_3
outputs\final_tables\canonical_udhr_v0_3_no_punctuation
```

### Important scripts

```text
scripts\summarise_rrlang_reports_v3.py
scripts\repair_canonical_udhr_labels.ps1
scripts\build_final_comparison_table.py
```

### Important binary

```text
target\release\rrlang.exe
```

---

## 4. RRLANG v0.3 status

The v0.3 operability patch added:

```text
--skip-raw / --no-raw
--linguistic-profile
--fast-profile
--max-chars <N|none>
--progress
batch command
batch --skip-existing / --resume
batch --continue-on-error
```

The successful canonical UDHR batch command was:

```powershell
cd D:\code\5hr-conlang\riemann_analysis\rrlang

.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets_canonical\parallel\udhr `
  --out-dir .\outputs\canonical_udhr_v0_3_nulls100 `
  --language CanonicalUDHR `
  --linguistic-profile `
  --nulls 100 `
  --skip-existing
```

Observed successful result:

```text
RRLANG BATCH RUN
================
inputs: 17
encodings: grapheme,grapheme_class,word_boundary,frequency_class
nulls: 100
max_chars: none
skip_existing: true

Batch summary:
  completed: 17
  skipped:   0
  failed:    0
  total:     17
```

Per-language runtime highlights:

```text
ar    57.56s
cy    79.59s
de    87.85s
el    41.61s
en    90.24s
es    86.71s
fr    80.64s
he    45.03s
is    73.25s
ja    34.56s
la    67.67s
mesu   6.15s
ru    44.47s
sw    85.13s
tr    72.56s
xh    80.37s
zh    23.15s
```

---

## 5. Canonical UDHR summary outputs

Summary folder:

```text
outputs\summary_canonical_udhr_v0_3_nulls100_labeled
```

Files created:

```text
alert_events.csv
alert_summary.csv
markov2_survivors.csv
metrics_flat.csv
report_index.csv
top_deviations.csv
```

Final table folders:

```text
outputs\final_tables\canonical_udhr_v0_3
outputs\final_tables\canonical_udhr_v0_3_no_punctuation
```

Files produced in each:

```text
final_comparison_table.csv
final_comparison_table.md
trusted_markov2_survivors.csv
metric_family_summary.csv
strongest_by_language.csv
```

---

## 6. Canonical UDHR final comparison results

### Main trusted table

Command used:

```powershell
py -3 .\scripts\build_final_comparison_table.py `
  --summary .\outputs\summary_canonical_udhr_v0_3_nulls100_labeled `
  --out .\outputs\final_tables\canonical_udhr_v0_3
```

Terminal summary:

```text
RRLANG FINAL COMPARISON TABLE
=============================
Reports indexed:    17
Survivor rows read: 144
Trusted rows kept:  60
Languages compared: 17
```

Main final table:

```text
rank language_guess trusted_survivor_count strongest_abs_z strongest_event strongest_metric   profile
1    xh             10                     10.369615       word_boundary   gap_entropy        strong
2    is             8                      6.165621        punctuation     gap_entropy        strong
3    tr             8                      5.929918        word_boundary   prime_gap_affinity strong
4    de             5                      12.614508       word_boundary   prime_gap_affinity strong
5    sw             4                      6.349023        punctuation     gap_entropy        moderate
6    la             4                      6.011814        word_boundary   run_entropy        moderate
7    es             4                      5.708398        word_boundary   gap_entropy        moderate
8    he             4                      5.014382        word_boundary   gap_entropy        moderate
9    ar             4                      4.805893        word_boundary   gap_entropy        moderate
10   ja             2                      10.155028       word_boundary   gap_entropy        strong
11   zh             2                      5.888456        word_boundary   gap_entropy        moderate
12   el             2                      5.755029        word_boundary   prime_gap_affinity moderate
13   fr             2                      4.042247        vowel           prime_gap_affinity weak
14   en             1                      9.014021        word_boundary   prime_gap_affinity strong
15   cy             0                      0.0                                                quiet
16   mesu           0                      0.0                                                quiet
17   ru             0                      0.0                                                quiet
```

### Stricter no-punctuation table

Command used:

```powershell
py -3 .\scripts\build_final_comparison_table.py `
  --summary .\outputs\summary_canonical_udhr_v0_3_nulls100_labeled `
  --out .\outputs\final_tables\canonical_udhr_v0_3_no_punctuation `
  --exclude-punctuation
```

Terminal summary:

```text
Reports indexed:    17
Survivor rows read: 144
Trusted rows kept:  40
Languages compared: 17
```

No-punctuation table:

```text
rank language_guess trusted_survivor_count strongest_abs_z strongest_event strongest_metric   profile
1    de             5                      12.614508       word_boundary   prime_gap_affinity strong
2    xh             4                      10.369615       word_boundary   gap_entropy        strong
3    la             4                      6.011814        word_boundary   run_entropy        moderate
4    tr             4                      5.929918        word_boundary   prime_gap_affinity moderate
5    es             4                      5.708398        word_boundary   gap_entropy        moderate
6    he             4                      5.014382        word_boundary   gap_entropy        moderate
7    ar             4                      4.805893        word_boundary   gap_entropy        moderate
8    ja             2                      10.155028       word_boundary   gap_entropy        strong
9    zh             2                      5.888456        word_boundary   gap_entropy        moderate
10   el             2                      5.755029        word_boundary   prime_gap_affinity moderate
11   is             2                      4.684722        vowel           prime_gap_affinity weak
12   fr             2                      4.042247        vowel           prime_gap_affinity weak
13   en             1                      9.014021        word_boundary   prime_gap_affinity strong
14   cy             0                      0.0                                                quiet
15   mesu           0                      0.0                                                quiet
16   ru             0                      0.0                                                quiet
17   sw             0                      0.0                                                quiet
```

---

## 7. Current scientific interpretation

### The clean result

In the canonical UDHR v0.3 comparison:

```text
Mesu canonical UDHR has 0 filtered Markov-2 survivors.
```

This remains true:

- with punctuation allowed,
- with punctuation excluded,
- using the canonical one-file-per-language UDHR dataset,
- using linguistic-profile encodings,
- using Markov-2 nulls,
- with 100 null samples.

### Interpretation

This is not a failure. It is a strong calibration result.

Current honest statement:

> In the canonical UDHR v0.3 run, Mesu has zero filtered Markov-2 survivors, both with punctuation allowed and with punctuation excluded. Several natural-language UDHR translations retain Markov-2-surviving structure, especially in word-boundary, punctuation, and vowel/consonant-related metrics. Mesu UDHR is therefore comparatively quiet in this legal-prose baseline.

### What this means for the paper

This supports the claim that RRLANG is **not simply flattering Mesu** or producing desired “Mesu specialness” artefacts.

The likely interpretation is:

> Mesu’s interesting signals, if present, are more likely to appear in register/form/fossilisation comparisons than in UDHR-style legal prose.

---

## 8. What not to claim

Do **not** claim:

- Mesu has proven special Riemann-resonant structure.
- RRLANG detects artificial languages.
- RRLANG detects alien languages.
- Prime numbers cause linguistic structure.
- Zeta metrics currently prove anything.
- A single UDHR translation is enough for a language-wide conclusion.
- The current p-values are final/fully corrected significance claims.

Also avoid overreading:

- raw UTF-8 bit findings,
- `other` event findings,
- digit findings,
- whitespace-only findings,
- zeta-spectral giant z-scores,
- cross-script word-boundary comparisons without caveats.

---

## 9. What can be safely claimed

Safe claims:

1. RRLANG v0.3 can run controlled batch analyses over multilingual corpora.
2. The v0.3 canonical UDHR run completed 17/17 files successfully.
3. Linguistic-profile mode avoids raw UTF-8 diagnostic layers.
4. The canonical UDHR table shows natural-language survivors in word-boundary, punctuation, and vowel/consonant-related metrics.
5. Mesu UDHR is comparatively quiet under the filtered Markov-2 survivor criterion.
6. The tool has useful anti-overclaiming behavior: it does not merely identify Mesu as special because Mesu is the project’s constructed-language case study.
7. More evidence is needed before any strong language-family or constructed-language claim.

---

## 10. Immediate next recommended experiment

### Mesu register comparison

Goal:

Compare Mesu legal prose vs Mesu poetry/register.

Files:

```text
Mesu UDHR canonical
Thomas Mesu text-only
Basho Mesu text-only
```

Research question:

> Mesu is quiet in UDHR legal prose. Does it become structurally marked in poetry/register?

### Commands to prepare dataset

Run from repo root:

```powershell
cd D:\code\5hr-conlang\riemann_analysis\rrlang

New-Item -Force -ItemType Directory .\testdata\datasets_canonical\mesu_register | Out-Null

Copy-Item -Force `
  .\testdata\datasets_canonical\parallel\udhr\mesu\udhr_mesu_canonical.txt `
  .\testdata\datasets_canonical\mesu_register\mesu_udhr_canonical.txt

Copy-Item -Force `
  .\testdata\datasets\private_parallel\dylan_thomas\prepared\thomas_mesu_text_only.txt `
  .\testdata\datasets_canonical\mesu_register\mesu_thomas_text_only.txt

Copy-Item -Force `
  .\testdata\datasets\private_parallel\basho\prepared\basho_mesu_text_only.txt `
  .\testdata\datasets_canonical\mesu_register\mesu_basho_text_only.txt
```

### Run Mesu register batch

```powershell
.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets_canonical\mesu_register `
  --out-dir .\outputs\mesu_register_v0_3_nulls100 `
  --language MesuRegister `
  --linguistic-profile `
  --nulls 100 `
  --skip-existing
```

### Summarise Mesu register batch

```powershell
py -3 .\scripts\summarise_rrlang_reports_v3.py `
  --input .\outputs\mesu_register_v0_3_nulls100 `
  --out .\outputs\summary_mesu_register_v0_3_nulls100
```

### Show clean Mesu register survivors

```powershell
Import-Csv .\outputs\summary_mesu_register_v0_3_nulls100\markov2_survivors.csv |
  Where-Object {
    $_.encoding -ne "utf8_bits" -and
    $_.event -notin @("other", "digit", "whitespace") -and
    $_.metric -ne "zeta_spectral_coherence"
  } |
  Sort-Object { [math]::Abs([double]$_.z) } -Descending |
  Select-Object language_guess, encoding, event, metric, z, p_emp, input |
  Format-Table -AutoSize | Out-String -Width 260
```

Potential issue:

The summariser may label all Mesu register rows poorly if it cannot parse the output names. If so, repair/infer from `input` path or output filename manually. The actual metrics are more important than the label.

---

## 11. Second recommended experiment

### v0.3 controls comparison

Goal:

Confirm known artefact controls behave like artefacts under the current v0.3 path.

Run:

```powershell
.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets\controls `
  --out-dir .\outputs\controls_v0_3_nulls100 `
  --language Controls `
  --linguistic-profile `
  --nulls 100 `
  --skip-existing
```

Summarise:

```powershell
py -3 .\scripts\summarise_rrlang_reports_v3.py `
  --input .\outputs\controls_v0_3_nulls100 `
  --out .\outputs\summary_controls_v0_3_nulls100
```

Then inspect trusted survivors:

```powershell
Import-Csv .\outputs\summary_controls_v0_3_nulls100\markov2_survivors.csv |
  Where-Object {
    $_.encoding -ne "utf8_bits" -and
    $_.event -notin @("other", "digit", "whitespace") -and
    $_.metric -ne "zeta_spectral_coherence"
  } |
  Sort-Object { [math]::Abs([double]$_.z) } -Descending |
  Select-Object encoding, event, metric, z, p_emp, input |
  Format-Table -AutoSize | Out-String -Width 260
```

Expected:

- duplicated Mesu should scream,
- punctuation-noise Mesu should show punctuation/spacing artefacts,
- random bits should be quiet under linguistic profile or irrelevant if not text-like.

---

## 12. Paper readiness

The paper can now be written as a **pilot/methods paper**, but two more outputs are recommended before final empirical write-up:

1. Mesu register comparison.
2. v0.3 controls comparison.

Paper title candidates:

```text
Riemann-Resonant Linguistics: A Prime-Gap and Null-Model Framework for Positional Structure in Language Corpora

RRLANG: A Controlled Prime-Spectral Instrument for Linguistic Positional Structure

Prime-Gap Linguistics Without Numerology: A Null-Model Framework for Corpus Positional Structure
```

Recommended framing:

- The paper is about the **method and instrument**.
- Mesu is a case study, not the center.
- Canonical UDHR results are a calibration case.
- Mesu UDHR being quiet is a strength, not a weakness.

Suggested paper results sections:

1. Instrument design.
2. Encoding ladder and event maps.
3. Null models and anti-overclaiming alerts.
4. Canonical UDHR comparison.
5. Mesu register comparison.
6. Controls and artefact tests.
7. Limitations.
8. Future work.

---

## 13. Known limitations to include

- Only 17 UDHR translations in canonical run.
- Single translation per language.
- UDHR translations differ in style, formatting, punctuation, segmentation.
- Word-boundary events are not directly comparable across scripts.
- Markov-2 null is useful but not sufficient for all higher-order structure.
- 100 null samples is decent but not final; stronger runs may need 1000.
- Multiple-comparisons correction/FDR not yet implemented.
- Zeta-like metrics remain diagnostic and unstable in some cases.
- Grapheme handling is a Unicode scalar approximation, not true full grapheme-cluster segmentation.
- No morphology-aware or syntax-aware encoding yet.
- No windowed corpus analysis yet.
- No cross-corpus natural-language baselines beyond UDHR in the clean v0.3 result.

---

## 14. v0.3.1 / v0.4 future patch ideas

Highest priority:

```text
built-in final comparison table command
built-in trusted-survivor filter
language-label extraction for canonical batch outputs
canonical dataset builder
controls batch preset
mesu-register preset
--exclude-punctuation option in Rust summary
FDR / multiple-comparison correction
--nulls 1000 support and timing estimates
windowed analysis: --window-size / --stride
true grapheme cluster support
script-aware vowel/consonant classification
word-boundary caution flags for no-space scripts
```

---

## 15. Safe restart prompt for a new chat

Paste this into a fresh conversation:

```text
We are continuing the RRLANG / Riemann-Resonant Linguistics project.

Current repo:
D:\code\5hr-conlang\riemann_analysis\rrlang

RRLANG v0.3 operability patch is applied and working. The release binary exists at:
target\release\rrlang.exe

Canonical UDHR v0.3 run completed successfully:
17/17 files, 0 failed, --linguistic-profile, --nulls 100, raw UTF-8 skipped.

Final canonical UDHR tables were built in:
outputs\final_tables\canonical_udhr_v0_3
outputs\final_tables\canonical_udhr_v0_3_no_punctuation

Main result:
Mesu canonical UDHR has 0 filtered Markov-2 survivors. Natural-language UDHR translations retain survivors mostly in word_boundary, punctuation, and vowel/consonant metrics.

Next planned task:
Run Mesu register comparison:
Mesu UDHR vs Thomas Mesu vs Basho Mesu.

Please help me continue from the handover document.
```

---

## 16. Recommended stopping point

This is a clean and safe handover checkpoint.

Do not do more broad corpus harvesting before Mesu register + controls are done.

Next move:

```text
Run Mesu register comparison.
```
