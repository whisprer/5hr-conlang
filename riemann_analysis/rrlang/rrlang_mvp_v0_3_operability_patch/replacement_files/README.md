# rrlang

`rrlang` is the MVP Rust research instrument for **Riemann-Resonant Linguistics**.

It is intentionally built as a measurement engine, not an oracle. It reports positional metrics, null-model comparisons, and evidence-tiered warnings for language-system analysis.

## Current MVP scope: v0.2.0

Supported encoding layers:

- `utf8_bits` — raw UTF-8 diagnostic bitstream, not primary linguistic evidence.
- `bit_text` — treats literal `0` and `1` characters as the bitstream itself; use this for random-bit text controls.
- `grapheme` — deterministic Unicode-scalar stream used as an MVP grapheme approximation.
- `grapheme_class` — broad classes such as vowel, consonant, digit, whitespace, punctuation, hyphen_boundary, other.
- `word_boundary` — whitespace-derived boundary bitmap.
- `frequency_class` — sample-local token frequency classes: hapax, mid-frequency, high-frequency.

Supported metric families:

- Event density
- Shannon symbol entropy
- Gap entropy
- Run entropy
- Prime-indexed occupancy bias
- Prime-gap affinity
- Modular residue peaks for mod 2, 3, 5, and 7
- Zeta-like spectral coherence
- Critical-line symmetry score
- Basic anti-pattern and artefact alerts

Supported null models in v0.2:

- `density_shuffle` — preserves event count / binary density.
- `markov_1` — preserves first-order local transition tendencies approximately.
- `markov_2` — preserves second-order local transition tendencies approximately.
- `gap_order_shuffle` — preserves event-gap multiset while shuffling its order; for `word_boundary`, this acts as the MVP word-length/order control.

v0.2 also adds explicit hyphen handling for Mesu and other compound-heavy texts:

- `punctuation` — hyphens are punctuation.
- `morpheme_boundary` — hyphens are morpheme-boundary events.
- `word_internal` — hyphens remain inside tokens.
- `remove` — hyphens are removed during preprocessing.

This MVP deliberately does **not** yet include full phonemic, morphemic, POS, or diachronic layers. Those are later extensions.

## Build

From the repository root:

```bash
cargo build --release
```

The binary will be at:

```bash
target/release/rrlang
```

On Windows PowerShell:

```powershell
cargo build --release
.\target\release\rrlang.exe help
```

## Run a quick inspection

```bash
cargo run -p rrlang -- inspect --input testdata/tiny/english.txt
```

## Run an MVP analysis

```bash
cargo run -p rrlang -- analyse \
  --input testdata/tiny/english.txt \
  --language English \
  --nulls 100 \
  --out outputs/english_report.json \
  --text-out outputs/english_report.txt
```

PowerShell version:

```powershell
cargo run -p rrlang -- analyse `
  --input testdata/tiny/english.txt `
  --language English `
  --nulls 100 `
  --out outputs/english_report.json `
  --text-out outputs/english_report.txt
```

## Run Mesu with hyphen-as-morpheme-boundary mode

```powershell
cargo run -p rrlang -- analyse `
  --input ..\rrlang_dataset_seed_v0_1\datasets\parallel\udhr\mesu\udhr_mesu_text_only.txt `
  --language Mesu `
  --hyphen-policy morpheme_boundary `
  --nulls 100 `
  --out outputs\mesu_udhr_v0_2_report.json `
  --text-out outputs\mesu_udhr_v0_2_report.txt
```

## Run random-bit text correctly

Use `bit_text` for a text file containing literal `0` and `1` characters:

```powershell
cargo run -p rrlang -- analyse `
  --input ..\rrlang_dataset_seed_v0_1\datasets\controls\random_bits_10k.txt `
  --language ControlRandomBits `
  --encodings bit_text `
  --nulls 100 `
  --out outputs\random_bits_bit_text_report.json `
  --text-out outputs\random_bits_bit_text_report.txt
```

## Run from config

```bash
cargo run -p rrlang -- analyse --config examples/config_basic.toml
```

## Interpretation discipline

`rrlang` reports:

- what was measured,
- which encoding layer it came from,
- how it compared against multiple null models,
- what alerts were triggered,
- and what interpretation level is allowed.

It does **not** report:

- proof of artificiality,
- proof of AI authorship,
- proof of alien origin,
- proof of hidden prime causality,
- or any claim about the Riemann Hypothesis.

Raw UTF-8 and `bit_text` findings are diagnostic only unless they are intentionally being used as controls.

## Known MVP limitations

- The `grapheme` layer currently uses Unicode scalar values, not full Unicode extended grapheme clusters.
- The word-boundary layer is whitespace-derived.
- The frequency-class layer calculates frequencies only inside the supplied sample.
- Markov nulls are approximate binary-event controls, not full linguistic Markov models.
- `gap_order_shuffle` is a first word-length/gap-order control, not a complete syntactic control.
- Results should be treated as exploratory.

These limitations are design-contained and reported in tool notes/alerts rather than hidden.


## v0.3 operability patch

This version adds the practical controls discovered during corpus testing:

- `--skip-raw` / `--no-raw` removes `utf8_bits` and `bit_text` diagnostic encodings.
- `--linguistic-profile` uses `grapheme,grapheme_class,word_boundary,frequency_class`.
- `--fast-profile` uses the linguistic profile, `--nulls 25`, and `--max-chars 25000` unless you override those options later on the command line.
- `--max-chars <N|none>` caps the cleaned text before analysis and records an `INPUT_TRUNCATED` warning in the report.
- `batch` recursively analyses `.txt` files with `--skip-existing` resume support.

Canonical UDHR run:

```powershell
cargo build --release -p rrlang

.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets_canonical\parallel\udhr `
  --out-dir .\outputs\canonical_udhr_v0_3_nulls100 `
  --language CanonicalUDHR `
  --linguistic-profile `
  --nulls 100 `
  --skip-existing
```

Broad fast calibration run:

```powershell
.\target\release\rrlang.exe batch `
  --dataset-root .\testdata\datasets `
  --out-dir .\outputs\broad_fast_v0_3 `
  --language BatchCorpus `
  --fast-profile `
  --skip-existing `
  --continue-on-error
```
