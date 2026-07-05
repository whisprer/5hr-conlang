#!/usr/bin/env python3
"""
Optional FLORES-200 fetch helper.

This uses Hugging Face's datasets library if installed:
    pip install datasets
Then it downloads facebook/flores for selected language codes.

This is deliberately optional because FLORES is bigger and has CC-BY-SA attribution/share-alike duties.
"""
from pathlib import Path
import argparse, json

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--split", default="devtest")
    ap.add_argument("--langs", default="eng_Latn,fra_Latn,spa_Latn,deu_Latn,rus_Cyrl,arb_Arab,heb_Hebr,zho_Hans,jpn_Jpan,tur_Latn,swh_Latn,xho_Latn,isl_Latn,ell_Grek")
    args = ap.parse_args()
    try:
        from datasets import load_dataset
    except Exception as e:
        raise SystemExit("Install datasets first: pip install datasets") from e
    root = Path(args.root).resolve()
    outbase = root / "datasets" / "parallel" / "flores200" / args.split
    outbase.mkdir(parents=True, exist_ok=True)
    for lang in [x.strip() for x in args.langs.split(',') if x.strip()]:
        ds = load_dataset("facebook/flores", lang, split=args.split)
        lines = [row["sentence"] for row in ds]
        (outbase / f"{lang}.{args.split}.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        (outbase / f"{lang}.{args.split}.source.json").write_text(json.dumps({"source":"facebook/flores", "license":"CC-BY-SA-4.0", "lang":lang, "split":args.split}, indent=2), encoding="utf-8")
        print("OK", lang, len(lines))
if __name__ == "__main__": main()
