#!/usr/bin/env python3
from __future__ import annotations

import argparse
import bz2
import csv
import io
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tarfile
import time
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Tuple
from urllib.parse import urlencode
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError

USER_AGENT = "rrlang-research-corpus-builder/0.3 (local research; contact: user-local)"

LANGS = {
    # rrlang code, display, wikipedia code, Tatoeba ISO3, FLORES-200 code, UDHR filename hints
    "en": {"name":"English", "wiki":"en", "tatoeba":"eng", "flores":"eng_Latn", "udhr":["English"]},
    "cy": {"name":"Welsh", "wiki":"cy", "tatoeba":"cym", "flores":"cym_Latn", "udhr":["Welsh", "Cymraeg"]},
    "fr": {"name":"French", "wiki":"fr", "tatoeba":"fra", "flores":"fra_Latn", "udhr":["French", "Francais", "Français"]},
    "es": {"name":"Spanish", "wiki":"es", "tatoeba":"spa", "flores":"spa_Latn", "udhr":["Spanish", "Espanol", "Español"]},
    "de": {"name":"German", "wiki":"de", "tatoeba":"deu", "flores":"deu_Latn", "udhr":["German", "Deutsch"]},
    "ru": {"name":"Russian", "wiki":"ru", "tatoeba":"rus", "flores":"rus_Cyrl", "udhr":["Russian", "Russky", "Russki"]},
    "ar": {"name":"Arabic", "wiki":"ar", "tatoeba":"ara", "flores":"arb_Arab", "udhr":["Arabic"]},
    "he": {"name":"Hebrew", "wiki":"he", "tatoeba":"heb", "flores":"heb_Hebr", "udhr":["Hebrew"]},
    "zh": {"name":"Chinese", "wiki":"zh", "tatoeba":"cmn", "flores":"zho_Hans", "udhr":["Chinese", "Mandarin"]},
    "ja": {"name":"Japanese", "wiki":"ja", "tatoeba":"jpn", "flores":"jpn_Jpan", "udhr":["Japanese"]},
    "tr": {"name":"Turkish", "wiki":"tr", "tatoeba":"tur", "flores":"tur_Latn", "udhr":["Turkish", "Turkce", "Türkçe"]},
    "sw": {"name":"Swahili", "wiki":"sw", "tatoeba":"swh", "flores":"swh_Latn", "udhr":["Swahili", "Kiswahili"]},
    "xh": {"name":"Xhosa", "wiki":"xh", "tatoeba":"xho", "flores":"xho_Latn", "udhr":["Xhosa"]},
    "is": {"name":"Icelandic", "wiki":"is", "tatoeba":"isl", "flores":"isl_Latn", "udhr":["Icelandic", "Islenska"]},
    "la": {"name":"Latin", "wiki":"la", "tatoeba":"lat", "flores":None, "udhr":["Latin", "Latina"]},
    "el": {"name":"Greek", "wiki":"el", "tatoeba":"ell", "flores":"ell_Grek", "udhr":["Greek", "Ellinika"]},
}

NLTK_UDHR_URLS = [
    "https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/udhr.zip",
    "https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/udhr2.zip",
]
TATOEBA_CC0_URL = "https://downloads.tatoeba.org/exports/sentences_CC0.tar.bz2"

GUTENBERG_IDS = {
    # Conservative tiny literary anchors. Users can add more IDs in manifests/gutenberg_ids.json.
    "en": [11, 1342, 1661],       # Alice, Pride and Prejudice, Sherlock Holmes
    "fr": [17489],                # placeholder curated French text if available
    "de": [2229],                 # placeholder curated German text if available
    "es": [2000],                 # Don Quijote (large)
    "it": [25344],                # optional if lang added
}

FLORES_SPLITS = ["dev", "devtest"]


def log(msg: str) -> None:
    print(msg, flush=True)


def ensure(p: Path) -> Path:
    p.mkdir(parents=True, exist_ok=True)
    return p


