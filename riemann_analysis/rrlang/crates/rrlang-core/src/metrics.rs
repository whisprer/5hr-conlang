use crate::nulls::{generate_bool_null, BoolNullKind, XorShift64};
use crate::primes::{is_prime_number, prime_sieve};
use crate::types::{Alert, EncodingKind, EventAnalysis, EventMap, MetricComparison, MetricObservation};
use std::collections::HashMap;

const TAUS: [f64; 9] = [1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 14.134_725, 21.022_040, 25.010_858];

pub fn shannon_entropy_labels(labels: &[String]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for label in labels {
        *counts.entry(label.as_str()).or_insert(0) += 1;
    }
    entropy_from_counts(counts.values().copied(), labels.len())
}

pub fn analyse_event_map(
    encoding: EncodingKind,
    event_map: &EventMap,
    null_samples: usize,
    seed: u64,
) -> EventAnalysis {
    let values = &event_map.values;
    let event_count = values.iter().filter(|value| **value).count();
    let mut observed = Vec::new();
    observed.push(MetricObservation {
        name: "event_density".to_string(),
        value: density(values),
        details: "count(event=true) / sequence_length".to_string(),
    });
    observed.push(MetricObservation {
        name: "event_count".to_string(),
        value: event_count as f64,
        details: "Number of true positions in the event map.".to_string(),
    });
    observed.push(MetricObservation {
        name: "sequence_length".to_string(),
        value: values.len() as f64,
        details: "Number of positions in the event map.".to_string(),
    });
    for modulus in [2usize, 3, 5, 7] {
        observed.push(MetricObservation {
            name: format!("modular_residue_peak_m_{modulus}"),
            value: modular_residue_peak(values, modulus),
            details: format!("Largest event-position residue fraction modulo {modulus}."),
        });
    }

    let metric_names = [
        "gap_entropy",
        "run_entropy",
        "prime_index_occupancy_bias",
        "prime_gap_affinity",
        "zeta_spectral_coherence",
        "critical_line_symmetry",
    ];

    let null_kinds = select_null_models(event_map, values, event_count);
    let mut comparisons = Vec::new();

    for null_kind in null_kinds {
        let mut null_values_by_metric: HashMap<&str, Vec<f64>> = HashMap::new();
        for metric_name in metric_names {
            null_values_by_metric.insert(metric_name, Vec::with_capacity(null_samples));
        }

        let mut rng = XorShift64::new(seed ^ ((null_kind as u64 + 1) << 8));
        for _ in 0..null_samples {
            let generated = generate_bool_null(values, null_kind, &mut rng);
            for metric_name in metric_names {
                let score = compute_bool_metric(metric_name, &generated);
                if let Some(bucket) = null_values_by_metric.get_mut(metric_name) {
                    bucket.push(score);
                }
            }
        }

        for metric_name in metric_names {
            let observed_value = compute_bool_metric(metric_name, values);
            let null_values = null_values_by_metric.remove(metric_name).unwrap_or_default();
            comparisons.push(compare_to_null(metric_name, null_kind.as_str(), observed_value, &null_values));
        }
    }

    let mut alerts = build_event_alerts(encoding, values, event_count, &comparisons);
    if null_samples < 100 {
        alerts.push(Alert::new(
            "LOW_NULL_SAMPLE_COUNT",
            "warning",
            1,
            format!("Only {null_samples} null samples were used. Treat p-values and z-scores as exploratory."),
        ));
    }

    EventAnalysis {
        event_name: event_map.name.clone(),
        description: event_map.description.clone(),
        event_count,
        observed,
        comparisons,
        alerts,
    }
}

fn select_null_models(event_map: &EventMap, values: &[bool], event_count: usize) -> Vec<BoolNullKind> {
    let mut models = vec![
        BoolNullKind::DensityShuffle,
        BoolNullKind::Markov1,
        BoolNullKind::Markov2,
    ];
    if values.len() >= 16 && event_count >= 3 && event_count + 3 <= values.len() {
        models.push(BoolNullKind::GapOrderShuffle);
    }
    if event_map.name == "word_boundary" {
        // The gap-order null is the MVP word-length/order control for whitespace-derived boundary maps.
    }
    models
}

