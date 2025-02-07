use crate::strategy::Strategy;
use rand::rng;
use rand::seq::SliceRandom;

pub const LOWER_BOUND: i32 = -1; // Lower boundary (no number is less than 0).
pub const UPPER_BOUND: i32 = 1000; // Upper boundary (all numbers are < 1000).
pub const NUM_SLOTS: usize = 20; // The board has 20 slots.

/// A `Gap` represents a contiguous group of empty slots along with
/// the boundaries in which a number must lie.
#[derive(Debug, Clone)]
pub struct Gap {
    pub lower: i32,
    pub upper: i32,
    pub first_index: usize,
    pub last_index: usize,
}

/// The result of a single game simulation.
#[derive(Debug, Clone)]
pub struct GameResult {
    pub placed_count: usize,
}

/// Finds the gap (if any) where `number` can be legally placed given the current board.
/// Returns a `Gap` if one is found or `None` if no valid gap exists.
pub fn find_valid_gap(board: &[Option<i32>], number: i32) -> Option<Gap> {
    let mut start = 0;
    let mut end = NUM_SLOTS;
    // Find the last index that is not None and is less than number
    for (i, &val) in board.iter().enumerate() {
        if let Some(val) = val {
            match val.cmp(&number) {
                std::cmp::Ordering::Less => start = i,
                std::cmp::Ordering::Greater => {
                    end = i;
                    break;
                }
                _ => {}
            }
        }
    }

    if end - start <= 1 {
        return None;
    }

    if start == 0 && end == NUM_SLOTS {
        return Some(Gap {
            lower: LOWER_BOUND,
            upper: UPPER_BOUND,
            first_index: 0,
            last_index: NUM_SLOTS - 1,
        });
    }

    if start == 0 {
        return Some(Gap {
            lower: LOWER_BOUND,
            upper: board[end].unwrap(),
            first_index: 0,
            last_index: end - 1,
        });
    }

    if end == NUM_SLOTS {
        return Some(Gap {
            lower: board[start].unwrap(),
            upper: UPPER_BOUND,
            first_index: start + 1,
            last_index: NUM_SLOTS - 1,
        });
    }

    Some(Gap {
        lower: board[start].unwrap(),
        upper: board[end].unwrap(),
        first_index: start + 1,
        last_index: end - 1,
    })
}

/// Simulates a game for multiple strategies using the same shuffled list of numbers.
///
/// # Arguments
///
/// * `strategies` - A slice of tuples, where each tuple contains a name (for identification)
///                  and a reference to a strategy implementing the `Strategy` trait.
///
/// # Returns
///
/// A vector of tuples, each containing the strategy name and its corresponding `GameResult`.
pub fn simulate_game_multi(
    strategies: &[(String, Box<dyn Strategy>)],
) -> Vec<(String, GameResult)> {
    let mut rng = rng();
    let mut numbers: Vec<i32> = (0..1000).collect();
    numbers.shuffle(&mut rng);

    // Prepare separate boards and result trackers for each strategy.
    let mut boards: Vec<Vec<Option<i32>>> = vec![vec![None; NUM_SLOTS]; strategies.len()];
    let mut placed_counts: Vec<usize> = vec![0; strategies.len()];
    let mut completed: Vec<bool> = vec![false; strategies.len()];

    for &number in &numbers {
        for (i, (_, strategy)) in strategies.iter().enumerate() {
            // Skip strategies that have already completed.
            if completed[i] {
                continue;
            }

            if let Some(gap) = find_valid_gap(&boards[i], number) {
                let chosen_slot = if gap.first_index == gap.last_index || number == gap.lower + 1 {
                    gap.first_index
                } else if number + 1 == gap.upper {
                    gap.last_index
				} else if gap.first_index == gap.last_index - 1 {
					let dist_lower = number - gap.lower;
					let dist_upper = gap.upper - number;
					if dist_lower < dist_upper {
						gap.first_index
					} else {
						gap.last_index
					}
                } else {
                    strategy.choose_slot(
                        gap.lower,
                        gap.upper,
                        gap.first_index,
                        gap.last_index,
                        number,
                    )
                };
                boards[i][chosen_slot] = Some(number);
                placed_counts[i] += 1;

                if placed_counts[i] == NUM_SLOTS {
                    completed[i] = true; // Mark strategy as completed (win)
                }
            } else {
                // No valid gap for the number; strategy loses.
                completed[i] = true;
            }
        }

        // Early exit if all strategies have completed.
        if completed.iter().all(|&c| c) {
            break;
        }
    }

    // Gather the results for each strategy.
    strategies
        .iter()
        .enumerate()
        .map(|(i, (strategy_name, _))| {
            (
                strategy_name.clone(),
                GameResult {
                    placed_count: placed_counts[i],
                },
            )
        })
        .collect()
}

/// Runs multiple simulations for multiple strategies and tracks the placement histogram.
///
/// # Arguments
///
/// * `strategies` - A slice of tuples, where each tuple contains the strategy name and reference.
/// * `num_simulations` - The number of simulations to run.
///
/// # Returns
///
/// A vector of tuples, each containing the strategy name and its corresponding placement histogram.
pub fn run_simulations_multi(
    strategies: &[(String, Box<dyn Strategy>)],
    num_simulations: usize,
) -> Vec<(String, [usize; NUM_SLOTS + 1])> {
    // Initialize histograms for each strategy.
    let mut histograms: Vec<[usize; NUM_SLOTS + 1]> = vec![[0; NUM_SLOTS + 1]; strategies.len()];

    for _ in 0..num_simulations {
        // Run the simulation for all strategies using the same shuffled list of numbers.
        let results = simulate_game_multi(strategies);

        // Update the histogram for each strategy based on the result.
        for (i, (_strategy_name, result)) in results.iter().enumerate() {
            histograms[i][result.placed_count] += 1;
        }
    }

    // Combine the strategy names with their corresponding histograms for the final result.
    strategies
        .iter()
        .enumerate()
        .map(|(i, (strategy_name, _))| (strategy_name.clone(), histograms[i]))
        .collect()
}
