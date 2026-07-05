use crate::types::{AnalyseOptions, EncodedSequence, EncodingKind, EventMap, HyphenPolicy, PunctuationPolicy, WhitespacePolicy};
use std::collections::HashMap;

pub fn preprocess_text(input: &str, options: &AnalyseOptions) -> String {
    let mut output = match options.case_policy {
        crate::types::CasePolicy::Preserve => input.to_string(),
        crate::types::CasePolicy::Lowercase => input.to_lowercase(),
    };

    output = apply_hyphen_policy(&output, options.hyphen_policy);

    if options.punctuation_policy == PunctuationPolicy::Remove {
        output = output
            .chars()
            .map(|ch| {
                if is_hyphen(ch) && matches!(options.hyphen_policy, HyphenPolicy::MorphemeBoundary | HyphenPolicy::WordInternal) {
                    ch
                } else if is_punctuation(ch) {
                    ' '
                } else {
                    ch
                }
            })
            .collect();
    }

    if options.whitespace_policy == WhitespacePolicy::Normalise {
        let mut normalised = String::with_capacity(output.len());
        let mut last_was_space = false;
        for ch in output.chars() {
            if ch.is_whitespace() {
                if !last_was_space {
                    normalised.push(' ');
                    last_was_space = true;
                }
            } else {
                normalised.push(ch);
                last_was_space = false;
            }
        }
        output = normalised.trim().to_string();
    }

    output
}

