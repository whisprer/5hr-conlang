from pathlib import Path
import textwrap, zipfile, json

base = Path("/mnt/data/rrlang_summary_tools_v0_2")
base.mkdir(parents=True, exist_ok=True)

script = r'''#!/usr/bin/env python3
"""
RRLANG report summariser v0.2

Reads RRLANG JSON reports and writes useful CSV summaries.

Outputs:
  report_index.csv
  metrics_flat.csv
  markov2_survivors.csv
  alert_events.csv
  alert_summary.csv
  top_deviations.csv

Usage from the rrlang repo root:

  py -3 scripts\summarise_rrlang_reports_v2.py ^
    --input outputs\corpus_batch_curated ^
    --out outputs\summary_curated
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import re
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


def finite_float(value: Any) -> Optional[float]:
    """Convert common JSON/string values to a finite float, or None."""
    if value is None or isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        value = float(value)
        return value if math.isfinite(value) else None
    if isinstance(value, str):
        value = value.strip().strip(",")
        try:
            out = float(value)
            return out if math.isfinite(out) else None
        except ValueError:
            return None
    return None


def first_of(obj: Dict[str, Any], names: Iterable[str], default: Any = "") -> Any:
    """Return the first present key from a dictionary."""
    for name in names:
        if name in obj:
            return obj[name]
    return default


def read_json(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8-sig") as handle:
        return json.load(handle)


def classify_report(report_path: Path, report: Dict[str, Any]) -> Dict[str, str]:
    """Guess corpus family/source/language from the input path and output name."""
    input_text = str(first_of(report, ["input", "input_path", "input_file"], ""))
    haystack = (input_text + " " + str(report_path)).replace("\\", "/").lower()

    corpus_family = "unknown"
    source = "unknown"
    language = str(first_of(report, ["language", "language_reported"], "")).strip() or "unknown"

    patterns = [
        ("/parallel/udhr/", "parallel_udhr", "udhr"),
        ("datasets_parallel_udhr_", "parallel_udhr", "udhr"),
        ("/native/tatoeba_cc0/", "native_tatoeba_cc0", "tatoeba_cc0"),
        ("datasets_native_tatoeba_cc0_", "native_tatoeba_cc0", "tatoeba_cc0"),
        ("/native/wikipedia_api/", "native_wikipedia", "wikipedia_api"),
        ("datasets_native_wikipedia_api_", "native_wikipedia", "wikipedia_api"),
        ("/private_parallel/", "private_parallel", "private_parallel"),
        ("datasets_private_parallel_", "private_parallel", "private_parallel"),
        ("/controls/", "controls", "controls"),
        ("datasets_controls_", "controls", "controls"),
    ]

    for marker, family, src in patterns:
        if marker in haystack:
            corpus_family = family
            source = src
            break

    # Extract language code from path layouts.
    slash_markers = [
        "/parallel/udhr/",
        "/native/tatoeba_cc0/",
        "/native/wikipedia_api/",
    ]
    for marker in slash_markers:
        if marker in haystack:
            tail = haystack.split(marker, 1)[1]
            candidate = tail.split("/", 1)[0].strip("_- ")
            if candidate:
                language = candidate
            break

    # Extract language code from flattened output names.
    flat_patterns = [
        r"datasets_parallel_udhr_([a-z]{2,3})_",
        r"datasets_native_tatoeba_cc0_([a-z]{2,3})_",
        r"datasets_native_wikipedia_api_([a-z]{2,3})_",
    ]
    for pattern in flat_patterns:
        match = re.search(pattern, haystack)
        if match:
            language = match.group(1)
            break

    if "mesu" in haystack:
        language = "mesu"

    return {
        "corpus_family": corpus_family,
        "source": source,
        "language_guess": language,
    }


def iter_encoding_reports(report: Dict[str, Any]) -> Iterable[Tuple[str, Dict[str, Any]]]:
    encodings = first_of(report, ["encodings", "encoding_reports", "encoding_results"], [])

    if isinstance(encodings, dict):
        for name, value in encodings.items():
            if isinstance(value, dict):
                yield str(name), value
        return

    if isinstance(encodings, list):
        for value in encodings:
            if isinstance(value, dict):
                name = str(first_of(value, ["encoding", "name", "encoding_name"], ""))
                yield name, value


def iter_event_reports(encoding: Dict[str, Any]) -> Iterable[Tuple[str, Dict[str, Any]]]:
    events = first_of(encoding, ["events", "event_reports", "event_results"], [])

    if isinstance(events, dict):
        for name, value in events.items():
            if isinstance(value, dict):
                yield str(name), value
        return

    if isinstance(events, list):
        for value in events:
            if isinstance(value, dict):
                name = str(first_of(value, ["event", "name", "event_name"], ""))
                yield name, value


def parse_observed_metric(item: Any) -> Optional[Dict[str, Any]]:
    """Parse observed metric records from dict/list/string forms."""
    if isinstance(item, dict):
        name = str(first_of(item, ["metric", "name", "key", "id"], ""))
        observed = finite_float(first_of(item, ["observed", "value", "score"], None))
        if name:
            return {
                "metric": name,
                "observed": observed,
                "raw": item,
            }

    if isinstance(item, (list, tuple)) and len(item) >= 2:
        return {
            "metric": str(item[0]),
            "observed": finite_float(item[1]),
            "raw": item,
        }

    if isinstance(item, str):
        # Example: "event_density = 0.123 (description)"
        if "=" in item:
            left, right = item.split("=", 1)
            token = right.strip().split()[0].strip(",")
            return {
                "metric": left.strip(),
                "observed": finite_float(token),
                "raw": item,
            }

    return None


def iter_observed_metrics(event: Dict[str, Any]) -> Iterable[Dict[str, Any]]:
    observed = event.get("observed", [])

    if isinstance(observed, dict):
        for name, value in observed.items():
            yield {
                "metric": str(name),
                "observed": finite_float(value),
                "raw": {name: value},
            }
        return

    if isinstance(observed, list):
        for item in observed:
            parsed = parse_observed_metric(item)
            if parsed:
                yield parsed


def parse_null_metric(item: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    metric = str(first_of(item, ["metric", "name", "metric_name"], ""))
    null_model = str(first_of(item, ["null_model", "null", "model"], ""))

    # Some reports may encode metric as "gap_entropy [markov_2]".
    if not null_model and "[" in metric and "]" in metric:
        before = metric.split("[", 1)[0].strip()
        inside = metric.split("[", 1)[1].split("]", 1)[0].strip()
        metric = before
        null_model = inside

    if not metric and not null_model:
        return None

    return {
        "metric": metric,
        "null_model": null_model,
        "observed": finite_float(first_of(item, ["observed", "value"], None)),
        "null_mean": finite_float(first_of(item, ["null_mean", "mean", "expected"], None)),
        "null_std": finite_float(first_of(item, ["null_std", "std", "stdev"], None)),
        "z": finite_float(first_of(item, ["z", "z_score", "zscore"], None)),
        "p_emp": finite_float(first_of(item, ["p_emp", "p", "empirical_p", "p_value"], None)),
        "raw": item,
    }


def parse_null_metric_string(text: str) -> Optional[Dict[str, Any]]:
    """
    Parse lines like:
      gap_entropy [markov_2]: observed=1.23, null_mean=1.1, null_std=0.2, z=3.1, p_emp=0.009
    """
    if ":" not in text:
        return None

    head, body = text.split(":", 1)
    metric = head.strip()
    null_model = ""

    if "[" in metric and "]" in metric:
        null_model = metric.split("[", 1)[1].split("]", 1)[0].strip()
        metric = metric.split("[", 1)[0].strip()

    values: Dict[str, Any] = {}
    for key in ["observed", "null_mean", "null_std", "z", "p_emp"]:
        match = re.search(rf"{key}\s*=\s*([-+0-9.eE]+)", body)
        if match:
            values[key] = finite_float(match.group(1))

    if not metric:
        return None

    return {
        "metric": metric,
        "null_model": null_model,
        "observed": values.get("observed"),
        "null_mean": values.get("null_mean"),
        "null_std": values.get("null_std"),
        "z": values.get("z"),
        "p_emp": values.get("p_emp"),
        "raw": text,
    }


def iter_null_metrics(event: Dict[str, Any]) -> Iterable[Dict[str, Any]]:
    for key in ["null_adjusted_metrics", "null_metrics", "comparisons", "metrics"]:
        block = event.get(key)
        if isinstance(block, list):
            for item in block:
                if isinstance(item, dict):
                    parsed = parse_null_metric(item)
                    if parsed:
                        yield parsed
                elif isinstance(item, str):
                    parsed = parse_null_metric_string(item)
                    if parsed:
                        yield parsed

    # Dict style fallback:
    nulls = event.get("nulls")
    if isinstance(nulls, dict):
        for null_model, metrics in nulls.items():
            if isinstance(metrics, dict):
                for metric_name, payload in metrics.items():
                    if isinstance(payload, dict):
                        row = parse_null_metric(payload)
                        if row:
                            row["metric"] = str(metric_name)
                            row["null_model"] = str(null_model)
                            yield row


def parse_alert(alert: Any) -> Dict[str, str]:
    if isinstance(alert, dict):
        return {
            "severity": str(first_of(alert, ["severity", "level"], "")),
            "code": str(first_of(alert, ["code", "kind", "alert_type"], "")),
            "interpretation_level": str(first_of(alert, ["interpretation_level", "tier"], "")),
            "message": str(first_of(alert, ["message", "text", "description"], "")),
            "raw_alert": json.dumps(alert, ensure_ascii=False),
        }

    text = str(alert)
    severity = ""
    code = ""
    tier = ""

    # Example:
    # [info:STATISTICAL_DEVIATION:L3] Metric ...
    if text.startswith("[") and "]" in text:
        head = text[1:text.index("]")]
        parts = head.split(":")
        if len(parts) > 0:
            severity = parts[0]
        if len(parts) > 1:
            code = parts[1]
        if len(parts) > 2:
            tier = parts[2]

    return {
        "severity": severity,
        "code": code,
        "interpretation_level": tier,
        "message": text,
        "raw_alert": text,
    }


def write_csv(path: Path, rows: List[Dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)

    if not rows:
        path.write_text("", encoding="utf-8")
        return

    fieldnames: List[str] = []
    seen = set()

    # Preserve useful order by taking fields in first-seen order.
    for row in rows:
        for key in row.keys():
            if key not in seen:
                seen.add(key)
                fieldnames.append(key)

    with path.open("w", encoding="utf-8-sig", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)


def parse_report(report_path: Path, group_name: str) -> Tuple[Dict[str, Any], List[Dict[str, Any]], List[Dict[str, Any]]]:
    report = read_json(report_path)
    classification = classify_report(report_path, report)

    input_path = str(first_of(report, ["input", "input_path", "input_file"], ""))

    report_row = {
        "report_file": str(report_path),
        "report_group": group_name,
        "experiment": first_of(report, ["experiment"], ""),
        "status": first_of(report, ["status"], ""),
        "language_reported": first_of(report, ["language"], ""),
        "language_guess": classification["language_guess"],
        "corpus_family": classification["corpus_family"],
        "source": classification["source"],
        "input": input_path,
        "input_bytes": first_of(report, ["input_bytes"], ""),
        "input_chars": first_of(report, ["input_chars"], ""),
        "cleaned_chars": first_of(report, ["cleaned_chars"], ""),
        "null_samples": first_of(report, ["null_samples", "null_samples_per_null_model"], ""),
        "seed": first_of(report, ["seed"], ""),
        "tool_version": first_of(report, ["tool_version"], ""),
        "hyphen_policy": first_of(report, ["hyphen_policy"], ""),
    }

    metric_rows: List[Dict[str, Any]] = []
    alert_rows: List[Dict[str, Any]] = []

    for encoding_name, encoding in iter_encoding_reports(report):
        if not encoding_name:
            encoding_name = str(first_of(encoding, ["encoding", "name"], ""))

        encoding_base = {
            "encoding": encoding_name,
            "encoding_sequence_len": first_of(encoding, ["sequence_len", "length"], ""),
            "encoding_unique_symbols": first_of(encoding, ["unique_symbols"], ""),
            "encoding_symbol_entropy_bits": first_of(encoding, ["symbol_entropy_bits"], ""),
        }

        for event_name, event in iter_event_reports(encoding):
            if not event_name:
                event_name = str(first_of(event, ["event", "name"], ""))

            event_base = {
                **report_row,
                **encoding_base,
                "event": event_name,
                "event_description": first_of(event, ["description"], ""),
                "event_count": first_of(event, ["event_count"], ""),
            }

            for observed in iter_observed_metrics(event):
                metric_rows.append({
                    **event_base,
                    "metric": observed["metric"],
                    "null_model": "observed_only",
                    "observed": observed["observed"] if observed["observed"] is not None else "",
                    "null_mean": "",
                    "null_std": "",
                    "z": "",
                    "abs_z": "",
                    "p_emp": "",
                    "survives_markov_2": "",
                    "raw_metric_json": json.dumps(observed["raw"], ensure_ascii=False),
                })

            for null_metric in iter_null_metrics(event):
                z = null_metric["z"]
                p_emp = null_metric["p_emp"]
                null_model = null_metric["null_model"]

                survives = ""
                if null_model == "markov_2" and z is not None:
                    survives = abs(z) >= 3.0 and (p_emp is None or p_emp <= 0.05)

                metric_rows.append({
                    **event_base,
                    "metric": null_metric["metric"],
                    "null_model": null_model,
                    "observed": null_metric["observed"] if null_metric["observed"] is not None else "",
                    "null_mean": null_metric["null_mean"] if null_metric["null_mean"] is not None else "",
                    "null_std": null_metric["null_std"] if null_metric["null_std"] is not None else "",
                    "z": z if z is not None else "",
                    "abs_z": abs(z) if z is not None else "",
                    "p_emp": p_emp if p_emp is not None else "",
                    "survives_markov_2": survives,
                    "raw_metric_json": json.dumps(null_metric["raw"], ensure_ascii=False),
                })

            for alert in event.get("alerts", []) or []:
                parsed_alert = parse_alert(alert)
                alert_rows.append({
                    **event_base,
                    **parsed_alert,
                })

    return report_row, metric_rows, alert_rows


def build_alert_summary(alert_rows: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    counts: Dict[Tuple[str, str, str, str, str], int] = {}

    for row in alert_rows:
        key = (
            str(row.get("report_group", "")),
            str(row.get("corpus_family", "")),
            str(row.get("language_guess", "")),
            str(row.get("severity", "")),
            str(row.get("code", "")),
        )
        counts[key] = counts.get(key, 0) + 1

    output = []
    for (group, family, language, severity, code), count in sorted(counts.items()):
        output.append({
            "report_group": group,
            "corpus_family": family,
            "language_guess": language,
            "severity": severity,
            "code": code,
            "count": count,
        })

    return output


def main() -> int:
    parser = argparse.ArgumentParser(description="Summarise RRLANG JSON reports into CSV files.")
    parser.add_argument("--input", action="append", required=True, help="Input report folder. Repeat allowed.")
    parser.add_argument("--out", required=True, help="Output summary folder.")
    args = parser.parse_args()

    report_rows: List[Dict[str, Any]] = []
    metric_rows: List[Dict[str, Any]] = []
    alert_rows: List[Dict[str, Any]] = []

    for input_arg in args.input:
        input_dir = Path(input_arg)
        group_name = input_dir.name
        json_files = sorted(input_dir.rglob("*.json"))

        print(f"Scanning {input_dir} ({len(json_files)} JSON files)")

        for report_path in json_files:
            try:
                report_row, metrics, alerts = parse_report(report_path, group_name)
                report_rows.append(report_row)
                metric_rows.extend(metrics)
                alert_rows.extend(alerts)
            except Exception as exc:
                print(f"WARNING: could not parse {report_path}: {exc}")
                report_rows.append({
                    "report_file": str(report_path),
                    "report_group": group_name,
                    "parse_error": repr(exc),
                })

    markov2_survivors = []
    for row in metric_rows:
        if row.get("null_model") != "markov_2":
            continue
        z = finite_float(row.get("z"))
        p_emp = finite_float(row.get("p_emp"))
        if z is None:
            continue
        if abs(z) >= 3.0 and (p_emp is None or p_emp <= 0.05):
            markov2_survivors.append(row)

    top_deviations = []
    for row in metric_rows:
        if row.get("null_model") in ("", "observed_only"):
            continue
        z = finite_float(row.get("z"))
        if z is not None:
            top_deviations.append(row)

    top_deviations.sort(key=lambda row: abs(finite_float(row.get("z")) or 0.0), reverse=True)
    top_deviations = top_deviations[:500]

    out_dir = Path(args.out)
    write_csv(out_dir / "report_index.csv", report_rows)
    write_csv(out_dir / "metrics_flat.csv", metric_rows)
    write_csv(out_dir / "markov2_survivors.csv", markov2_survivors)
    write_csv(out_dir / "alert_events.csv", alert_rows)
    write_csv(out_dir / "alert_summary.csv", build_alert_summary(alert_rows))
    write_csv(out_dir / "top_deviations.csv", top_deviations)

    print()
    print(f"Reports parsed:      {len(report_rows)}")
    print(f"Metric rows:         {len(metric_rows)}")
    print(f"Alert rows:          {len(alert_rows)}")
    print(f"Markov-2 survivors:  {len(markov2_survivors)}")
    print(f"Output folder:       {out_dir}")

    return 0


    if __name__ == "__main__":
        raise SystemExit(main())
          '''

