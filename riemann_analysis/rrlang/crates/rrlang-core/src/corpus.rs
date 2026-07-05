use crate::error::Result;
use std::fs;

#[derive(Debug, Clone)]
pub struct CorpusInspection {
    pub path: String,
    pub byte_len: usize,
    pub char_len: usize,
    pub line_count: usize,
    pub word_like_count: usize,
    pub unique_word_like_count: usize,
}

pub fn read_text_file(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn inspect_file(path: &str) -> Result<CorpusInspection> {
    let text = read_text_file(path)?;
    let mut words = std::collections::BTreeSet::new();
    let mut word_like_count = 0usize;
    for token in text.split(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-') {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            word_like_count += 1;
            words.insert(trimmed.to_ascii_lowercase());
        }
    }

    Ok(CorpusInspection {
        path: path.to_string(),
        byte_len: text.len(),
        char_len: text.chars().count(),
        line_count: text.lines().count(),
        word_like_count,
        unique_word_like_count: words.len(),
    })
}
