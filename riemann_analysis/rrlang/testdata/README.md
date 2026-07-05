# RRLANG Document Pack v0.1

This pack is the dataset scaffold for the Riemann-Resonant Linguistics experiments.

It includes the Mesu seed texts already supplied by the user, plus scripts to fetch a cross-linguistic UDHR set and optional FLORES-200 material.

## Why this is not just a pile of downloaded files

The safest research route is source-tracked retrieval. The scripts write a `source.json` beside every downloaded document so later RRLANG reports can cite exactly where each text came from.

## Included immediately

- Mesu UDHR prepared text
- Mesu Dylan Thomas prepared text
- Mesu Bashō prepared text
- Mesu fossil/root/reference notes copied from the seed dataset
- Control generator script

## Fetch the UDHR target-language pack

From this folder in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangDocuments.ps1
```

Fetch only a few languages:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangDocuments.ps1 -Only "en,fr,de,es,ja,zh"
```

The fetcher tries:

1. OHCHR official translation pages
2. EFELE / UDHR-in-XML individual plain-text files
3. EFELE bulk plain-text zip

Downloaded files land under:

```text
datasets/parallel/udhr/target_languages/<code>/udhr_<code>.txt
```

## Generate controls

```powershell
py -3 .\scripts\generate_controls.py
```

## Run RRLANG over the UDHR pack

Adjust the RrLangRepo path if needed:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run_rrlang_udhr_batch.ps1 -RrLangRepo "..\rrlang_mvp_v0_2" -Nulls 100
```

## Optional FLORES-200

FLORES-200 is useful for modern parallel prose. It is optional because it uses the Hugging Face `datasets` package and has CC-BY-SA attribution/share-alike obligations.

```powershell
py -3 -m pip install datasets
py -3 .\scripts\fetch_flores200_optional.py --split devtest
```

## Core target languages

- English
- Welsh
- French
- Spanish
- German
- Russian
- Arabic
- Hebrew
- Chinese
- Japanese
- Turkish
- Swahili
- Xhosa
- Icelandic
- Latin
- Greek
- Mesu

## Licensing notes

UDHR is widely described as public domain/copyright-free, but individual packaging and sites may have their own metadata or terms. Keep `source.json` files with every document.

FLORES-200 is CC-BY-SA 4.0.

Tatoeba textual downloads are CC BY 2.0 FR and require attribution.

OPUS contains many corpora with mixed licenses; do not mix OPUS data into a publishable pack without per-corpus license checks.
