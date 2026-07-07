#!/usr/bin/env python3
"""
summarise_rrlang_reports_v3.py

Flatten RRLANG v0.2 JSON reports into CSV tables.

This script is deliberately dependency-free: only Python's standard library.

Run from the RRLANG repo root, for example:

  py -3 .\scripts\summarise_rrlang_reports_v3.py `
    --input .\outputs\corpus_batch_curated `
    --out .\outputs\summary_curated

Outputs:
  report_index.csv
  metrics_flat.csv
  markov2_survivors.csv
  alert_events.csv
  alert_summary.csv
  top_deviations.csv
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import re
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


Row = Dict[str, Any]


def to_float(value: Any) -> Optional[float]:
    if value is None or isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        value = float(value)
        return value if math.isfinite(value) else None
    if isinstance(value, str):
        text = value.strip().rstrip(',')
        try:
            out = float(text)
        except ValueError:
            return None
        return out if math.isfinite(out) else None
    return None


def first(obj: Dict[str, Any], keys: Iterable[str], default: Any = "") -> Any:
    for key in keys:
        if key in obj:
            return obj[key]
    return default


def json_text(value: Any) -> str:
    try:
        return json.dumps(value, ensure_ascii=False)
    except Exception:
        return str(value)


def read_report(path: Path) -> Dict[str, Any]:
    with path.open('r', encoding='utf-8-sig') as f:
        return json.load(f)


def classify(report_path: Path, report: Dict[str, Any]) -> Dict[str, str]:
    input_path = str(first(report, ['input', 'input_path', 'input_file'], ''))
    hay = (input_path + ' ' + str(report_path)).replace('\\', '/').lower()

    family = 'unknown'
    source = 'unknown'
    language = str(first(report, ['language'], '')).strip() or 'unknown'

    if '/parallel/udhr/' in hay or 'datasets_parallel_udhr_' in hay:
        family, source = 'parallel_udhr', 'udhr'
    elif '/native/tatoeba_cc0/' in hay or 'datasets_native_tatoeba_cc0_' in hay:
        family, source = 'native_tatoeba_cc0', 'tatoeba_cc0'
    elif '/native/wikipedia_api/' in hay or 'datasets_native_wikipedia_api_' in hay:
        family, source = 'native_wikipedia', 'wikipedia_api'
    elif '/private_parallel/' in hay or 'datasets_private_parallel_' in hay:
        family, source = 'private_parallel', 'private_parallel'
    elif '/controls/' in hay or 'datasets_controls_' in hay:
        family, source = 'controls', 'controls'

    slash_markers = [
        '/parallel/udhr/',
        '/native/tatoeba_cc0/',
        '/native/wikipedia_api/',
    ]
    for marker in slash_markers:
        if marker in hay:
            tail = hay.split(marker, 1)[1]
            candidate = tail.split('/', 1)[0].strip('_- ')
            if candidate:
                language = candidate
            break

    flat_patterns = [
        r'datasets_parallel_udhr_([a-z]{2,3})_',
        r'datasets_native_tatoeba_cc0_([a-z]{2,3})_',
        r'datasets_native_wikipedia_api_([a-z]{2,3})_',
    ]
    for pattern in flat_patterns:
        m = re.search(pattern, hay)
        if m:
            language = m.group(1)
            break

    if 'mesu' in hay:
        language = 'mesu'

    return {'corpus_family': family, 'source': source, 'language_guess': language}


def iter_encodings(report: Dict[str, Any]) -> Iterable[Tuple[str, Dict[str, Any]]]:
    block = first(report, ['encodings', 'encoding_reports', 'encoding_results'], [])

    if isinstance(block, dict):
        for name, item in block.items():
            if isinstance(item, dict):
                yield str(name), item
        return

    if isinstance(block, list):
        for item in block:
            if isinstance(item, dict):
                name = str(first(item, ['name', 'encoding', 'encoding_name'], ''))
                yield name, item
        return


def iter_events(encoding: Dict[str, Any]) -> Iterable[Tuple[str, Dict[str, Any]]]:
    block = first(encoding, ['events', 'event_reports', 'event_results'], [])

    if isinstance(block, dict):
        for name, item in block.items():
            if isinstance(item, dict):
                yield str(name), item
        return

    if isinstance(block, list):
        for item in block:
            if isinstance(item, dict):
                name = str(first(item, ['name', 'event', 'event_name'], ''))
                yield name, item
        return


def parse_observed_item(item: Any) -> Optional[Row]:
    if isinstance(item, dict):
        name = str(first(item, ['metric', 'name', 'key', 'id'], ''))
        value = to_float(first(item, ['observed', 'value', 'score'], None))
        if name:
            return {'metric': name, 'observed': value, 'raw': item}
        return None

    if isinstance(item, (list, tuple)) and len(item) >= 2:
        return {'metric': str(item[0]), 'observed': to_float(item[1]), 'raw': item}

    if isinstance(item, str) and '=' in item:
        left, right = item.split('=', 1)
        number = right.strip().split()[0].rstrip(',')
        return {'metric': left.strip(), 'observed': to_float(number), 'raw': item}

    return None


def iter_observed(event: Dict[str, Any]) -> Iterable[Row]:
    block = event.get('observed', [])

    if isinstance(block, dict):
        for k, v in block.items():
            yield {'metric': str(k), 'observed': to_float(v), 'raw': {k: v}}
        return

    if isinstance(block, list):
        for item in block:
            parsed = parse_observed_item(item)
            if parsed:
                yield parsed
        return


def parse_null_string(text: str) -> Optional[Row]:
    # Example:
    # gap_entropy [markov_2]: observed=2.1, null_mean=2.2, null_std=0.03, z=-4.2, p_emp=0.009901
    if ':' not in text:
        return None
    head, body = text.split(':', 1)
    metric = head.strip()
    null_model = ''
    if '[' in metric and ']' in metric:
        null_model = metric.split('[', 1)[1].split(']', 1)[0].strip()
        metric = metric.split('[', 1)[0].strip()
    if not metric:
        return None

    out: Row = {
        'metric': metric,
        'null_model': null_model,
        'observed': None,
        'null_mean': None,
        'null_std': None,
        'z': None,
        'p_emp': None,
        'raw': text,
    }
    for key in ['observed', 'null_mean', 'null_std', 'z', 'p_emp']:
        m = re.search(rf'{key}\s*=\s*([-+0-9.eE]+)', body)
        if m:
            out[key] = to_float(m.group(1))
    return out


def parse_null_dict(item: Dict[str, Any]) -> Optional[Row]:
    metric = str(first(item, ['metric', 'name', 'metric_name'], ''))
    null_model = str(first(item, ['null_model', 'null', 'model'], ''))

    if not null_model and '[' in metric and ']' in metric:
        null_model = metric.split('[', 1)[1].split(']', 1)[0].strip()
        metric = metric.split('[', 1)[0].strip()

    if not metric and not null_model:
        return None

    return {
        'metric': metric,
        'null_model': null_model,
        'observed': to_float(first(item, ['observed', 'value'], None)),
        'null_mean': to_float(first(item, ['null_mean', 'mean', 'expected'], None)),
        'null_std': to_float(first(item, ['null_std', 'std', 'stdev'], None)),
        'z': to_float(first(item, ['z', 'z_score', 'zscore'], None)),
        'p_emp': to_float(first(item, ['p_emp', 'p', 'empirical_p', 'p_value'], None)),
        'raw': item,
    }


def iter_null_metrics(event: Dict[str, Any]) -> Iterable[Row]:
    for key in ['null_adjusted_metrics', 'null_metrics', 'comparisons', 'metrics']:
        block = event.get(key)
        if not isinstance(block, list):
            continue
        for item in block:
            parsed = None
            if isinstance(item, dict):
                parsed = parse_null_dict(item)
            elif isinstance(item, str):
                parsed = parse_null_string(item)
            if parsed:
                yield parsed

    nulls = event.get('nulls')
    if isinstance(nulls, dict):
        for null_model, metrics in nulls.items():
            if isinstance(metrics, dict):
                for metric_name, payload in metrics.items():
                    if isinstance(payload, dict):
                        parsed = parse_null_dict(payload)
                        if parsed:
                            parsed['metric'] = str(metric_name)
                            parsed['null_model'] = str(null_model)
                            yield parsed


def parse_alert(alert: Any) -> Row:
    if isinstance(alert, dict):
        return {
            'severity': str(first(alert, ['severity', 'level'], '')),
            'code': str(first(alert, ['code', 'kind', 'alert_type'], '')),
            'interpretation_level': str(first(alert, ['interpretation_level', 'tier'], '')),
            'message': str(first(alert, ['message', 'text', 'description'], '')),
            'raw_alert': json_text(alert),
        }

    text = str(alert)
    severity = ''
    code = ''
    tier = ''
    if text.startswith('[') and ']' in text:
        head = text[1:text.index(']')]
        parts = head.split(':')
        if len(parts) >= 1:
            severity = parts[0]
        if len(parts) >= 2:
            code = parts[1]
        if len(parts) >= 3:
            tier = parts[2]
    return {
        'severity': severity,
        'code': code,
        'interpretation_level': tier,
        'message': text,
        'raw_alert': text,
    }


def write_csv(path: Path, rows: List[Row]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not rows:
        path.write_text('', encoding='utf-8')
        return

    fields: List[str] = []
    seen = set()
    for row in rows:
        for k in row.keys():
            if k not in seen:
                seen.add(k)
                fields.append(k)

    with path.open('w', encoding='utf-8-sig', newline='') as f:
        writer = csv.DictWriter(f, fields, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(rows)


def parse_one_report(path: Path, group: str) -> Tuple[Row, List[Row], List[Row]]:
    report = read_report(path)
    cls = classify(path, report)

    report_row: Row = {
        'report_file': str(path),
        'report_group': group,
        'experiment': first(report, ['experiment'], ''),
        'status': first(report, ['status'], ''),
        'language_reported': first(report, ['language'], ''),
        'language_guess': cls['language_guess'],
        'corpus_family': cls['corpus_family'],
        'source': cls['source'],
        'input': first(report, ['input', 'input_path', 'input_file'], ''),
        'input_bytes': first(report, ['input_bytes'], ''),
        'input_chars': first(report, ['input_chars'], ''),
        'cleaned_chars': first(report, ['cleaned_chars'], ''),
        'null_samples': first(report, ['null_samples', 'null_samples_per_null_model'], ''),
        'seed': first(report, ['seed'], ''),
        'tool_version': first(report, ['tool_version'], ''),
        'hyphen_policy': first(report, ['hyphen_policy'], ''),
    }

    metrics: List[Row] = []
    alerts: List[Row] = []

    for enc_name, enc in iter_encodings(report):
        if not enc_name:
            enc_name = str(first(enc, ['name', 'encoding'], ''))

        enc_base: Row = {
            'encoding': enc_name,
            'encoding_sequence_len': first(enc, ['sequence_len', 'length'], ''),
            'encoding_unique_symbols': first(enc, ['unique_symbols'], ''),
            'encoding_symbol_entropy_bits': first(enc, ['symbol_entropy_bits'], ''),
        }

        for event_name, event in iter_events(enc):
            if not event_name:
                event_name = str(first(event, ['name', 'event'], ''))

            base: Row = {
                **report_row,
                **enc_base,
                'event': event_name,
                'event_description': first(event, ['description'], ''),
                'event_count': first(event, ['event_count'], ''),
            }

            for obs in iter_observed(event):
                metrics.append({
                    **base,
                    'metric': obs['metric'],
                    'null_model': 'observed_only',
                    'observed': '' if obs['observed'] is None else obs['observed'],
                    'null_mean': '',
                    'null_std': '',
                    'z': '',
                    'abs_z': '',
                    'p_emp': '',
                    'survives_markov_2': '',
                    'raw_metric_json': json_text(obs['raw']),
                })

            for nm in iter_null_metrics(event):
                z = nm['z']
                p = nm['p_emp']
                survives = ''
                if nm['null_model'] == 'markov_2' and z is not None:
                    survives = abs(z) >= 3.0 and (p is None or p <= 0.05)

                metrics.append({
                    **base,
                    'metric': nm['metric'],
                    'null_model': nm['null_model'],
                    'observed': '' if nm['observed'] is None else nm['observed'],
                    'null_mean': '' if nm['null_mean'] is None else nm['null_mean'],
                    'null_std': '' if nm['null_std'] is None else nm['null_std'],
                    'z': '' if z is None else z,
                    'abs_z': '' if z is None else abs(z),
                    'p_emp': '' if p is None else p,
                    'survives_markov_2': survives,
                    'raw_metric_json': json_text(nm['raw']),
                })

            for alert in event.get('alerts', []) or []:
                alerts.append({**base, **parse_alert(alert)})

    return report_row, metrics, alerts


def alert_summary_rows(alerts: List[Row]) -> List[Row]:
    counts: Dict[Tuple[str, str, str, str, str], int] = {}
    for a in alerts:
        key = (
            str(a.get('report_group', '')),
            str(a.get('corpus_family', '')),
            str(a.get('language_guess', '')),
            str(a.get('severity', '')),
            str(a.get('code', '')),
        )
        counts[key] = counts.get(key, 0) + 1

    rows: List[Row] = []
    for key, count in sorted(counts.items()):
        rows.append({
            'report_group': key[0],
            'corpus_family': key[1],
            'language_guess': key[2],
            'severity': key[3],
            'code': key[4],
            'count': count,
        })
    return rows


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', action='append', required=True, help='Input report folder. Can be repeated.')
    parser.add_argument('--out', required=True, help='Output summary folder.')
    args = parser.parse_args()

    report_rows: List[Row] = []
    metric_rows: List[Row] = []
    alert_rows: List[Row] = []

    for input_arg in args.input:
        input_dir = Path(input_arg)
        group = input_dir.name
        files = sorted(input_dir.rglob('*.json'))
        print(f'Scanning {input_dir} ({len(files)} JSON files)')

        if not input_dir.exists():
            print(f'ERROR: input folder does not exist: {input_dir}', file=sys.stderr)
            return 2

        for f in files:
            try:
                report, metrics, alerts = parse_one_report(f, group)
                report_rows.append(report)
                metric_rows.extend(metrics)
                alert_rows.extend(alerts)
            except Exception as exc:
                print(f'WARNING: failed to parse {f}: {exc}', file=sys.stderr)
                report_rows.append({'report_file': str(f), 'report_group': group, 'parse_error': repr(exc)})

    survivors: List[Row] = []
    deviations: List[Row] = []

    for row in metric_rows:
        z = to_float(row.get('z'))
        p = to_float(row.get('p_emp'))
        null_model = str(row.get('null_model', ''))

        if null_model not in ('', 'observed_only') and z is not None:
            deviations.append(row)

        if null_model == 'markov_2' and z is not None:
            if abs(z) >= 3.0 and (p is None or p <= 0.05):
                survivors.append(row)

    deviations.sort(key=lambda r: abs(to_float(r.get('z')) or 0.0), reverse=True)

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    write_csv(out / 'report_index.csv', report_rows)
    write_csv(out / 'metrics_flat.csv', metric_rows)
    write_csv(out / 'markov2_survivors.csv', survivors)
    write_csv(out / 'alert_events.csv', alert_rows)
    write_csv(out / 'alert_summary.csv', alert_summary_rows(alert_rows))
    write_csv(out / 'top_deviations.csv', deviations[:500])

    print('')
    print(f'Reports parsed:     {len(report_rows)}')
    print(f'Metric rows:        {len(metric_rows)}')
    print(f'Alert rows:         {len(alert_rows)}')
    print(f'Markov-2 survivors: {len(survivors)}')
    print(f'Output folder:      {out.resolve()}')

    return 0


if __name__ == '__main__':
    raise SystemExit(main())
