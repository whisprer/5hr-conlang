#!/usr/bin/env python3
"""
build_final_comparison_table.py

Build final comparison tables from RRLANG summary CSVs.

Typical use from the rrlang repo root:

  py -3 .\scripts\build_final_comparison_table.py ^
    --summary .\outputs\summary_canonical_udhr_v0_3_nulls100_labeled ^
    --out .\outputs\final_tables\canonical_udhr_v0_3

Outputs:
  final_comparison_table.csv
  final_comparison_table.md
  trusted_markov2_survivors.csv
  metric_family_summary.csv
"""

from __future__ import annotations

import argparse
import csv
import math
import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

NOISE_ENCODINGS = {"utf8_bits"}
NOISE_EVENTS = {"other", "digit", "whitespace"}
NOISE_METRICS = {"zeta_spectral_coherence"}


def read_csv(path: Path) -> List[Dict[str, str]]:
    if not path.exists():
        raise SystemExit(f"Missing required CSV: {path}")
    with path.open("r", encoding="utf-8-sig", newline="") as handle:
        return list(csv.DictReader(handle))


def write_csv(path: Path, rows: List[Dict[str, Any]], fieldnames: List[str] | None = None) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if fieldnames is None:
        fieldnames = []
        seen = set()
        for row in rows:
            for key in row.keys():
                if key not in seen:
                    seen.add(key)
                    fieldnames.append(key)
    with path.open("w", encoding="utf-8-sig", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)


def to_float(value: Any) -> Optional[float]:
    if value is None:
        return None
    text = str(value).strip()
    if not text:
        return None
    try:
        out = float(text)
    except ValueError:
        return None
    return out if math.isfinite(out) else None


def infer_language(row: Dict[str, str]) -> str:
    direct = (row.get("language_guess") or "").strip()
    if direct and direct != "unknown":
        return direct

    hay = " ".join([
        row.get("input", ""),
        row.get("report_file", ""),
    ]).replace("\\", "/").lower()

    # v0.3 canonical report names: ar_udhr_ar_canonical.json
    m = re.search(r"(?:^|/)([a-z]{2,4})_udhr_\1_canonical\.json", hay)
    if m:
        return m.group(1)

    # canonical input paths: .../udhr/ar/udhr_ar_canonical.txt
    m = re.search(r"/udhr/([a-z]{2,4})/", hay)
    if m:
        return m.group(1)

    # flattened names from older batch output
    m = re.search(r"datasets_parallel_udhr_([a-z]{2,4})_", hay)
    if m:
        return m.group(1)

    if "mesu" in hay:
        return "mesu"

    return "unknown"


def infer_family(row: Dict[str, str]) -> str:
    direct = (row.get("corpus_family") or "").strip()
    if direct and direct != "unknown":
        return direct

    hay = " ".join([row.get("input", ""), row.get("report_file", "")]).replace("\\", "/").lower()
    if "udhr" in hay:
        return "parallel_udhr"
    if "private_parallel" in hay:
        return "private_parallel"
    if "controls" in hay:
        return "controls"
    if "tatoeba" in hay:
        return "native_tatoeba_cc0"
    if "wikipedia" in hay:
        return "native_wikipedia"
    return "unknown"


def is_trusted(row: Dict[str, str], exclude_punctuation: bool) -> bool:
    if (row.get("null_model") or "").strip() != "markov_2":
        return False
    if (row.get("encoding") or "").strip() in NOISE_ENCODINGS:
        return False
    event = (row.get("event") or "").strip()
    metric = (row.get("metric") or "").strip()
    if event in NOISE_EVENTS:
        return False
    if metric in NOISE_METRICS:
        return False
    if exclude_punctuation and event == "punctuation":
        return False
    z = to_float(row.get("z"))
    p = to_float(row.get("p_emp"))
    if z is None:
        return False
    if abs(z) < 3.0:
        return False
    if p is not None and p > 0.05:
        return False
    return True