fn build_event_alerts(
    encoding: EncodingKind,
    values: &[bool],
    event_count: usize,
    comparisons: &[MetricComparison],
) -> Vec<Alert> {
    let mut alerts = Vec::new();
    if values.len() < 128 {
        alerts.push(Alert::new(
            "WINDOW_TOO_SMALL",
            "warning",
            0,
            "The event map has fewer than 128 positions; positional and spectral metrics may be unstable.",
        ));
    }
    if event_count < 5 {
        alerts.push(Alert::new(
            "LOW_EVENT_COUNT",
            "warning",
            0,
            "The event occurs fewer than five times; gap and prime-position metrics are weak evidence.",
        ));
    }
    if event_count == 0 || event_count == values.len() {
        alerts.push(Alert::new(
            "DEGENERATE_EVENT_MAP",
            "warning",
            0,
            "The event map is all false or all true; many metrics collapse to trivial values.",
        ));
    }

    let mut survived_markov2 = false;
    for comparison in comparisons {
        if comparison.z_score.abs() >= 3.0 && comparison.null_std > 0.0 {
            alerts.push(Alert::new(
                "STATISTICAL_DEVIATION",
                "info",
                null_interpretation_level(&comparison.null_model),
                format!(
                    "Metric '{}' deviates from '{}' null with z={:.3}. This is a tripwire, not a conclusion.",
                    comparison.name, comparison.null_model, comparison.z_score
                ),
            ));
            if encoding == EncodingKind::Utf8Bits {
                alerts.push(Alert::new(
                    "RAW_LAYER_FINDING_NOT_LINGUISTIC",
                    "warning",
                    0,
                    "This deviation occurred in raw UTF-8 bits. Treat it as encoding-level unless it survives linguistic encodings.",
                ));
            }
            if comparison.null_model == "markov_2" {
                survived_markov2 = true;
            }
        }
        if comparison.name == "run_entropy" && comparison.z_score <= -3.0 && comparison.null_std > 0.0 {
            alerts.push(Alert::new(
                "OVER_REGULARITY_WARNING",
                "warning",
                null_interpretation_level(&comparison.null_model),
                format!(
                    "Run entropy is substantially lower than the '{}' null expectation. Possible causes include genuine structure, repeated boilerplate, encoding artefact, formal constraint, or local transition structure not captured by this null.",
                    comparison.null_model
                ),
            ));
        }
        if comparison.name == "gap_entropy" && comparison.z_score <= -3.0 && comparison.null_std > 0.0 {
            alerts.push(Alert::new(
                "PERIODICITY_OR_CLUSTERING_WARNING",
                "warning",
                null_interpretation_level(&comparison.null_model),
                format!(
                    "Gap entropy is substantially lower than the '{}' null expectation. Inspect for periodicity, clustering, repeated material, genre constraints, or local phonotactic regularity.",
                    comparison.null_model
                ),
            ));
        }
    }

    if survived_markov2 {
        alerts.push(Alert::new(
            "SURVIVES_MARKOV_2_NULL",
            "info",
            3,
            "At least one metric remains deviant under the Markov-2 null. This is stronger than density-only evidence, but still requires corpus and encoding replication.",
        ));
    }

    alerts
}

fn null_interpretation_level(null_model: &str) -> u8 {
    match null_model {
        "density_shuffle" => 1,
        "markov_1" => 2,
        "markov_2" => 3,
        "gap_order_shuffle" => 2,
        _ => 1,
    }
}

fn compare_to_null(metric_name: &str, null_model: &str, observed: f64, null_values: &[f64]) -> MetricComparison {
    let null_mean = mean(null_values);
    let null_std = stddev(null_values, null_mean);
    let z_score = if null_std > 0.0 { (observed - null_mean) / null_std } else { 0.0 };
    let diff = (observed - null_mean).abs();
    let extreme_count = null_values
        .iter()
        .filter(|value| (**value - null_mean).abs() >= diff)
        .count();
    let empirical_p = if null_values.is_empty() {
        1.0
    } else {
        (extreme_count as f64 + 1.0) / (null_values.len() as f64 + 1.0)
    };
    MetricComparison {
        name: metric_name.to_string(),
        null_model: null_model.to_string(),
        observed,
        null_mean,
        null_std,
        z_score,
        empirical_p,
        null_samples: null_values.len(),
    }
}

fn compute_bool_metric(metric_name: &str, values: &[bool]) -> f64 {
    match metric_name {
        "gap_entropy" => gap_entropy(values),
        "run_entropy" => run_entropy(values),
        "prime_index_occupancy_bias" => prime_index_occupancy_bias(values),
        "prime_gap_affinity" => prime_gap_affinity(values),
        "zeta_spectral_coherence" => zeta_spectral_coherence(values),
        "critical_line_symmetry" => critical_line_symmetry(values),
        _ => 0.0,
    }
}

pub fn density(values: &[bool]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().filter(|value| **value).count() as f64 / values.len() as f64
}

pub fn gap_entropy(values: &[bool]) -> f64 {
    let positions = event_positions(values);
    if positions.len() < 2 {
        return 0.0;
    }
    let gaps = positions.windows(2).map(|pair| pair[1] - pair[0]).collect::<Vec<_>>();
    entropy_usize(&gaps)
}

pub fn run_entropy(values: &[bool]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut runs = Vec::new();
    let mut current = values[0];
    let mut length = 1usize;
    for value in &values[1..] {
        if *value == current {
            length += 1;
        } else {
            runs.push(length);
            current = *value;
            length = 1;
        }
    }
    runs.push(length);
    entropy_usize(&runs)
}