def write_text(path: Path, text: str) -> None:
    ensure(path.parent)
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: dict) -> None:
    ensure(path.parent)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")


def request_bytes(url: str, timeout: int = 60) -> bytes:
    req = Request(url, headers={"User-Agent": USER_AGENT})
    with urlopen(req, timeout=timeout) as r:
        return r.read()


def download(url: str, dest: Path, timeout: int = 120, force: bool = False) -> Path:
    ensure(dest.parent)
    if dest.exists() and dest.stat().st_size > 0 and not force:
        return dest
    log(f"DOWNLOAD {url}")
    data = request_bytes(url, timeout=timeout)
    dest.write_bytes(data)
    return dest


def clean_text_basic(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    # strip control chars except line/tab
    text = "".join(ch if (ch == "\n" or ch == "\t" or ord(ch) >= 32) else " " for ch in text)
    # trim trailing spaces but preserve paragraph lines
    lines = [ln.strip() for ln in text.split("\n")]
    # collapse excessive blank lines
    out = []
    blank = 0
    for ln in lines:
        if not ln:
            blank += 1
            if blank <= 1:
                out.append("")
        else:
            blank = 0
            out.append(ln)
    return "\n".join(out).strip() + "\n"


def safe_name(s: str) -> str:
    s = re.sub(r"[^A-Za-z0-9._-]+", "_", s)
    return s.strip("_")[:120] or "text"


def fetch_udhr(root: Path, langs: List[str]) -> None:
    cache = ensure(root / "_cache" / "udhr_nltk")
    zips = []
    for url in NLTK_UDHR_URLS:
        dest = cache / url.rsplit("/", 1)[-1]
        try:
            zips.append(download(url, dest))
        except Exception as e:
            log(f"WARN UDHR download failed {url}: {e}")
    entries: List[Tuple[str, bytes, str]] = []
    for zp in zips:
        try:
            with zipfile.ZipFile(zp, "r") as z:
                for name in z.namelist():
                    if name.endswith("/"):
                        continue
                    base = Path(name).name
                    if not base or "." in base and base.rsplit(".",1)[-1].lower() not in ["txt", "latin1", "utf8", "utf-8"]:
                        # NLTK files often have no .txt extension; don't overfilter names without extension
                        pass
                    try:
                        entries.append((base, z.read(name), zp.name))
                    except Exception:
                        continue
        except zipfile.BadZipFile:
            log(f"WARN bad UDHR zip {zp}")
    if not entries:
        log("MISS UDHR no zip entries found")
        return

    for code in langs:
        info = LANGS[code]
        hints = [h.lower() for h in info["udhr"] + [info["name"], code]]
        scored = []
        for base, data, source_zip in entries:
            b = base.lower()
            score = 0
            for h in hints:
                if h and h.lower() in b:
                    score += len(h) + 10
            # prefer UTF8-ish and avoid index/readme files
            if "utf" in b: score += 3
            if "readme" in b or "index" in b: score -= 100
            if score > 0:
                scored.append((score, base, data, source_zip))
        if not scored:
            log(f"MISS UDHR {code} {info['name']}")
            continue
        scored.sort(reverse=True, key=lambda x: x[0])
        _, base, data, source_zip = scored[0]
        text = None
        for enc in ["utf-8", "latin-1", "utf-16"]:
            try:
                text = data.decode(enc)
                break
            except UnicodeDecodeError:
                continue
        if not text:
            log(f"MISS UDHR decode {code} {base}")
            continue
        out_dir = ensure(root / "datasets" / "parallel" / "udhr" / code)
        out = out_dir / f"udhr_{code}.txt"
        write_text(out, clean_text_basic(text))
        write_json(out_dir / "source.json", {
            "dataset": "UDHR",
            "source": "NLTK UDHR corpus packages",
            "source_zip": source_zip,
            "source_file": base,
            "language_code": code,
            "language": info["name"],
            "notes": "Fetched from NLTK data package mirror; verify against UN/OHCHR for final publication if needed."
        })
        log(f"OK UDHR {code} <- {base}")


def fetch_tatoeba_cc0(root: Path, langs: List[str], per_lang: int) -> None:
    cache = ensure(root / "_cache" / "tatoeba")
    archive = cache / "sentences_CC0.tar.bz2"
    try:
        download(TATOEBA_CC0_URL, archive, timeout=300)
    except Exception as e:
        log(f"MISS TATOEBA download: {e}")
        return
    wanted = {LANGS[c]["tatoeba"]: c for c in langs if LANGS[c].get("tatoeba")}
    buckets: Dict[str, List[str]] = {c: [] for c in langs}
    try:
        with tarfile.open(archive, "r:bz2") as tar:
            member = None
            for m in tar.getmembers():
                if m.name.endswith("sentences_CC0.csv"):
                    member = m
                    break
            if member is None:
                log("MISS TATOEBA: sentences_CC0.csv not found in archive")
                return
            f = tar.extractfile(member)
            if f is None:
                log("MISS TATOEBA: cannot extract csv")
                return
            wrapper = io.TextIOWrapper(f, encoding="utf-8", errors="replace", newline="")
            reader = csv.reader(wrapper, delimiter="\t")
            for row in reader:
                if len(row) < 3:
                    continue
                _sid, iso3, sent = row[0], row[1], row[2]
                code = wanted.get(iso3)
                if code and len(buckets[code]) < per_lang:
                    sent = sent.strip()
                    if sent:
                        buckets[code].append(sent)
                if all(len(v) >= per_lang for v in buckets.values() if v is not None):
                    # Keep reading not needed once all requested buckets filled
                    pass
    except Exception as e:
        log(f"MISS TATOEBA parse: {e}")
        return

    for code, lines in buckets.items():
        if not lines:
            log(f"MISS TATOEBA {code} {LANGS[code]['name']}")
            continue
        out_dir = ensure(root / "datasets" / "native" / "tatoeba_cc0" / code)
        out = out_dir / f"tatoeba_cc0_{code}.txt"
        write_text(out, clean_text_basic("\n".join(lines)))
        write_json(out_dir / "source.json", {
            "dataset": "Tatoeba sentences_CC0",
            "source": "Tatoeba downloads sentences_CC0.tar.bz2",
            "language_code": code,
            "tatoeba_iso3": LANGS[code]["tatoeba"],
            "language": LANGS[code]["name"],
            "sentences_saved": len(lines),
            "license_note": "CC0 subset as distributed by Tatoeba."
        })
        log(f"OK TATOEBA {code} {len(lines)} sentences")


def fetch_wikipedia(root: Path, langs: List[str], pages_per_lang: int) -> None:
    for code in langs:
        wiki = LANGS[code].get("wiki")
        if not wiki:
            continue
        pages: List[str] = []
        titles: List[str] = []
        attempts = 0
        while len(pages) < pages_per_lang and attempts < max(10, pages_per_lang * 4):
            attempts += 1
            limit = min(10, pages_per_lang - len(pages))
            params = {
                "action": "query",
                "format": "json",
                "generator": "random",
                "grnnamespace": "0",
                "grnlimit": str(limit),
                "prop": "extracts",
                "explaintext": "1",
                "exsectionformat": "plain",
            }
            url = f"https://{wiki}.wikipedia.org/w/api.php?" + urlencode(params)
            try:
                data = json.loads(request_bytes(url, timeout=60).decode("utf-8"))
                qpages = data.get("query", {}).get("pages", {})
                for _, p in qpages.items():
                    title = p.get("title", "")
                    extract = p.get("extract", "").strip()
                    # Filter tiny stubs/disambiguations.
                    if len(extract) >= 800 and "may refer to" not in extract[:300].lower():
                        titles.append(title)
                        pages.append(f"# {title}\n\n{extract}")
                        if len(pages) >= pages_per_lang:
                            break
                time.sleep(0.25)
            except Exception as e:
                log(f"WARN WIKI {code} attempt {attempts}: {e}")
                time.sleep(1.0)
        if not pages:
            log(f"MISS WIKI {code} {LANGS[code]['name']}")
            continue
        out_dir = ensure(root / "datasets" / "native" / "wikipedia_api" / code)
        out = out_dir / f"wikipedia_random_{code}.txt"
        write_text(out, clean_text_basic("\n\n---\n\n".join(pages)))
        write_json(out_dir / "source.json", {
            "dataset": "Wikipedia random article extracts via MediaWiki API",
            "language_code": code,
            "wiki_code": wiki,
            "language": LANGS[code]["name"],
            "pages_saved": len(pages),
            "titles": titles,
            "license_note": "Wikipedia text is generally CC BY-SA; preserve attribution/share-alike requirements for redistribution."
        })
        log(f"OK WIKI {code} {len(pages)} pages")


def fetch_flores(root: Path, langs: List[str]) -> None:
    # Best handled by HuggingFace datasets because the original repository points to hosted data.
    # We don't hard fail if unavailable.
    try:
        import datasets  # type: ignore
    except Exception:
        log("MISS FLORES: Python package 'datasets' not installed. Try: py -3 -m pip install datasets")
        return
    try:
        # Many current installs expose facebook/flores with configs per language pair or 'all'.
        # We try facebook/flores first, then Muennighoff/flores200.
        ds = None
        last = None
        for name in ["facebook/flores", "Muennighoff/flores200"]:
            try:
                ds = datasets.load_dataset(name, "all")
                log(f"OK FLORES loaded {name}: all")
                break
            except Exception as e:
                last = e
        if ds is None:
            log(f"MISS FLORES load: {last}")
            return
        wanted = {c: LANGS[c].get("flores") for c in langs if LANGS[c].get("flores")}
        for split in [s for s in FLORES_SPLITS if s in ds]:
            table = ds[split]
            # Depending on builder version, columns may be language codes directly.
            cols = set(table.column_names)
            for code, flores_code in wanted.items():
                if not flores_code or flores_code not in cols:
                    log(f"MISS FLORES {code} column {flores_code}")
                    continue
                lines = [str(x).strip() for x in table[flores_code] if str(x).strip()]
                out_dir = ensure(root / "datasets" / "parallel" / "flores200" / split / code)
                out = out_dir / f"flores200_{split}_{code}.txt"
                write_text(out, clean_text_basic("\n".join(lines)))
                write_json(out_dir / "source.json", {
                    "dataset": "FLORES-200",
                    "split": split,
                    "language_code": code,
                    "flores_code": flores_code,
                    "language": LANGS[code]["name"],
                    "license_note": "FLORES-200 is distributed as CC BY-SA 4.0 in the official repository metadata; verify exact source package for publication."
                })
                log(f"OK FLORES {split} {code} {len(lines)} lines")
    except Exception as e:
        log(f"MISS FLORES unexpected: {e}")


def fetch_gutenberg(root: Path, langs: List[str]) -> None:
    manifest = root / "manifests" / "gutenberg_ids.json"
    ids = dict(GUTENBERG_IDS)
    if manifest.exists():
        try:
            custom = json.loads(manifest.read_text(encoding="utf-8"))
            for k, v in custom.items():
                ids[k] = v
        except Exception as e:
            log(f"WARN Gutenberg manifest parse failed: {e}")
    for code in langs:
        for gid in ids.get(code, []):
            url_candidates = [
                f"https://www.gutenberg.org/cache/epub/{gid}/pg{gid}.txt",
                f"https://www.gutenberg.org/files/{gid}/{gid}-0.txt",
                f"https://www.gutenberg.org/files/{gid}/{gid}.txt",
            ]
            text = None
            src = None
            for url in url_candidates:
                try:
                    text = request_bytes(url, timeout=60).decode("utf-8", errors="replace")
                    if len(text) > 1000:
                        src = url
                        break
                except Exception:
                    continue
            if not text or not src:
                log(f"MISS GUTENBERG {code} {gid}")
                continue
            out_dir = ensure(root / "datasets" / "native" / "gutenberg" / code)
            out = out_dir / f"gutenberg_{code}_{gid}.txt"
            write_text(out, clean_text_basic(text))
            write_json(out.with_suffix(".source.json"), {
                "dataset": "Project Gutenberg",
                "gutenberg_id": gid,
                "url": src,
                "language_code": code,
                "license_note": "Project Gutenberg texts are typically public domain in the US; verify local jurisdiction and Gutenberg license terms before redistribution."
            })
            log(f"OK GUTENBERG {code} {gid}")


def generate_controls(root: Path) -> None:
    controls = ensure(root / "datasets" / "controls")
    rng = random.Random(18427)
    bit_text = "".join("1" if rng.random() < 0.5 else "0" for _ in range(100000))
    write_text(controls / "random_bits_100k.txt", bit_text)
    letters = "abcdefghijklmnopqrstuvwxyz     .,;:\n"
    iid = "".join(rng.choice(letters) for _ in range(100000))
    write_text(controls / "iid_ascii_100k.txt", iid)
    periodic = ("abcde " * 20000)[:100000]
    write_text(controls / "periodic_ascii_100k.txt", periodic)
    code = "\n".join([
        "fn main() {",
        "    let mut total = 0u64;",
        "    for i in 0..10000 { total += i * 3 + 7; }",
        "    println!(\"{}\", total);",
        "}",
    ] * 2000)
    write_text(controls / "rust_code_repeated_100k.rs", code[:100000])
    write_json(controls / "source.json", {"dataset":"synthetic controls", "seed":18427})
    log("OK CONTROLS synthetic controls written")


def parse_args(argv: Optional[List[str]] = None) -> argparse.Namespace:
    ap = argparse.ArgumentParser(description="Fetch broad RRLANG corpus pack")
    ap.add_argument("--root", default=".")
    ap.add_argument("--preset", default="Core", choices=["Core","Broad","UDHR","Wiki","Tatoeba","Flores","Controls","Gutenberg"])
    ap.add_argument("--langs", default="")
    ap.add_argument("--wiki-pages-per-lang", type=int, default=20)
    ap.add_argument("--tatoeba-per-lang", type=int, default=10000)
    ap.add_argument("--skip-flores", action="store_true")
    ap.add_argument("--skip-tatoeba", action="store_true")
    ap.add_argument("--skip-wiki", action="store_true")
    ap.add_argument("--skip-udhr", action="store_true")
    ap.add_argument("--skip-controls", action="store_true")
    return ap.parse_args(argv)


def main(argv: Optional[List[str]] = None) -> int:
    args = parse_args(argv)
    root = Path(args.root).resolve()
    ensure(root / "logs")
    ensure(root / "datasets")
    if args.langs.strip():
        langs = [x.strip() for x in re.split(r"[,\s]+", args.langs) if x.strip()]
    else:
        langs = ["en","cy","fr","es","de","ru","ar","he","zh","ja","tr","sw","xh","is","la","el"]
    langs = [c for c in langs if c in LANGS]
    log(f"Root: {root}")
    log(f"Languages: {', '.join(langs)}")

    preset = args.preset.lower()
    do_udhr = preset in ["core","broad","udhr"] and not args.skip_udhr
    do_wiki = preset in ["core","broad","wiki"] and not args.skip_wiki
    do_tatoeba = preset in ["core","broad","tatoeba"] and not args.skip_tatoeba
    do_flores = preset in ["broad","flores"] and not args.skip_flores
    do_gutenberg = preset in ["broad","gutenberg"]
    do_controls = preset in ["core","broad","controls"] and not args.skip_controls

    if do_udhr: fetch_udhr(root, langs)
    if do_tatoeba: fetch_tatoeba_cc0(root, langs, args.tatoeba_per_lang)
    if do_wiki: fetch_wikipedia(root, langs, args.wiki_pages_per_lang)
    if do_flores: fetch_flores(root, langs)
    if do_gutenberg: fetch_gutenberg(root, langs)
    if do_controls: generate_controls(root)

    log("DONE")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
