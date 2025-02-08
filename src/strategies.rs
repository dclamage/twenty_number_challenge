use std::collections::HashMap;
use std::hash::Hash;
use std::io::BufRead;

use crate::strategy::Strategy;
use libm::erf;

// A strategy that picks the mathematically optimal slot for winning.
pub struct OptimalWinStrategy;

impl Strategy for OptimalWinStrategy {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        let num_slots = last_slot - first_slot + 1;

        assert!(
            num_slots >= 3,
            "OptimalWinStrategy requires at least 3 slots"
        );

        // Proportional mapping to slot index
        let slot_index = (((number - lower) as f64 * num_slots as f64) / (upper - lower) as f64)
            .floor() as usize;

        // Clamp to ensure index is within bounds
        let slot_index = slot_index.min(num_slots - 1);

        first_slot + slot_index
    }
}

pub struct CautiousOptimalStrategy {
    epsilon_percent: usize,
}

impl CautiousOptimalStrategy {
    pub fn new(epsilon_percent: usize) -> Self {
        Self { epsilon_percent }
    }
}

impl Strategy for CautiousOptimalStrategy {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        // Determine the number of available slots.
        let num_slots = last_slot - first_slot + 1;

        assert!(
            num_slots >= 3,
            "CautiousOptimalStrategy requires at least 3 slots"
        );

        // Calculate slot size (range each slot would naturally cover).
        let slot_size = (upper - lower) as f64 / num_slots as f64;

        // Convert EPSILON_PERCENT into a fraction (e.g., 5% becomes 0.05).
        const SCALE: f64 = 100.0; // Fixed-point scale factor.
        let epsilon_fraction = self.epsilon_percent as f64 / SCALE;

        // Determine epsilon threshold based on slot size.
        let epsilon_threshold = epsilon_fraction * slot_size;

        // Check if the number is within epsilon_threshold of the lower or upper bound.
        if ((number - lower) as f64) < epsilon_threshold {
            first_slot
        } else if ((upper - number) as f64) < epsilon_threshold {
            last_slot
        } else {
            // Proportionally place within the inner slots.
            let adjusted_fraction = ((number as f64 - lower as f64) - epsilon_threshold)
                / ((upper as f64 - lower as f64) - 2.0 * epsilon_threshold);
            let inner_slot = (adjusted_fraction * (num_slots as f64 - 2.0)).floor() as usize;
            first_slot + 1 + inner_slot.min(num_slots - 3)
        }
    }
}

/// A strategy that uses a Gaussian distribution to bias slot selection.
pub struct GaussianStrategy<const SIGMA_FIXED: usize>;

impl<const SIGMA_FIXED: usize> Strategy for GaussianStrategy<SIGMA_FIXED> {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        let num_slots = last_slot - first_slot + 1;
        // We assume there are at least 3 slots available.
        assert!(num_slots >= 3, "GaussianStrategy requires at least 3 slots");

        // Convert SIGMA_FIXED (fixed-point) to a floating-point sigma.
        const SCALE: f64 = 1000.0;
        let sigma = SIGMA_FIXED as f64 / SCALE;

        // Normalize the number to a fraction between 0 and 1.
        let fraction = (number - lower) as f64 / (upper - lower) as f64;

        // Compute the normal CDF for a distribution centered at 0.5.
        // That is, new_fraction = 0.5*(1 + erf((fraction - 0.5) / (sigma * sqrt(2)))).
        let z = (fraction - 0.5) / (sigma * f64::sqrt(2.0));
        let new_fraction = 0.5 * (1.0 + erf(z));

        // Map the new_fraction to a slot index.
        let slot_index = (new_fraction * (num_slots - 1) as f64).round() as usize;
        first_slot + slot_index.min(num_slots - 1)
    }
}

pub struct BinomialStrategy;

impl BinomialStrategy {
    /// Computes the binomial coefficient "n choose k" as a floating-point number.
    fn binom(n: usize, k: usize) -> f64 {
        if k > n {
            return 0.0;
        }
        let mut result = 1.0;
        for i in 0..k {
            result *= (n - i) as f64 / (i + 1) as f64;
        }
        result
    }
}

impl Strategy for BinomialStrategy {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        // Determine the number of available slots in the current gap.
        let num_slots = last_slot - first_slot + 1;
        // We require at least 3 slots for a meaningful decision.
        assert!(num_slots >= 3, "BinomialStrategy requires at least 3 slots");

        // Normalize the drawn number to [0,1] assuming a continuous approximation.
        let x = (number - lower) as f64 / (upper - lower) as f64;
        // There are (num_slots - 1) positions left after placing the current number.
        let remaining = num_slots - 1;

        // Initialize the best candidate.
        let mut best_k = 0;
        let mut best_prob = -1.0;

