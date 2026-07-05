# RRLANG Document Pack Patch v0.3 — Mass Corpus Harvester

This patch adds a broader dataset harvester for RRLANG. It is designed to populate:

- UDHR parallel texts via NLTK's UDHR packages
- FLORES-200 parallel benchmark texts via Hugging Face `datasets` when available
- Tatoeba CC0 monolingual sentence samples
- Wikipedia native/factual prose via MediaWiki API random/extract queries
- Project Gutenberg curated literary texts by user-supplied Gutenberg IDs
- Synthetic controls

The goal is not to blindly download terabytes. The default run is a sane research pack; `-Big` and `-Huge` increase limits.

## Apply

Copy the `scripts/` folder over your current document pack's `scripts/` folder.

Your current root appears to be:

```text
D:\code\5hr-conlang\riemann_analysis\rrlang\testdata
```

Run commands from that folder.

## Quick first real pack

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangMegaPack.ps1 -Preset Core
```

## Bigger pack

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangMegaPack.ps1 -Preset Broad -WikiPagesPerLang 40 -TatoebaPerLang 20000
```

## Huge-ish local pack

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangMegaPack.ps1 -Preset Broad -Big
```

## Outputs

Files land under:

```text
datasets/
  parallel/udhr/
  parallel/flores200/
  native/wikipedia_api/
  native/tatoeba_cc0/
  native/gutenberg/
  controls/
_cache/
logs/
```

Every downloaded text gets a sidecar `source.json` where practical.
