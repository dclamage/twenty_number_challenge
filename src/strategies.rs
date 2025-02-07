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

pub struct CautiousOptimalStrategy
{
	epsilon_percent: usize
}

impl CautiousOptimalStrategy
{
	pub fn new(epsilon_percent: usize) -> Self
	{
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