        // Evaluate each possible slot position (0 = leftmost, num_slots-1 = rightmost).
        for k in 0..num_slots {
            let prob = Self::binom(remaining, k)
                * x.powi(k as i32)
                * (1.0 - x).powi((remaining - k) as i32);
            if prob > best_prob {
                best_prob = prob;
                best_k = k;
            }
        }

        // Return the actual slot index.
        first_slot + best_k
    }
}

pub struct BinomialQuantizedStrategy;

impl BinomialQuantizedStrategy {
    /// Computes the binomial coefficient "n choose k" as a floating-point number.
    fn binom(n: usize, k: usize) -> f64 {
        if k > n {
            return 0.0;
        }
        let mut result = 1.0;
        for i in 0..k {
            result *= (n - i) as f64 / (i + 1) as f64;
        }
        result
    }
}

impl Strategy for BinomialQuantizedStrategy {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        // Determine the number of available slots in the current gap.
        let num_slots = last_slot - first_slot + 1;
        // Require at least 3 slots for a meaningful decision.
        assert!(
            num_slots >= 3,
            "BinomialQuantizedStrategy requires at least 3 slots"
        );

        // Calculate the number of available discrete numbers on either side.
        // These adjustments (subtracting 1) correct for the drawn number and boundaries.
        let num_options_lower = (number - lower) as usize - 1;
        let num_options_upper = (upper - number) as usize - 1;

        // After placing the drawn number, there are `remaining` positions to be filled.
        let remaining = num_slots - 1;

        let mut best_k = 0;
        let mut best_likelihood = -1.0;

        // For each possible candidate slot (0-indexed within the gap),
        // compute the likelihood that the drawn number is the (k+1)th smallest
        // among all numbers destined for this gap.
        for k in 0..num_slots {
            // Likelihood is proportional to:
            // binom(num_options_lower, k) * binom(num_options_upper, remaining - k)
            let likelihood =
                Self::binom(num_options_lower, k) * Self::binom(num_options_upper, remaining - k);
            if likelihood > best_likelihood {
                best_likelihood = likelihood;
                best_k = k;
            }
        }

        // Return the absolute slot index.
        first_slot + best_k
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LookupKey {
    num_slots: usize,
    num_values: usize,
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    upper_bound: i32,
    placement_index: usize,
}

pub struct LookupTableStrategy {
    // Map from (num_slots, num_values) to a sorted vector of candidate entries.
    table: HashMap<LookupKey, Vec<Candidate>>,
}

impl LookupTableStrategy {
    pub fn new(file_path: &str) -> Self {
        let mut raw_entries = Vec::new();
        let file = std::fs::File::open(file_path).unwrap();
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split_whitespace().collect();
            let num_slots = parts[0].parse().unwrap();
            let num_values = parts[1].parse().unwrap();
            let placement_index = parts[2].parse().unwrap();
            let upper_bound: i32 = parts[3].parse().unwrap();
            raw_entries.push((num_slots, num_values, placement_index, upper_bound));
        }
        // Build the map.
        let mut table: HashMap<LookupKey, Vec<Candidate>> = HashMap::new();
        for (num_slots, num_values, placement_index, upper_bound) in raw_entries {
            let key = LookupKey {
                num_slots,
                num_values,
            };
            table.entry(key).or_default().push(Candidate {
                upper_bound,
                placement_index,
            });
        }
        // Sort each vector by upper_bound.
        for vec in table.values_mut() {
            vec.sort_by_key(|candidate| candidate.upper_bound);
        }
        Self { table }
    }
}

impl Strategy for LookupTableStrategy {
    fn want_full_control(&self) -> bool {
        true
    }

    fn choose_slot(
        &self,
        lower: i32, // gap lower boundary (exclusive)
        upper: i32, // gap upper boundary (exclusive)
        first_slot: usize,
        last_slot: usize,
        number: i32,
        _current_board: &[Option<i32>],
    ) -> usize {
        let gap_slots = last_slot - first_slot + 1;
        let gap_values = (upper - lower - 1) as usize; // because lower and upper are exclusive.
        let offset = number - lower - 1; // 0-indexed offset in the gap.

        let key = LookupKey {
            num_slots: gap_slots,
            num_values: gap_values,
        };

        // Get the block of candidates for these parameters.
        let candidates = self.table.get(&key).unwrap_or_else(|| {
            panic!(
                "No lookup table entry for gap with {} slots and {} values",
                gap_slots, gap_values
            )
        });

        // Find the best candidate that fits the offset.
        let mut best_candidate_index = 0usize;
        for (i, candidate) in candidates.iter().enumerate() {
            best_candidate_index = i;
            if candidate.upper_bound >= offset {
                break;
            }
        }

        // Return the absolute slot index.
        first_slot + candidates[best_candidate_index].placement_index
    }
}