fn apply_hyphen_policy(input: &str, policy: HyphenPolicy) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        if is_hyphen(ch) {
            match policy {
                HyphenPolicy::Remove => {}
                HyphenPolicy::Punctuation | HyphenPolicy::MorphemeBoundary | HyphenPolicy::WordInternal => output.push('-'),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

pub fn encode_text(cleaned_text: &str, kind: EncodingKind, options: &AnalyseOptions) -> EncodedSequence {
    match kind {
        EncodingKind::Utf8Bits => encode_utf8_bits(cleaned_text),
        EncodingKind::BitText => encode_bit_text(cleaned_text),
        EncodingKind::Grapheme => encode_grapheme(cleaned_text, options.hyphen_policy),
        EncodingKind::GraphemeClass => encode_grapheme_class(cleaned_text, options.hyphen_policy),
        EncodingKind::WordBoundary => encode_word_boundary(cleaned_text),
        EncodingKind::FrequencyClass => encode_frequency_class(cleaned_text, options.hyphen_policy),
    }
}

fn encode_utf8_bits(text: &str) -> EncodedSequence {
    let mut labels = Vec::with_capacity(text.len() * 8);
    for byte in text.as_bytes() {
        for bit_index in (0..8).rev() {
            let bit = (byte >> bit_index) & 1;
            labels.push(bit.to_string());
        }
    }

    build_bit_sequence(EncodingKind::Utf8Bits, labels, vec![
        "E0 raw UTF-8 bit analysis is diagnostic only. Raw-byte findings are not linguistic evidence without cross-encoding support.".to_string(),
    ])
}

fn encode_bit_text(text: &str) -> EncodedSequence {
    let mut labels = Vec::new();
    let mut ignored_non_bit = 0usize;
    for ch in text.chars() {
        if ch == '0' || ch == '1' {
            labels.push(ch.to_string());
        } else if !ch.is_whitespace() {
            ignored_non_bit += 1;
        }
    }

    let mut notes = vec![
        "E0b bit-text analysis treats literal '0' and '1' characters as the bitstream itself. This is the correct control mode for random-bit text files.".to_string(),
    ];
    if ignored_non_bit > 0 {
        notes.push(format!(
            "Ignored {ignored_non_bit} non-bit, non-whitespace characters while building the bit_text stream."
        ));
    }
    if labels.is_empty() {
        notes.push("No literal 0/1 bits were found; bit_text metrics will be degenerate.".to_string());
    }

    build_bit_sequence(EncodingKind::BitText, labels, notes)
}

fn build_bit_sequence(kind: EncodingKind, labels: Vec<String>, notes: Vec<String>) -> EncodedSequence {
    let bit_one = labels.iter().map(|label| label == "1").collect::<Vec<_>>();
    let mut transition = Vec::with_capacity(labels.len());
    let mut previous: Option<&str> = None;
    for label in &labels {
        let changed = previous.map(|prev| prev != label).unwrap_or(false);
        transition.push(changed);
        previous = Some(label.as_str());
    }

    EncodedSequence {
        kind,
        labels,
        event_maps: vec![
            EventMap {
                name: "bit_one".to_string(),
                description: "Positions where the bitstream contains 1.".to_string(),
                values: bit_one,
            },
            EventMap {
                name: "bit_transition".to_string(),
                description: "Positions where the bitstream changes from the previous bit.".to_string(),
                values: transition,
            },
        ],
        notes,
    }
}

fn encode_grapheme(text: &str, hyphen_policy: HyphenPolicy) -> EncodedSequence {
    let labels = text.chars().map(|ch| ch.to_string()).collect::<Vec<_>>();
    let chars = text.chars().collect::<Vec<_>>();
    let mut notes = vec![
        "MVP E1 uses Unicode scalar values as a deterministic grapheme approximation. True extended grapheme-cluster segmentation is a planned later extension.".to_string(),
    ];
    notes.push(format!("Hyphen policy for this run: {}.", hyphen_policy.as_str()));
    EncodedSequence {
        kind: EncodingKind::Grapheme,
        labels,
        event_maps: build_common_char_event_maps(&chars, hyphen_policy),
        notes,
    }
}

fn encode_grapheme_class(text: &str, hyphen_policy: HyphenPolicy) -> EncodedSequence {
    let chars = text.chars().collect::<Vec<_>>();
    let labels = chars.iter().map(|ch| char_class(*ch, hyphen_policy).to_string()).collect::<Vec<_>>();
    let mut classes = vec!["vowel", "consonant", "digit", "whitespace", "punctuation", "hyphen_boundary", "other"];
    let mut event_maps = Vec::new();
    for class in classes.drain(..) {
        let values = labels.iter().map(|label| label == class).collect::<Vec<_>>();
        event_maps.push(EventMap {
            name: class.to_string(),
            description: format!("Positions classified as {class} in the grapheme-class stream."),
            values,
        });
    }
    EncodedSequence {
        kind: EncodingKind::GraphemeClass,
        labels,
        event_maps,
        notes: vec![format!(
            "E2 maps Unicode scalar values into broad grapheme classes. Hyphen policy: {}.",
            hyphen_policy.as_str()
        )],
    }
}

fn encode_word_boundary(text: &str) -> EncodedSequence {
    let chars = text.chars().collect::<Vec<_>>();
    let mut labels = Vec::with_capacity(chars.len());
    let mut values = Vec::with_capacity(chars.len());
    for ch in chars {
        let boundary = ch.is_whitespace();
        labels.push(if boundary { "boundary" } else { "non_boundary" }.to_string());
        values.push(boundary);
    }
    EncodedSequence {
        kind: EncodingKind::WordBoundary,
        labels,
        event_maps: vec![EventMap {
            name: "word_boundary".to_string(),
            description: "Positions marked as word boundaries after declared whitespace preprocessing.".to_string(),
            values,
        }],
        notes: vec!["E6 MVP word boundaries are whitespace-derived and should be interpreted as orthographic/token boundaries.".to_string()],
    }
}

fn encode_frequency_class(text: &str, hyphen_policy: HyphenPolicy) -> EncodedSequence {
    let tokens = tokenize_words(text, hyphen_policy);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for token in &tokens {
        *counts.entry(token.clone()).or_insert(0) += 1;
    }

    let mut labels = Vec::with_capacity(tokens.len());
    for token in &tokens {
        let count = *counts.get(token).unwrap_or(&0);
        let class = if count == 1 {
            "hapax"
        } else if count >= 5 {
            "high_frequency"
        } else if count >= 2 {
            "mid_frequency"
        } else {
            "other"
        };
        labels.push(class.to_string());
    }

    let class_names = ["hapax", "mid_frequency", "high_frequency"];
    let mut event_maps = Vec::new();
    for class_name in class_names {
        event_maps.push(EventMap {
            name: class_name.to_string(),
            description: format!("Token positions in frequency class {class_name}."),
            values: labels.iter().map(|label| label == class_name).collect(),
        });
    }

    EncodedSequence {
        kind: EncodingKind::FrequencyClass,
        labels,
        event_maps,
        notes: vec![format!(
            "E7 MVP frequency classes are calculated inside the supplied sample only. Hyphen policy: {}.",
            hyphen_policy.as_str()
        )],
    }
}

fn build_common_char_event_maps(chars: &[char], hyphen_policy: HyphenPolicy) -> Vec<EventMap> {
    let vowel = chars.iter().map(|ch| is_basic_vowel(*ch)).collect::<Vec<_>>();
    let consonant = chars
        .iter()
        .map(|ch| ch.is_alphabetic() && !is_basic_vowel(*ch))
        .collect::<Vec<_>>();
    let whitespace = chars.iter().map(|ch| ch.is_whitespace()).collect::<Vec<_>>();
    let punctuation = chars.iter().map(|ch| punctuation_under_policy(*ch, hyphen_policy)).collect::<Vec<_>>();
    let digit = chars.iter().map(|ch| ch.is_ascii_digit()).collect::<Vec<_>>();
    let hyphen_boundary = chars
        .iter()
        .map(|ch| is_hyphen(*ch) && hyphen_policy == HyphenPolicy::MorphemeBoundary)
        .collect::<Vec<_>>();
    vec![
        EventMap { name: "vowel".to_string(), description: "Basic Latin-vowel positions plus common accented vowels.".to_string(), values: vowel },
        EventMap { name: "consonant".to_string(), description: "Alphabetic non-vowel positions under the MVP classifier.".to_string(), values: consonant },
        EventMap { name: "whitespace".to_string(), description: "Whitespace positions.".to_string(), values: whitespace },
        EventMap { name: "punctuation".to_string(), description: "Punctuation positions under the declared punctuation/hyphen policies.".to_string(), values: punctuation },
        EventMap { name: "hyphen_boundary".to_string(), description: "Hyphen positions treated as morpheme boundaries by the declared hyphen policy.".to_string(), values: hyphen_boundary },
        EventMap { name: "digit".to_string(), description: "ASCII digit positions.".to_string(), values: digit },
    ]
}

fn char_class(ch: char, hyphen_policy: HyphenPolicy) -> &'static str {
    if is_basic_vowel(ch) {
        "vowel"
    } else if ch.is_alphabetic() {
        "consonant"
    } else if ch.is_ascii_digit() {
        "digit"
    } else if ch.is_whitespace() {
        "whitespace"
    } else if is_hyphen(ch) && hyphen_policy == HyphenPolicy::MorphemeBoundary {
        "hyphen_boundary"
    } else if punctuation_under_policy(ch, hyphen_policy) {
        "punctuation"
    } else {
        "other"
    }
}

fn tokenize_words(text: &str, hyphen_policy: HyphenPolicy) -> Vec<String> {
    text.split(|ch: char| {
        if ch == '_' {
            false
        } else if is_hyphen(ch) {
            hyphen_policy != HyphenPolicy::WordInternal
        } else {
            !ch.is_alphanumeric()
        }
    })
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect()
}

fn punctuation_under_policy(ch: char, hyphen_policy: HyphenPolicy) -> bool {
    if is_hyphen(ch) {
        return hyphen_policy == HyphenPolicy::Punctuation;
    }
    is_punctuation(ch)
}

fn is_basic_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'e' | 'i' | 'o' | 'u' |
        'A' | 'E' | 'I' | 'O' | 'U' |
        'á' | 'é' | 'í' | 'ó' | 'ú' |
        'à' | 'è' | 'ì' | 'ò' | 'ù' |
        'â' | 'ê' | 'î' | 'ô' | 'û' |
        'ä' | 'ë' | 'ï' | 'ö' | 'ü' |
        'ã' | 'õ' | 'å' | 'æ' | 'œ'
    )
}

fn is_hyphen(ch: char) -> bool {
    matches!(ch, '-' | '‐' | '‑' | '‒' | '–' | '—')
}

fn is_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation()
        || matches!(
            ch,
            '“' | '”' | '‘' | '’' | '—' | '–' | '…' | '。' | '、' | '，' | '！' | '？' | '؛' | '،'
        )
}
