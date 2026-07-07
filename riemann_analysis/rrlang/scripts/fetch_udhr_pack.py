#!/usr/bin/env python3
"""
Fetch UDHR target documents for RRLANG.

This script tries sources in this order:
1) OHCHR official translation pages, when a target slug is known.
2) EFELE / UDHR-in-XML individual plain-text files.
3) EFELE bulk plain-text zip, if direct files fail.

It uses only Python standard library.
"""
from __future__ import annotations

import argparse, html, json, re, sys, time, zipfile
from dataclasses import dataclass, asdict
from html.parser import HTMLParser
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen
from urllib.parse import urljoin

BASE_OHCHR = "https://www.ohchr.org/en/human-rights/universal-declaration/translations/"
BASE_EFELE = "https://efele.net/udhr/d/"
BULK_EFELE = "https://efele.net/udhr/assemblies/udhr_txt.zip"

TARGETS = [
    ("en", "English", "english", ["eng"]),
    ("cy", "Welsh", "welsh-cymraeg", ["cym", "wel"]),
    ("fr", "French", "french", ["fra", "fre"]),
    ("es", "Spanish", "spanish", ["spa", "spn"]),
    ("de", "German", "german-deutsch", ["deu", "ger"]),
    ("ru", "Russian", "russian", ["rus"]),
    ("ar", "Arabic", "arabic", ["arb", "ara"]),
    ("he", "Hebrew", "hebrew-ivrit", ["heb"]),
    ("zh", "Chinese", "chinese-mandarin", ["cmn_hans", "zho", "chi", "cmn"]),
    ("ja", "Japanese", "japanese-nihongo", ["jpn"]),
    ("tr", "Turkish", "turkish-turkce", ["tur"]),
    ("sw", "Swahili", "swahili-kiswahili", ["swh", "swa"]),
    ("xh", "Xhosa", "xhosa", ["xho"]),
    ("is", "Icelandic", "icelandic-islenska", ["isl", "ice"]),
    ("la", "Latin", "latin", ["lat_1", "lat"]),
    ("el", "Greek", "greek-ellinika", ["ell", "gre"]),
]

class TextExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.skip = 0
        self.parts = []
    def handle_starttag(self, tag, attrs):
        if tag in {"script", "style", "nav", "footer", "header", "noscript"}:
            self.skip += 1
        if tag in {"p", "div", "li", "h1", "h2", "h3", "h4", "br"}:
            self.parts.append("\n")
    def handle_endtag(self, tag):
        if tag in {"script", "style", "nav", "footer", "header", "noscript"} and self.skip:
            self.skip -= 1
        if tag in {"p", "div", "li", "h1", "h2", "h3", "h4"}:
            self.parts.append("\n")
    def handle_data(self, data):
        if not self.skip:
            s = data.strip()
            if s:
                self.parts.append(s + " ")

def http_get(url: str, timeout: int = 30) -> bytes:
    req = Request(url, headers={"User-Agent": "rrlang-dataset-fetcher/0.1"})
    with urlopen(req, timeout=timeout) as r:
        return r.read()

