# RRLANG Dataset Seed v0.1

This bundle organises the Mesu materials uploaded for the first real RRLANG tests.

## Status

This is a **research seed dataset**, not a publishable public corpus. Some branches are private/copyright-sensitive or user-provided and should not be redistributed without review.

## High-value branches

- `datasets/parallel/udhr/mesu/udhr_mesu_text_only.txt`
  - first gold Mesu legal/factual parallel anchor.
- `datasets/private_parallel/dylan_thomas/prepared/thomas_mesu_text_only.txt`
  - private poetic/kenning register test.
- `datasets/private_parallel/basho/prepared/basho_mesu_text_only.txt`
  - private minimal stillness/haiku test.
- `datasets/constructed/mesu/lexicon/`
  - roots, kennings, and fossil registry seed tables for future morpheme/fossil work.

## First experiment

Run RRLANG on:

1. `datasets/parallel/udhr/mesu/udhr_mesu_text_only.txt`
2. `datasets/controls/random_bits_10k.txt`
3. `datasets/controls/duplicated_udhr_mesu.txt`
4. `datasets/controls/punctuation_noise_udhr_mesu.txt`
5. `datasets/controls/code_sample_rust.rs`

Then add official/public UDHR translations for other languages into:

`datasets/parallel/udhr/<language-code>/udhr_<language-code>.txt`

## Important caution

The prepared Mesu text-only files were extracted conservatively from Markdown/code blocks. Review them before treating them as canonical.
