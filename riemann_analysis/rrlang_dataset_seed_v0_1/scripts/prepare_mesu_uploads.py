#!/usr/bin/env python3
"""
prepare_mesu_uploads.py

Regenerates the prepared Mesu text-only files from the raw user-provided files.

Usage:
  python scripts/prepare_mesu_uploads.py --root .

This script is intentionally conservative. Review outputs before treating them as canonical.
"""
from __future__ import annotations

import argparse
import re
import unicodedata
from pathlib import Path


def norm(s: str) -> str:
    return unicodedata.normalize("NFC", s).replace("\r\n", "\n").replace("\r", "\n").strip() + "\n"


def extract_udhr_mesu(txt: str) -> str:
    parts: list[str] = []
    m = re.search(r"\*\*(PAN-LUM[^\n]+)\*\*", txt)
    if m:
        parts.append(m.group(1).strip())
    for m in re.finditer(r"### Mesu:\s*```(?:[^\n]*)\n(.*?)```", txt, flags=re.S):
        parts.append(m.group(1).strip())
    return norm("\n\n".join(parts))


def code_blocks_first_segment(txt: str) -> str:
    parts: list[str] = []
    for block in re.findall(r"```(?:[^\n]*)\n(.*?)```", txt, flags=re.S):
        seg: list[str] = []
        for line in block.splitlines():
            if not line.strip():
                break
            seg.append(line.rstrip())
        if seg:
            parts.append("\n".join(seg))
    return norm("\n\n".join(parts))


def extract_basho(txt: str) -> str:
    start = txt.find("attempt 4: the final form")
    if start == -1:
        return ""
    sub = txt[start:].split("reading:", 1)[0]
    lines = [line.strip() for line in sub.splitlines()[1:] if line.strip()]
    return norm("\n".join(lines))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="Dataset root")
    args = ap.parse_args()

    root = Path(args.root)
    raw = root / "datasets" / "constructed" / "mesu" / "source_raw"

    udhr_raw = (raw / "UDHR-mesu-translation.md").read_text(encoding="utf-8", errors="replace")
    thomas_raw = (raw / "mesu-thomas-translation.md").read_text(encoding="utf-8", errors="replace")
    basho_raw = (raw / "basho.txt").read_text(encoding="utf-8", errors="replace")
    poetry_raw = (raw / "poetry.txt").read_text(encoding="utf-8", errors="replace")

    (root / "datasets/parallel/udhr/mesu/udhr_mesu_text_only.txt").write_text(extract_udhr_mesu(udhr_raw), encoding="utf-8")
    (root / "datasets/constructed/mesu/prepared/udhr_mesu_text_only.txt").write_text(extract_udhr_mesu(udhr_raw), encoding="utf-8")
    (root / "datasets/private_parallel/dylan_thomas/prepared/thomas_mesu_text_only.txt").write_text(code_blocks_first_segment(thomas_raw), encoding="utf-8")
    (root / "datasets/private_parallel/basho/prepared/basho_mesu_text_only.txt").write_text(extract_basho(basho_raw), encoding="utf-8")
    (root / "datasets/private_parallel/discord_poetry/prepared/discord_poetry_mesu_text_only.txt").write_text(code_blocks_first_segment(poetry_raw), encoding="utf-8")

    print("Prepared Mesu files regenerated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
