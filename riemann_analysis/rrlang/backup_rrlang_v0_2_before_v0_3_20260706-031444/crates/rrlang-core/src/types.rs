use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingKind {
    Utf8Bits,
    BitText,
    Grapheme,
    GraphemeClass,
    WordBoundary,
    FrequencyClass,
}

impl EncodingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EncodingKind::Utf8Bits => "utf8_bits",
            EncodingKind::BitText => "bit_text",
            EncodingKind::Grapheme => "grapheme",
            EncodingKind::GraphemeClass => "grapheme_class",
            EncodingKind::WordBoundary => "word_boundary",
            EncodingKind::FrequencyClass => "frequency_class",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let key = name.trim().to_ascii_lowercase().replace('-', "_");
        match key.as_str() {
            "utf8" | "utf8_bits" | "raw" | "raw_utf8" | "binary" => Some(Self::Utf8Bits),
            "bit_text" | "bits_text" | "text_bits" | "ascii_bits" | "literal_bits" => Some(Self::BitText),
            "grapheme" | "graphemes" | "char" | "chars" => Some(Self::Grapheme),
            "grapheme_class" | "class" | "classes" => Some(Self::GraphemeClass),
            "word_boundary" | "boundary" | "word_boundaries" => Some(Self::WordBoundary),
            "frequency" | "frequency_class" | "freq" | "token_frequency" => Some(Self::FrequencyClass),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasePolicy {
    Preserve,
    Lowercase,
}

impl CasePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            CasePolicy::Preserve => "preserve",
            CasePolicy::Lowercase => "lowercase",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "preserve" | "keep" => Some(Self::Preserve),
            "lower" | "lowercase" | "casefold" | "case_fold" => Some(Self::Lowercase),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunctuationPolicy {
    Preserve,
    Remove,
}

impl PunctuationPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            PunctuationPolicy::Preserve => "preserve",
            PunctuationPolicy::Remove => "remove",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "preserve" | "keep" => Some(Self::Preserve),
            "remove" | "strip" => Some(Self::Remove),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyphenPolicy {
    Punctuation,
    MorphemeBoundary,
    WordInternal,
    Remove,
}

impl HyphenPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            HyphenPolicy::Punctuation => "punctuation",
            HyphenPolicy::MorphemeBoundary => "morpheme_boundary",
            HyphenPolicy::WordInternal => "word_internal",
            HyphenPolicy::Remove => "remove",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let key = name.trim().to_ascii_lowercase().replace('-', "_");
        match key.as_str() {
            "punctuation" | "punct" | "hyphen_as_punctuation" => Some(Self::Punctuation),
            "morpheme" | "morpheme_boundary" | "boundary" | "hyphen_as_morpheme_boundary" => Some(Self::MorphemeBoundary),
            "word_internal" | "internal" | "keep" | "hyphen_as_word_internal" => Some(Self::WordInternal),
            "remove" | "strip" | "delete" | "hyphen_removed" => Some(Self::Remove),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespacePolicy {
    Preserve,
    Normalise,
}

impl WhitespacePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            WhitespacePolicy::Preserve => "preserve",
            WhitespacePolicy::Normalise => "normalise",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "preserve" | "keep" => Some(Self::Preserve),
            "normalise" | "normalize" | "collapse" => Some(Self::Normalise),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalyseOptions {
    pub input_path: String,
    pub output_json: Option<String>,
    pub output_text: Option<String>,
    pub language: String,
    pub experiment_name: String,
    pub encodings: Vec<EncodingKind>,
    pub null_samples: usize,
    pub seed: u64,
    pub case_policy: CasePolicy,
    pub punctuation_policy: PunctuationPolicy,
    pub hyphen_policy: HyphenPolicy,
    pub whitespace_policy: WhitespacePolicy,
}

impl Default for AnalyseOptions {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_json: Some("rrlang_report.json".to_string()),
            output_text: Some("rrlang_report.txt".to_string()),
            language: "unknown".to_string(),
            experiment_name: "rrlang_mvp_analysis_v0_2".to_string(),
            encodings: vec![
                EncodingKind::Utf8Bits,
                EncodingKind::Grapheme,
                EncodingKind::GraphemeClass,
                EncodingKind::WordBoundary,
                EncodingKind::FrequencyClass,
            ],
            null_samples: 100,
            seed: 18427,
            case_policy: CasePolicy::Lowercase,
            punctuation_policy: PunctuationPolicy::Preserve,
            hyphen_policy: HyphenPolicy::Punctuation,
            whitespace_policy: WhitespacePolicy::Normalise,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventMap {
    pub name: String,
    pub description: String,
    pub values: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct EncodedSequence {
    pub kind: EncodingKind,
    pub labels: Vec<String>,
    pub event_maps: Vec<EventMap>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MetricObservation {
    pub name: String,
    pub value: f64,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct MetricComparison {
    pub name: String,
    pub null_model: String,
    pub observed: f64,
    pub null_mean: f64,
    pub null_std: f64,
    pub z_score: f64,
    pub empirical_p: f64,
    pub null_samples: usize,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub code: String,
    pub severity: String,
    pub interpretation_level: u8,
    pub message: String,
}

impl Alert {
    pub fn new(code: &str, severity: &str, interpretation_level: u8, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: severity.to_string(),
            interpretation_level,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventAnalysis {
    pub event_name: String,
    pub description: String,
    pub event_count: usize,
    pub observed: Vec<MetricObservation>,
    pub comparisons: Vec<MetricComparison>,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone)]
pub struct EncodingAnalysis {
    pub encoding: EncodingKind,
    pub sequence_len: usize,
    pub symbol_entropy: f64,
    pub unique_symbol_count: usize,
    pub notes: Vec<String>,
    pub events: Vec<EventAnalysis>,
}

#[derive(Debug, Clone)]
pub struct ExperimentMetadata {
    pub experiment_name: String,
    pub input_path: String,
    pub language: String,
    pub unix_timestamp: u64,
    pub null_samples: usize,
    pub seed: u64,
    pub case_policy: String,
    pub punctuation_policy: String,
    pub hyphen_policy: String,
    pub whitespace_policy: String,
    pub tool_version: String,
    pub status: String,
}

impl ExperimentMetadata {
    pub fn from_options(options: &AnalyseOptions) -> Self {
        let unix_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        Self {
            experiment_name: options.experiment_name.clone(),
            input_path: options.input_path.clone(),
            language: options.language.clone(),
            unix_timestamp,
            null_samples: options.null_samples,
            seed: options.seed,
            case_policy: options.case_policy.as_str().to_string(),
            punctuation_policy: options.punctuation_policy.as_str().to_string(),
            hyphen_policy: options.hyphen_policy.as_str().to_string(),
            whitespace_policy: options.whitespace_policy.as_str().to_string(),
            tool_version: "0.2.0".to_string(),
            status: "exploratory".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExperimentReport {
    pub metadata: ExperimentMetadata,
    pub input_byte_len: usize,
    pub input_char_len: usize,
    pub cleaned_char_len: usize,
    pub encodings: Vec<EncodingAnalysis>,
    pub global_alerts: Vec<Alert>,
}
