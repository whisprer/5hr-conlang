use crate::types::{Alert, EncodingAnalysis, EventAnalysis, ExperimentReport, MetricComparison, MetricObservation};

pub fn report_to_text(report: &ExperimentReport) -> String {
    let mut out = String::new();
    out.push_str("RRLANG ANALYSIS REPORT\n");
    out.push_str("======================\n\n");
    out.push_str(&format!("Experiment: {}\n", report.metadata.experiment_name));
    out.push_str(&format!("Status: {}\n", report.metadata.status));
    out.push_str(&format!("Language: {}\n", report.metadata.language));
    out.push_str(&format!("Input: {}\n", report.metadata.input_path));
    out.push_str(&format!("Input bytes: {}\n", report.input_byte_len));
    out.push_str(&format!("Input chars: {}\n", report.input_char_len));
    out.push_str(&format!("Cleaned chars: {}\n", report.cleaned_char_len));
    out.push_str(&format!("Null samples per null model: {}\n", report.metadata.null_samples));
    out.push_str(&format!("Seed: {}\n", report.metadata.seed));
    out.push_str(&format!("Tool version: {}\n", report.metadata.tool_version));
    out.push_str(&format!("Hyphen policy: {}\n", report.metadata.hyphen_policy));
    match report.metadata.max_chars {
        Some(max_chars) => out.push_str(&format!("Max chars cap: {}\n\n", max_chars)),
        None => out.push_str("Max chars cap: none\n\n"),
    }

    if !report.global_alerts.is_empty() {
        out.push_str("Global alerts:\n");
        for alert in &report.global_alerts {
            out.push_str(&format_alert(alert));
        }
        out.push('\n');
    }

    for encoding in &report.encodings {
        out.push_str(&format_encoding_text(encoding));
    }

    out.push_str("Interpretation rule:\n");
    out.push_str("  This tool reports measurements and evidence-tiered warnings, not origin classifications.\n");
    out.push_str("  Raw UTF-8 findings are diagnostic only unless supported by linguistic encodings. v0.3 adds max-chars caps, batch/resume ergonomics, fast/linguistic profiles, and skip-raw support.\n");
    out
}

fn format_encoding_text(encoding: &EncodingAnalysis) -> String {
    let mut out = String::new();
    out.push_str(&format!("Encoding: {}\n", encoding.encoding.as_str()));
    out.push_str(&format!("  sequence_len: {}\n", encoding.sequence_len));
    out.push_str(&format!("  unique_symbols: {}\n", encoding.unique_symbol_count));
    out.push_str(&format!("  symbol_entropy_bits: {:.6}\n", encoding.symbol_entropy));
    for note in &encoding.notes {
        out.push_str(&format!("  note: {}\n", note));
    }
    out.push('\n');

    for event in &encoding.events {
        out.push_str(&format_event_text(event));
    }
    out.push('\n');
    out
}

fn format_event_text(event: &EventAnalysis) -> String {
    let mut out = String::new();
    out.push_str(&format!("  Event: {}\n", event.event_name));
    out.push_str(&format!("    description: {}\n", event.description));
    out.push_str(&format!("    event_count: {}\n", event.event_count));
    if !event.alerts.is_empty() {
        out.push_str("    alerts:\n");
        for alert in &event.alerts {
            out.push_str(&format!("      - [{}:{}:L{}] {}\n", alert.severity, alert.code, alert.interpretation_level, alert.message));
        }
    }
    out.push_str("    observed:\n");
    for obs in &event.observed {
        out.push_str(&format!("      - {} = {:.6} ({})\n", obs.name, obs.value, obs.details));
    }
    out.push_str("    null-adjusted metrics:\n");
    for comparison in &event.comparisons {
        out.push_str(&format!(
            "      - {} [{}]: observed={:.6}, null_mean={:.6}, null_std={:.6}, z={:.3}, p_emp={:.6}\n",
            comparison.name,
            comparison.null_model,
            comparison.observed,
            comparison.null_mean,
            comparison.null_std,
            comparison.z_score,
            comparison.empirical_p
        ));
    }
    out
}

