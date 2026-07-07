# RRLANG Summary Tools v0.3

This fixes the broken v0.2 package where the installed file accidentally contained the artifact-builder script instead of the actual summariser.

## Install

Extract this zip, then from the extracted folder run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install_summary_tool_v3.ps1
```

Or manually copy:

```text
summarise_rrlang_reports_v3.py
```

to:

```text
D:\code\5hr-conlang\riemann_analysis\rrlang\scripts\summarise_rrlang_reports_v3.py
```

## Run curated summary

```powershell
cd D:\code\5hr-conlang\riemann_analysis\rrlang

py -3 .\scripts\summarise_rrlang_reports_v3.py `
  --input .\outputs\corpus_batch_curated `
  --out .\outputs\summary_curated
```

## Check outputs

```powershell
Get-ChildItem .\outputs\summary_curated
```

Expected files:

```text
alert_events.csv
alert_summary.csv
markov2_survivors.csv
metrics_flat.csv
report_index.csv
top_deviations.csv
```