ps1 = r'''# install_summary_tool.ps1
    # Run from the rrlang repo root:
    #   powershell -ExecutionPolicy Bypass -File .\install_summary_tool.ps1

    $ErrorActionPreference = "Stop"

    $ScriptDir = Join-Path (Get-Location) "scripts"
    New-Item -Force -ItemType Directory $ScriptDir | Out-Null

    $Source = Join-Path $PSScriptRoot "summarise_rrlang_reports_v2.py"
    $Dest = Join-Path $ScriptDir "summarise_rrlang_reports_v2.py"

    Copy-Item -Force $Source $Dest

    Write-Host "Installed:" -ForegroundColor Green
    Write-Host "  $Dest"
    Write-Host ""
    Write-Host "Test command:" -ForegroundColor Cyan
    Write-Host "  py -3 .\scripts\summarise_rrlang_reports_v2.py --input .\outputs\corpus_batch_curated --out .\outputs\summary_curated"
    '''

readme = """# RRLANG Summary Tools v0.2

    This patch contains a properly formatted Python summariser.

    ## Install

    Extract this zip somewhere, then copy:

    ```text
    summarise_rrlang_reports_v2.py

    to:

    D:\\code\\5hr-conlang\\riemann_analysis\\rrlang\\scripts\\summarise_rrlang_reports_v2.py

    Or from the extracted patch folder, run:

    powershell -ExecutionPolicy Bypass -File .\\install_summary_tool.ps1
    Run curated summary

    From the RRLANG repo root:

    cd D:\\code\\5hr-conlang\\riemann_analysis\\rrlang

    py -3 .\\scripts\\summarise_rrlang_reports_v2.py `
      --input .\\outputs\\corpus_batch_curated `
      --out .\\outputs\\summary_curated
    Run combined summary
    py -3 .\\scripts\\summarise_rrlang_reports_v2.py `
      --input .\\outputs\\corpus_batch_curated `
      --input .\\outputs\\corpus_batch_sampled_25k `
      --input .\\outputs\\corpus_batch `
      --input .\\outputs `
      --out .\\outputs\\summary_all

    """

(base / "summarise_rrlang_reports_v2.py").write_text(script, encoding="utf-8", newline="\n")
(base / "install_summary_tool.ps1").write_text(ps1, encoding="utf-8", newline="\n")
(base / "README.md").write_text(readme, encoding="utf-8", newline="\n")

zip_path = Path("/mnt/data/rrlang_summary_tools_v0_2.zip")
with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
    for p in base.rglob("*"):
        z.write(p, p.relative_to(base.parent))

print(f"Created {zip_path}")
print(f"Created {base / 'summarise_rrlang_reports_v2.py'}")