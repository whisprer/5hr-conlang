#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoolNullKind {
    DensityShuffle,
    Markov1,
    Markov2,
    GapOrderShuffle,
}

impl BoolNullKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BoolNullKind::DensityShuffle => "density_shuffle",
            BoolNullKind::Markov1 => "markov_1",
            BoolNullKind::Markov2 => "markov_2",
            BoolNullKind::GapOrderShuffle => "gap_order_shuffle",
        }
    }

    pub fn interpretation_strength(self) -> u8 {
        match self {
            BoolNullKind::DensityShuffle => 1,
            BoolNullKind::Markov1 => 2,
            BoolNullKind::Markov2 => 3,
            BoolNullKind::GapOrderShuffle => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn gen_range(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper_exclusive
    }

    pub fn gen_f64(&mut self) -> f64 {
        let raw = self.next_u64() >> 11;
        raw as f64 / ((1u64 << 53) as f64)
    }
}

pub fn generate_bool_null(values: &[bool], kind: BoolNullKind, rng: &mut XorShift64) -> Vec<bool> {
    match kind {
        BoolNullKind::DensityShuffle => shuffle_bool_preserving_count(values, rng),
        BoolNullKind::Markov1 => markov1_bool_sequence(values, rng),
        BoolNullKind::Markov2 => markov2_bool_sequence(values, rng),
        BoolNullKind::GapOrderShuffle => gap_order_shuffle(values, rng),
    }
}

pub fn shuffle_bool_preserving_count(values: &[bool], rng: &mut XorShift64) -> Vec<bool> {
    let mut shuffled = values.to_vec();
    if shuffled.len() < 2 {
        return shuffled;
    }
    for i in (1..shuffled.len()).rev() {
        let j = rng.gen_range(i + 1);
        shuffled.swap(i, j);
    }
    shuffled
}

fn markov1_bool_sequence(values: &[bool], rng: &mut XorShift64) -> Vec<bool> {
    if values.len() < 2 {
        return values.to_vec();
    }

    let mut transition_true = [[0usize; 2]; 2];
    let mut transition_total = [0usize; 2];
    for pair in values.windows(2) {
        let from = bool_index(pair[0]);
        let to = bool_index(pair[1]);
        transition_total[from] += 1;
        transition_true[from][to] += 1;
    }

    let true_density = values.iter().filter(|value| **value).count() as f64 / values.len() as f64;
    let mut out = Vec::with_capacity(values.len());
    out.push(rng.gen_f64() < true_density);

    for idx in 1..values.len() {
        let prev = bool_index(out[idx - 1]);
        let p_true = if transition_total[prev] == 0 {
            true_density
        } else {
            transition_true[prev][1] as f64 / transition_total[prev] as f64
        };
        out.push(rng.gen_f64() < p_true);
    }
    out
}

fn markov2_bool_sequence(values: &[bool], rng: &mut XorShift64) -> Vec<bool> {
    if values.len() < 3 {
        return markov1_bool_sequence(values, rng);
    }

    let mut next_true = [[0usize; 2]; 4];
    let mut total = [0usize; 4];
    for triple in values.windows(3) {
        let state = bool_index(triple[0]) * 2 + bool_index(triple[1]);
        let to = bool_index(triple[2]);
        total[state] += 1;
        next_true[state][to] += 1;
    }

    let true_density = values.iter().filter(|value| **value).count() as f64 / values.len() as f64;
    let mut starts = values.windows(2).map(|pair| [pair[0], pair[1]]).collect::<Vec<_>>();
    if starts.is_empty() {
        return markov1_bool_sequence(values, rng);
    }
    let start_index = rng.gen_range(starts.len());
    let start = starts.swap_remove(start_index);
    let mut out = Vec::with_capacity(values.len());
    out.push(start[0]);
    out.push(start[1]);

    for idx in 2..values.len() {
        let state = bool_index(out[idx - 2]) * 2 + bool_index(out[idx - 1]);
        let p_true = if total[state] == 0 {
            true_density
        } else {
            next_true[state][1] as f64 / total[state] as f64
        };
        out.push(rng.gen_f64() < p_true);
    }
    out
}

fn gap_order_shuffle(values: &[bool], rng: &mut XorShift64) -> Vec<bool> {
    let positions = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| if *value { Some(index) } else { None })
        .collect::<Vec<_>>();

    if positions.len() < 3 {
        return shuffle_bool_preserving_count(values, rng);
    }

    let first = positions[0];
    let mut gaps = positions.windows(2).map(|pair| pair[1] - pair[0]).collect::<Vec<_>>();
    for i in (1..gaps.len()).rev() {
        let j = rng.gen_range(i + 1);
        gaps.swap(i, j);
    }

    let mut out = vec![false; values.len()];
    if first >= out.len() {
        return shuffle_bool_preserving_count(values, rng);
    }
    let mut pos = first;
    out[pos] = true;
    for gap in gaps {
        pos = pos.saturating_add(gap);
        if pos >= out.len() {
            return shuffle_bool_preserving_count(values, rng);
        }
        out[pos] = true;
    }
    out
}

fn bool_index(value: bool) -> usize {
    if value { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_shuffle_preserves_count() {
        let values = [true, false, true, false, false, true];
        let mut rng = XorShift64::new(7);
        let shuffled = shuffle_bool_preserving_count(&values, &mut rng);
        assert_eq!(values.iter().filter(|v| **v).count(), shuffled.iter().filter(|v| **v).count());
    }

    #[test]
    fn markov_generators_preserve_length() {
        let values = [true, false, true, true, false, false, true, false];
        let mut rng = XorShift64::new(11);
        assert_eq!(markov1_bool_sequence(&values, &mut rng).len(), values.len());
        assert_eq!(markov2_bool_sequence(&values, &mut rng).len(), values.len());
    }
}