def signal_family(row: Dict[str, str]) -> str:
    event = (row.get("event") or "").strip()
    encoding = (row.get("encoding") or "").strip()
    metric = (row.get("metric") or "").strip()
    if event == "word_boundary" or encoding == "word_boundary":
        return "word_boundary"
    if event == "punctuation":
        return "punctuation"
    if event in {"vowel", "consonant"}:
        return "vowel_consonant"
    if event in {"hapax", "low_frequency", "mid_frequency", "high_frequency"}:
        return "frequency_class"
    if metric in {"gap_entropy", "run_entropy"}:
        return "rhythm_spacing"
    if metric == "prime_gap_affinity":
        return "prime_gap_affinity"
    if metric == "critical_line_symmetry":
        return "critical_line_symmetry"
    return "other_linguistic"


def direction(z: Optional[float]) -> str:
    if z is None:
        return ""
    if z < 0:
        return "lower_than_null"
    if z > 0:
        return "higher_than_null"
    return "same_as_null"


def profile(count: int, max_abs_z: float) -> str:
    if count == 0:
        return "quiet"
    if count <= 2 and max_abs_z < 5:
        return "weak"
    if count <= 5 and max_abs_z < 8:
        return "moderate"
    return "strong"


def write_markdown(path: Path, rows: List[Dict[str, Any]]) -> None:
    cols = [
        "rank", "language_guess", "trusted_survivor_count", "strongest_abs_z",
        "strongest_event", "strongest_metric", "profile", "notes"
    ]
    lines = [
        "# RRLANG Final Comparison Table",
        "",
        "Filtered Markov-2 survivor comparison.",
        "",
        "| " + " | ".join(cols) + " |",
        "| " + " | ".join(["---"] * len(cols)) + " |",
    ]
    for row in rows:
        cells = []
        for col in cols:
            value = str(row.get(col, "")).replace("|", "\\|").replace("\n", " ")
            cells.append(value)
        lines.append("| " + " | ".join(cells) + " |")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="Build RRLANG final comparison tables.")
    ap.add_argument("--summary", required=True, help="Folder containing report_index.csv and markov2_survivors.csv")
    ap.add_argument("--out", required=True, help="Output folder")
    ap.add_argument("--exclude-punctuation", action="store_true", help="Exclude punctuation events from trusted table")
    args = ap.parse_args()

    summary = Path(args.summary)
    out_dir = Path(args.out)

    report_rows = read_csv(summary / "report_index.csv")
    survivor_rows = read_csv(summary / "markov2_survivors.csv")

    languages = sorted({infer_language(r) for r in report_rows})
    if "unknown" in languages and len(languages) > 1:
        languages = [x for x in languages if x != "unknown"] + ["unknown"]

    trusted: List[Dict[str, Any]] = []
    for row in survivor_rows:
        if is_trusted(row, args.exclude_punctuation):
            z = to_float(row.get("z"))
            out = dict(row)
            out["language_guess"] = infer_language(row)
            out["corpus_family"] = infer_family(row)
            out["abs_z"] = abs(z) if z is not None else ""
            out["direction"] = direction(z)
            out["signal_family"] = signal_family(row)
            trusted.append(out)

    by_lang: Dict[str, List[Dict[str, Any]]] = {lang: [] for lang in languages}
    for row in trusted:
        by_lang.setdefault(row["language_guess"], []).append(row)

    comparison: List[Dict[str, Any]] = []
    for lang in sorted(by_lang.keys() | set(languages)):
        lang_reports = [r for r in report_rows if infer_language(r) == lang]
        lang_rows = by_lang.get(lang, [])
        strongest = None
        if lang_rows:
            strongest = max(lang_rows, key=lambda r: float(r.get("abs_z") or 0.0))
        count = len(lang_rows)
        max_abs_z = float(strongest.get("abs_z")) if strongest else 0.0

        fam_counts: Dict[str, int] = {}
        for row in lang_rows:
            fam = str(row.get("signal_family", "unknown"))
            fam_counts[fam] = fam_counts.get(fam, 0) + 1
        dominant_family = ""
        if fam_counts:
            dominant_family = sorted(fam_counts.items(), key=lambda kv: (-kv[1], kv[0]))[0][0]

        cleaned_values = []
        for report in lang_reports:
            val = to_float(report.get("cleaned_chars"))
            if val is not None:
                cleaned_values.append(int(val))

        notes = []
        if count == 0:
            notes.append("no filtered Markov-2 survivors")
        elif dominant_family:
            notes.append(f"dominant signal: {dominant_family}")
        if strongest and strongest.get("event") == "word_boundary":
            notes.append("strongest row is word-boundary based")
        if strongest and strongest.get("event") == "punctuation":
            notes.append("strongest row is punctuation based")

        comparison.append({
            "language_guess": lang,
            "corpus_family": ";".join(sorted({infer_family(r) for r in lang_reports})) if lang_reports else "unknown",
            "report_count": len(lang_reports),
            "cleaned_chars": max(cleaned_values) if cleaned_values else "",
            "trusted_survivor_count": count,
            "strongest_abs_z": round(max_abs_z, 6),
            "strongest_z": strongest.get("z", "") if strongest else "",
            "strongest_direction": strongest.get("direction", "") if strongest else "",
            "strongest_encoding": strongest.get("encoding", "") if strongest else "",
            "strongest_event": strongest.get("event", "") if strongest else "",
            "strongest_metric": strongest.get("metric", "") if strongest else "",
            "strongest_p_emp": strongest.get("p_emp", "") if strongest else "",
            "dominant_signal_family": dominant_family,
            "profile": profile(count, max_abs_z),
            "notes": "; ".join(notes),
            "strongest_input": strongest.get("input", "") if strongest else "",
        })

    comparison.sort(key=lambda r: (-int(r["trusted_survivor_count"]), -float(r["strongest_abs_z"]), str(r["language_guess"])))
    for i, row in enumerate(comparison, start=1):
        row["rank"] = i

    comparison_fields = [
        "rank", "language_guess", "corpus_family", "report_count", "cleaned_chars",
        "trusted_survivor_count", "strongest_abs_z", "strongest_z", "strongest_direction",
        "strongest_encoding", "strongest_event", "strongest_metric", "strongest_p_emp",
        "dominant_signal_family", "profile", "notes", "strongest_input"
    ]
    trusted_fields = [
        "language_guess", "corpus_family", "encoding", "event", "metric", "signal_family",
        "z", "abs_z", "direction", "p_emp", "input", "report_file"
    ]

    fam_summary: Dict[Tuple[str, str], Dict[str, Any]] = {}
    for row in trusted:
        key = (row["language_guess"], row["signal_family"])
        z = to_float(row.get("z")) or 0.0
        if key not in fam_summary:
            fam_summary[key] = {
                "language_guess": row["language_guess"],
                "signal_family": row["signal_family"],
                "count": 0,
                "strongest_abs_z": 0.0,
                "strongest_event": "",
                "strongest_metric": "",
                "strongest_z": "",
            }
        dest = fam_summary[key]
        dest["count"] += 1
        if abs(z) > float(dest["strongest_abs_z"]):
            dest["strongest_abs_z"] = round(abs(z), 6)
            dest["strongest_event"] = row.get("event", "")
            dest["strongest_metric"] = row.get("metric", "")
            dest["strongest_z"] = row.get("z", "")

    fam_rows = sorted(fam_summary.values(), key=lambda r: (r["language_guess"], -int(r["count"]), r["signal_family"]))

    write_csv(out_dir / "final_comparison_table.csv", comparison, comparison_fields)
    write_markdown(out_dir / "final_comparison_table.md", comparison)
    write_csv(out_dir / "trusted_markov2_survivors.csv", trusted, trusted_fields)
    write_csv(out_dir / "metric_family_summary.csv", fam_rows)
    write_csv(out_dir / "strongest_by_language.csv", comparison, comparison_fields)

    print("RRLANG FINAL COMPARISON TABLE")
    print("=============================")
    print(f"Summary input:      {summary}")
    print(f"Reports indexed:    {len(report_rows)}")
    print(f"Survivor rows read: {len(survivor_rows)}")
    print(f"Trusted rows kept:  {len(trusted)}")
    print(f"Languages compared: {len(comparison)}")
    print(f"Output folder:      {out_dir}")
    print("")
    print("Top rows:")
    for row in comparison[:10]:
        print(f"  {row['rank']:>2}. {row['language_guess']:<5} count={row['trusted_survivor_count']:<3} max|z|={row['strongest_abs_z']:<9} {row['strongest_event']}/{row['strongest_metric']} [{row['profile']}]")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