def clean_lines(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    text = html.unescape(text)
    lines = []
    for line in text.splitlines():
        line = re.sub(r"[ \t]+", " ", line).strip()
        if not line:
            continue
        low = line.lower()
        # Remove common boilerplate/header-ish lines from EFELE/OHCHR extraction.
        if low.startswith("universal declaration of human rights -"):
            continue
        if "plain text version prepared" in low:
            continue
        if low in {"share", "download", "listen", "official language", "preamble"}:
            continue
        if line.startswith("-----"):
            continue
        lines.append(line)
    return "\n".join(lines).strip() + "\n"

def html_to_text(raw: bytes) -> str:
    parser = TextExtractor()
    parser.feed(raw.decode("utf-8", errors="replace"))
    txt = "".join(parser.parts)
    # Start near UDHR title if present; otherwise leave whole extracted body.
    markers = [
        "Universal Declaration of Human Rights",
        "Déclaration universelle des droits de l'homme",
        "Declaración Universal de Derechos Humanos",
        "Die Allgemeine Erklärung der Menschenrechte",
    ]
    for m in markers:
        idx = txt.find(m)
        if idx >= 0:
            txt = txt[idx:]
            break
    return clean_lines(txt)

def try_ohchr(slug: str) -> tuple[str, str] | None:
    if not slug:
        return None
    url = BASE_OHCHR + slug
    try:
        raw = http_get(url)
        txt = html_to_text(raw)
        if len(txt) > 1000:
            return txt, url
    except Exception as e:
        return None
    return None

def try_efele(candidates: list[str]) -> tuple[str, str] | None:
    for c in candidates:
        url = f"{BASE_EFELE}udhr_{c}.txt"
        try:
            raw = http_get(url)
            txt = clean_lines(raw.decode("utf-8", errors="replace"))
            if len(txt) > 1000:
                return txt, url
        except Exception:
            continue
    return None

def fetch_bulk_zip(cache_dir: Path) -> Path | None:
    cache_dir.mkdir(parents=True, exist_ok=True)
    zpath = cache_dir / "udhr_txt.zip"
    if zpath.exists() and zpath.stat().st_size > 10000:
        return zpath
    try:
        raw = http_get(BULK_EFELE, timeout=120)
        zpath.write_bytes(raw)
        return zpath
    except Exception:
        return None

def try_bulk(cache_dir: Path, candidates: list[str]) -> tuple[str, str] | None:
    zpath = fetch_bulk_zip(cache_dir)
    if not zpath:
        return None
    try:
        with zipfile.ZipFile(zpath) as z:
            names = z.namelist()
            for c in candidates:
                # Try basename match anywhere in zip.
                wanted = f"udhr_{c}.txt"
                for name in names:
                    if name.endswith(wanted):
                        raw = z.read(name)
                        txt = clean_lines(raw.decode("utf-8", errors="replace"))
                        if len(txt) > 1000:
                            return txt, f"{BULK_EFELE}::{name}"
    except Exception:
        return None
    return None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="Root of rrlang document pack")
    ap.add_argument("--only", default="", help="Comma-separated language codes to fetch")
    ap.add_argument("--prefer", default="ohchr,efele,bulk", help="Source order: ohchr,efele,bulk")
    args = ap.parse_args()
    root = Path(args.root).resolve()
    outbase = root / "datasets" / "parallel" / "udhr" / "target_languages"
    outbase.mkdir(parents=True, exist_ok=True)
    cache_dir = root / "_cache"
    only = {x.strip() for x in args.only.split(",") if x.strip()}
    order = [x.strip() for x in args.prefer.split(",") if x.strip()]
    logs = []
    for code, name, slug, candidates in TARGETS:
        if only and code not in only:
            continue
        dest_dir = outbase / code
        dest_dir.mkdir(parents=True, exist_ok=True)
        text = source = None
        for src in order:
            got = None
            if src == "ohchr": got = try_ohchr(slug)
            elif src == "efele": got = try_efele(candidates)
            elif src == "bulk": got = try_bulk(cache_dir, candidates)
            if got:
                text, source = got
                break
        status = "ok" if text else "failed"
        if text:
            (dest_dir / f"udhr_{code}.txt").write_text(text, encoding="utf-8")
            (dest_dir / "source.json").write_text(json.dumps({
                "language_code": code,
                "language": name,
                "source": source,
                "chars": len(text),
            }, ensure_ascii=False, indent=2), encoding="utf-8")
            print(f"OK {code:>3} {name:<12} chars={len(text)} source={source}")
        else:
            print(f"MISS {code:>3} {name:<12}")
        logs.append({"code": code, "language": name, "status": status, "source": source})
        time.sleep(0.2)
    (root / "logs" / "fetch_udhr_log.json").write_text(json.dumps(logs, ensure_ascii=False, indent=2), encoding="utf-8")

if __name__ == "__main__":
    main()
