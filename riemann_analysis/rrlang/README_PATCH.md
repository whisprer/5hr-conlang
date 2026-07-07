# RRLANG Document Pack Patch v0.2

This patch replaces the failing UDHR downloader with a GitHub/NLTK-backed downloader.

Why: Unicode stopped hosting the UDHR in Unicode files directly in Jan 2024, and OHCHR search endpoints can block scripted fetching. This patch downloads NLTK's archived `udhr2.zip` corpus package instead, then extracts selected languages into the RRLANG dataset layout.

Copy `scripts/Fetch-RRLangDocuments.ps1` and `scripts/run_rrlang_udhr_batch.ps1` over the old files in your document pack.

From the document-pack root, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Fetch-RRLangDocuments.ps1 -Only "en,fr,de,es,ja,zh"

Get-ChildItem -Recurse .\datasets\parallel\udhr -Filter *.txt | Select-Object FullName

powershell -ExecutionPolicy Bypass -File .\scripts\run_rrlang_udhr_batch.ps1 -RrLangRepo ".." -Nulls 100
```

If your RRLANG Cargo workspace is the parent folder of `testdata`, `-RrLangRepo ".."` is usually correct.