fn format_alert(alert: &Alert) -> String {
    format!(
        "  - [{}:{}:L{}] {}\n",
        alert.severity, alert.code, alert.interpretation_level, alert.message
    )
}

pub fn report_to_json(report: &ExperimentReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"metadata\": {\n");
    out.push_str(&json_pair_str("experiment_name", &report.metadata.experiment_name, 4, true));
    out.push_str(&json_pair_str("input_path", &report.metadata.input_path, 4, true));
    out.push_str(&json_pair_str("language", &report.metadata.language, 4, true));
    out.push_str(&json_pair_num("unix_timestamp", report.metadata.unix_timestamp as f64, 4, true));
    out.push_str(&json_pair_num("null_samples", report.metadata.null_samples as f64, 4, true));
    out.push_str(&json_pair_num("seed", report.metadata.seed as f64, 4, true));
    out.push_str(&json_pair_str("case_policy", &report.metadata.case_policy, 4, true));
    out.push_str(&json_pair_str("punctuation_policy", &report.metadata.punctuation_policy, 4, true));
    out.push_str(&json_pair_str("hyphen_policy", &report.metadata.hyphen_policy, 4, true));
    out.push_str(&json_pair_str("whitespace_policy", &report.metadata.whitespace_policy, 4, true));
    out.push_str(&json_pair_opt_usize("max_chars", report.metadata.max_chars, 4, true));
    out.push_str(&json_pair_str("tool_version", &report.metadata.tool_version, 4, true));
    out.push_str(&json_pair_str("status", &report.metadata.status, 4, false));
    out.push_str("  },\n");
    out.push_str(&json_pair_num("input_byte_len", report.input_byte_len as f64, 2, true));
    out.push_str(&json_pair_num("input_char_len", report.input_char_len as f64, 2, true));
    out.push_str(&json_pair_num("cleaned_char_len", report.cleaned_char_len as f64, 2, true));
    out.push_str("  \"global_alerts\": ");
    out.push_str(&alerts_json(&report.global_alerts, 2));
    out.push_str(",\n");
    out.push_str("  \"encodings\": [\n");
    for (index, encoding) in report.encodings.iter().enumerate() {
        out.push_str(&encoding_json(encoding, 4));
        if index + 1 != report.encodings.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn encoding_json(encoding: &EncodingAnalysis, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    out.push_str(&json_pair_str("encoding", encoding.encoding.as_str(), spaces + 2, true));
    out.push_str(&json_pair_num("sequence_len", encoding.sequence_len as f64, spaces + 2, true));
    out.push_str(&json_pair_num("unique_symbol_count", encoding.unique_symbol_count as f64, spaces + 2, true));
    out.push_str(&json_pair_num("symbol_entropy", encoding.symbol_entropy, spaces + 2, true));
    out.push_str(&format!("{}\"notes\": [", " ".repeat(spaces + 2)));
    for (index, note) in encoding.notes.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", escape_json(note)));
    }
    out.push_str("],\n");
    out.push_str(&format!("{}\"events\": [\n", " ".repeat(spaces + 2)));
    for (index, event) in encoding.events.iter().enumerate() {
        out.push_str(&event_json(event, spaces + 4));
        if index + 1 != encoding.events.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&format!("{}]\n", " ".repeat(spaces + 2)));
    out.push_str(&format!("{}}}", pad));
    out
}

