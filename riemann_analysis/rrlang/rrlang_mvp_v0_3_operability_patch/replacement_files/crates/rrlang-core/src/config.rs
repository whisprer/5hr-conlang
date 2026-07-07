use crate::error::{Result, RrlangError};
use crate::types::{AnalyseOptions, CasePolicy, EncodingKind, HyphenPolicy, PunctuationPolicy, WhitespacePolicy};
use std::fs;

pub fn load_config_file(path: &str) -> Result<AnalyseOptions> {
    let text = fs::read_to_string(path)?;
    let mut options = AnalyseOptions::default();

    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "input_path" | "input" => options.input_path = parse_string(value),
            "output_json" | "json" => options.output_json = nonempty_opt(parse_string(value)),
            "output_text" | "text" => options.output_text = nonempty_opt(parse_string(value)),
            "language" => options.language = parse_string(value),
            "experiment_name" | "name" => options.experiment_name = parse_string(value),
            "null_samples" | "samples" => {
                options.null_samples = parse_string(value).parse::<usize>().map_err(|_| {
                    RrlangError::Message(format!("Invalid null sample count in config: {value}"))
                })?;
            }
            "seed" => {
                options.seed = parse_string(value).parse::<u64>().map_err(|_| {
                    RrlangError::Message(format!("Invalid seed in config: {value}"))
                })?;
            }
            "max_chars" | "max_characters" => {
                let parsed = parse_string(value);
                if parsed.trim().is_empty() || parsed.eq_ignore_ascii_case("none") {
                    options.max_chars = None;
                } else {
                    options.max_chars = Some(parsed.parse::<usize>().map_err(|_| {
                        RrlangError::Message(format!("Invalid max_chars in config: {value}"))
                    })?);
                }
            }
            "case_policy" => {
                let parsed = parse_string(value);
                options.case_policy = CasePolicy::from_name(&parsed).ok_or_else(|| {
                    RrlangError::Message(format!("Unknown case_policy: {parsed}"))
                })?;
            }
            "punctuation_policy" => {
                let parsed = parse_string(value);
                options.punctuation_policy = PunctuationPolicy::from_name(&parsed).ok_or_else(|| {
                    RrlangError::Message(format!("Unknown punctuation_policy: {parsed}"))
                })?;
            }
            "hyphen_policy" => {
                let parsed = parse_string(value);
                options.hyphen_policy = HyphenPolicy::from_name(&parsed).ok_or_else(|| {
                    RrlangError::Message(format!("Unknown hyphen_policy: {parsed}"))
                })?;
            }
            "whitespace_policy" => {
                let parsed = parse_string(value);
                options.whitespace_policy = WhitespacePolicy::from_name(&parsed).ok_or_else(|| {
                    RrlangError::Message(format!("Unknown whitespace_policy: {parsed}"))
                })?;
            }
            "encodings" => {
                let names = parse_string_array(value);
                let mut encodings = Vec::new();
                for name in names {
                    let kind = EncodingKind::from_name(&name).ok_or_else(|| {
                        RrlangError::Message(format!("Unknown encoding in config: {name}"))
                    })?;
                    encodings.push(kind);
                }
                if !encodings.is_empty() {
                    options.encodings = encodings;
                }
            }
            _ => {}
        }
    }

    Ok(options)
}

fn nonempty_opt(value: String) -> Option<String> {
    if value.trim().is_empty() || value.trim().eq_ignore_ascii_case("none") {
        None
    } else {
        Some(value)
    }
}

fn parse_string(value: &str) -> String {
    value
        .trim()
        .trim_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn parse_string_array(value: &str) -> Vec<String> {
    let trimmed = value.trim().trim_start_matches('[').trim_end_matches(']');
    trimmed
        .split(',')
        .map(parse_string)
        .filter(|item| !item.trim().is_empty())
        .collect()
}
