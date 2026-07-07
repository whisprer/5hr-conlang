use crate::corpus::read_text_file;
use crate::encode::{encode_text, preprocess_text};
use crate::error::{Result, RrlangError};
use crate::metrics::{analyse_event_map, shannon_entropy_labels};
use crate::types::{Alert, AnalyseOptions, EncodingAnalysis, ExperimentMetadata, ExperimentReport};
use std::collections::BTreeSet;

pub fn run_analysis(options: &AnalyseOptions) -> Result<ExperimentReport> {
    if options.input_path.trim().is_empty() {
        return Err(RrlangError::Message("No input path supplied.".to_string()));
    }

    let raw_text = read_text_file(&options.input_path)?;
    let mut cleaned_text = preprocess_text(&raw_text, options);
    let mut global_alerts = Vec::new();

    if let Some(max_chars) = options.max_chars {
        let cleaned_chars = cleaned_text.chars().count();
        if cleaned_chars > max_chars {
            cleaned_text = cleaned_text.chars().take(max_chars).collect::<String>();
            global_alerts.push(Alert::new(
                "INPUT_TRUNCATED",
                "warning",
                0,
                format!(
                    "Cleaned input was capped at {max_chars} characters for this run; original cleaned length was {cleaned_chars} characters.",
                ),
            ));
        }
    }

    if raw_text.is_empty() {
        global_alerts.push(Alert::new(
            "EMPTY_INPUT",
            "error",
            0,
            "Input file is empty; metric output is not meaningful.",
        ));
    }
    if cleaned_text.chars().count() < 128 {
        global_alerts.push(Alert::new(
            "SHORT_CORPUS",
            "warning",
            0,
            "Cleaned input has fewer than 128 characters; this run is exploratory only.",
        ));
    }

    let mut encodings = Vec::new();
    for (encoding_index, kind) in options.encodings.iter().enumerate() {
        let encoded = encode_text(&cleaned_text, *kind, options);
        let mut events = Vec::new();
        for (event_index, event_map) in encoded.event_maps.iter().enumerate() {
            let event_seed = options.seed
                ^ ((*kind as u64 + 1) << 16)
                ^ ((encoding_index as u64 + 1) << 32)
                ^ ((event_index as u64 + 1) << 48);
            events.push(analyse_event_map(
                *kind,
                event_map,
                options.null_samples,
                event_seed,
            ));
        }
        let unique_symbol_count = encoded.labels.iter().collect::<BTreeSet<_>>().len();
        encodings.push(EncodingAnalysis {
            encoding: *kind,
            sequence_len: encoded.labels.len(),
            symbol_entropy: shannon_entropy_labels(&encoded.labels),
            unique_symbol_count,
            notes: encoded.notes,
            events,
        });
    }

    add_cross_encoding_alerts(&mut global_alerts, &encodings);

    Ok(ExperimentReport {
        metadata: ExperimentMetadata::from_options(options),
        input_byte_len: raw_text.len(),
        input_char_len: raw_text.chars().count(),
        cleaned_char_len: cleaned_text.chars().count(),
        encodings,
        global_alerts,
    })
}

fn add_cross_encoding_alerts(global_alerts: &mut Vec<Alert>, encodings: &[EncodingAnalysis]) {
    let mut raw_has_strong_deviation = false;
    let mut linguistic_has_strong_deviation = false;

    for encoding in encodings {
        for event in &encoding.events {
            for comparison in &event.comparisons {
                if comparison.z_score.abs() >= 3.0 && comparison.null_std > 0.0 {
                    if matches!(encoding.encoding, crate::types::EncodingKind::Utf8Bits | crate::types::EncodingKind::BitText) {
                        raw_has_strong_deviation = true;
                    } else {
                        linguistic_has_strong_deviation = true;
                    }
                }
            }
        }
    }

    if raw_has_strong_deviation && !linguistic_has_strong_deviation {
        global_alerts.push(Alert::new(
            "RAW_ONLY_EFFECT",
            "warning",
            0,
            "Strong deviations were found in raw/bit diagnostic encodings but not in the enabled linguistic encodings. Treat as encoding-level unless replicated elsewhere.",
        ));
    }
}