fn event_json(event: &EventAnalysis, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    out.push_str(&json_pair_str("event_name", &event.event_name, spaces + 2, true));
    out.push_str(&json_pair_str("description", &event.description, spaces + 2, true));
    out.push_str(&json_pair_num("event_count", event.event_count as f64, spaces + 2, true));
    out.push_str(&format!("{}\"alerts\": ", " ".repeat(spaces + 2)));
    out.push_str(&alerts_json(&event.alerts, spaces + 2));
    out.push_str(",\n");
    out.push_str(&format!("{}\"observed\": [\n", " ".repeat(spaces + 2)));
    for (index, obs) in event.observed.iter().enumerate() {
        out.push_str(&observation_json(obs, spaces + 4));
        if index + 1 != event.observed.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&format!("{}],\n", " ".repeat(spaces + 2)));
    out.push_str(&format!("{}\"comparisons\": [\n", " ".repeat(spaces + 2)));
    for (index, comparison) in event.comparisons.iter().enumerate() {
        out.push_str(&comparison_json(comparison, spaces + 4));
        if index + 1 != event.comparisons.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&format!("{}]\n", " ".repeat(spaces + 2)));
    out.push_str(&format!("{}}}", pad));
    out
}

fn observation_json(obs: &MetricObservation, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    out.push_str(&json_pair_str("name", &obs.name, spaces + 2, true));
    out.push_str(&json_pair_num("value", obs.value, spaces + 2, true));
    out.push_str(&json_pair_str("details", &obs.details, spaces + 2, false));
    out.push_str(&format!("{}}}", pad));
    out
}

fn comparison_json(comparison: &MetricComparison, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    out.push_str(&json_pair_str("name", &comparison.name, spaces + 2, true));
    out.push_str(&json_pair_str("null_model", &comparison.null_model, spaces + 2, true));
    out.push_str(&json_pair_num("observed", comparison.observed, spaces + 2, true));
    out.push_str(&json_pair_num("null_mean", comparison.null_mean, spaces + 2, true));
    out.push_str(&json_pair_num("null_std", comparison.null_std, spaces + 2, true));
    out.push_str(&json_pair_num("z_score", comparison.z_score, spaces + 2, true));
    out.push_str(&json_pair_num("empirical_p", comparison.empirical_p, spaces + 2, true));
    out.push_str(&json_pair_num("null_samples", comparison.null_samples as f64, spaces + 2, false));
    out.push_str(&format!("{}}}", pad));
    out
}

fn alerts_json(alerts: &[Alert], spaces: usize) -> String {
    if alerts.is_empty() {
        return "[]".to_string();
    }
    let pad = " ".repeat(spaces);
    let item_pad = " ".repeat(spaces + 2);
    let mut out = String::new();
    out.push_str("[\n");
    for (index, alert) in alerts.iter().enumerate() {
        out.push_str(&format!("{}{{\n", item_pad));
        out.push_str(&json_pair_str("code", &alert.code, spaces + 4, true));
        out.push_str(&json_pair_str("severity", &alert.severity, spaces + 4, true));
        out.push_str(&json_pair_num("interpretation_level", alert.interpretation_level as f64, spaces + 4, true));
        out.push_str(&json_pair_str("message", &alert.message, spaces + 4, false));
        out.push_str(&format!("{} }}", item_pad));
        if index + 1 != alerts.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&format!("{}]", pad));
    out
}

fn json_pair_str(key: &str, value: &str, spaces: usize, comma: bool) -> String {
    format!(
        "{}\"{}\": \"{}\"{}\n",
        " ".repeat(spaces),
        escape_json(key),
        escape_json(value),
        if comma { "," } else { "" }
    )
}

fn json_pair_opt_usize(key: &str, value: Option<usize>, spaces: usize, comma: bool) -> String {
    let formatted = match value {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    };
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(spaces),
        escape_json(key),
        formatted,
        if comma { "," } else { "" }
    )
}

fn json_pair_num(key: &str, value: f64, spaces: usize, comma: bool) -> String {
    let formatted = if value.is_finite() {
        if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
            format!("{:.0}", value)
        } else {
            format!("{:.12}", value)
        }
    } else {
        "null".to_string()
    };
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(spaces),
        escape_json(key),
        formatted,
        if comma { "," } else { "" }
    )
}

fn escape_json(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}
