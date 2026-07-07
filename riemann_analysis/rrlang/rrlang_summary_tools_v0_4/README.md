# RRLANG Summary Tools v0.4

This pack fixes the canonical UDHR language label issue where `language_guess` was reported as `unknown` for v0.3 batch output names such as:

```text
ar_udhr_ar_canonical.json
mesu_udhr_mesu_canonical.json
```

## Install

Copy this file into your RRLANG repo:

```text
repair_canonical_udhr_labels.ps1
```

to:

```text
D:\code\5hr-conlang\riemann_analysis\rrlang\scripts\repair_canonical_udhr_labels.ps1
```

## Run

From the RRLANG repo root:

```powershell
cd D:\code\5hr-conlang\riemann_analysis\rrlang

powershell -ExecutionPolicy Bypass -File .\scripts\repair_canonical_udhr_labels.ps1 `
  -InputSummary .\outputs\summary_canonical_udhr_v0_3_nulls100 `
  -OutputSummary .\outputs\summary_canonical_udhr_v0_3_nulls100_labeled
```

Then query the `_labeled` folder instead of the original summary folder.
