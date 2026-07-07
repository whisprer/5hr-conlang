# RRLANG MVP v0.3 Operability Patch

This patch replaces a small set of Rust source files in the existing `rrlang` repo.
It adds the practical corpus-run controls we discovered we need:

- `--skip-raw` / `--no-raw`
- `--linguistic-profile`
- `--fast-profile`
- `--max-chars <N|none>`
- `--progress`
- built-in `batch` command with `--skip-existing` / `--resume`

## Apply

Extract this zip, then from your existing repo root run:

```powershell
cd D:\code\5hr-conlang\riemann_analysis\rrlang
powershell -ExecutionPolicy Bypass -File D:\code\5hr-conlang\riemann_analysis\rrlang\rrlang_mvp_v0_3_operability_patch\apply_rrlang_v0_3_operability_patch.ps1
```

Then build:

```powershell
cargo build --release -p rrlang
.\target\release\rrlang.exe --help
```

## Canonical UDHR run

```powershell
.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets_canonical\parallel\udhr `
  --out-dir .\outputs\canonical_udhr_v0_3_nulls100 `
  --language CanonicalUDHR `
  --linguistic-profile `
  --nulls 100 `
  --skip-existing
```

## Broad fast calibration run

```powershell
.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets `
  --out-dir .\outputs\broad_fast_v0_3 `
  --language BatchCorpus `
  --fast-profile `
  --skip-existing `
  --continue-on-error
```

## Rollback

The apply script creates a timestamped backup folder in your repo root.
Copy those files back over the repo files if anything misbehaves.