pub fn prime_index_occupancy_bias(values: &[bool]) -> f64 {
    if values.len() < 3 {
        return 0.0;
    }
    let primes = prime_sieve(values.len());
    let mut prime_true = 0usize;
    let mut prime_total = 0usize;
    let mut nonprime_true = 0usize;
    let mut nonprime_total = 0usize;

    for (zero_index, value) in values.iter().enumerate() {
        let one_index = zero_index + 1;
        if one_index == 1 {
            continue;
        }
        if primes[one_index] {
            prime_total += 1;
            if *value {
                prime_true += 1;
            }
        } else {
            nonprime_total += 1;
            if *value {
                nonprime_true += 1;
            }
        }
    }

    let prime_density = if prime_total == 0 { 0.0 } else { prime_true as f64 / prime_total as f64 };
    let nonprime_density = if nonprime_total == 0 { 0.0 } else { nonprime_true as f64 / nonprime_total as f64 };
    prime_density - nonprime_density
}

pub fn prime_gap_affinity(values: &[bool]) -> f64 {
    let positions = event_positions(values);
    if positions.len() < 2 {
        return 0.0;
    }
    let gaps = positions.windows(2).map(|pair| pair[1] - pair[0]).collect::<Vec<_>>();
    let prime_gaps = gaps.iter().filter(|gap| is_prime_number(**gap)).count();
    prime_gaps as f64 / gaps.len() as f64
}

pub fn modular_residue_peak(values: &[bool], modulus: usize) -> f64 {
    if modulus == 0 {
        return 0.0;
    }
    let positions = event_positions(values);
    if positions.is_empty() {
        return 0.0;
    }
    let mut buckets = vec![0usize; modulus];
    for position in positions {
        buckets[position % modulus] += 1;
    }
    buckets.into_iter().max().unwrap_or(0) as f64 / event_positions(values).len() as f64
}

pub fn zeta_spectral_coherence(values: &[bool]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for tau in TAUS {
        let z = zeta_like_transform(values, 0.5, tau);
        total += z.norm_squared();
    }
    total / TAUS.len() as f64 / values.len() as f64
}

pub fn critical_line_symmetry(values: &[bool]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let eps = 0.05;
    let delta = 1e-12;
    let mut total_asymmetry = 0.0;
    for tau in TAUS {
        let left = zeta_like_transform(values, 0.5 - eps, tau).norm();
        let right = zeta_like_transform(values, 0.5 + eps, tau).norm();
        total_asymmetry += (left - right).abs() / (left + right + delta);
    }
    let mean_asymmetry = total_asymmetry / TAUS.len() as f64;
    (1.0 - mean_asymmetry).clamp(0.0, 1.0)
}

fn zeta_like_transform(values: &[bool], sigma: f64, tau: f64) -> Complex64 {
    let mu = density(values);
    let mut sum = Complex64::zero();
    for (zero_index, value) in values.iter().enumerate() {
        let index = (zero_index + 1) as f64;
        let y = if *value { 1.0 - mu } else { -mu };
        let log_i = index.ln();
        let amp = index.powf(-sigma);
        let angle = -tau * log_i;
        let factor = Complex64::new(amp * angle.cos(), amp * angle.sin());
        sum = sum + factor.scale(y);
    }
    sum
}

#[derive(Debug, Clone, Copy)]
struct Complex64 {
    re: f64,
    im: f64,
}

impl Complex64 {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    fn scale(self, factor: f64) -> Self {
        Self {
            re: self.re * factor,
            im: self.im * factor,
        }
    }

    fn norm_squared(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }
}

impl std::ops::Add for Complex64 {
    type Output = Complex64;

    fn add(self, rhs: Self) -> Self::Output {
        Complex64 {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

fn event_positions(values: &[bool]) -> Vec<usize> {
    values
        .iter()
        .enumerate()
        .filter_map(|(zero_index, value)| if *value { Some(zero_index + 1) } else { None })
        .collect()
}

fn entropy_usize(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for value in values {
        *counts.entry(*value).or_insert(0) += 1;
    }
    entropy_from_counts(counts.values().copied(), values.len())
}

fn entropy_from_counts<I>(counts: I, total: usize) -> f64
where
    I: IntoIterator<Item = usize>,
{
    if total == 0 {
        return 0.0;
    }
    counts
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let p = count as f64 / total as f64;
            -p * p.log2()
        })
        .sum()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn stddev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|value| {
            let diff = *value - mean;
            diff * diff
        })
        .sum::<f64>()
        / (values.len() as f64 - 1.0);
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_works() {
        assert!((density(&[true, false, true, false]) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn prime_gap_affinity_works() {
        let values = [true, false, true, false, false, true];
        let score = prime_gap_affinity(&values);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn critical_line_score_is_bounded() {
        let values = [true, false, false, true, false, true, true, false];
        let score = critical_line_symmetry(&values);
        assert!((0.0..=1.0).contains(&score));
    }
}
