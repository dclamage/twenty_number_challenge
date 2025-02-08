use std::sync::Arc;

use crate::strategy::Strategy;
use rand::{rng, Rng};
use rayon::prelude::*;

pub const LOWER_BOUND: i32 = -1; // Lower boundary (no number is less than 0).
pub const UPPER_BOUND: i32 = 999 + 1; // Upper boundary (all numbers are < UPPER_BOUND).
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
    let mut have_start = false;
    let mut have_end = false;
    let mut start = 0;
    let mut end = NUM_SLOTS;
    // Find the last index that is not None and is less than number
    for (i, &val) in board.iter().enumerate() {
        if let Some(val) = val {
            match val.cmp(&number) {
                std::cmp::Ordering::Less => {
                    start = i;
                    have_start = true;
                }
                std::cmp::Ordering::Greater => {
                    end = i;
                    have_end = true;
                    break;
                }
                _ => {}
            }
        }
    }

    // No start and no end means the board is empty.
    if !have_start && !have_end {
        return Some(Gap {
            lower: LOWER_BOUND,
            upper: UPPER_BOUND,
            first_index: 0,
            last_index: NUM_SLOTS - 1,
        });
    }

    // No start means the number is less than the first number on the board.
    if !have_start {
        if end == 0 {
            return None;
        }

        return Some(Gap {
            lower: LOWER_BOUND,
            upper: board[end].unwrap(),
            first_index: 0,
            last_index: end - 1,
        });
    }

    // No end means the number is greater than the last number on the board.
    if !have_end {
        if start == NUM_SLOTS - 1 {
            return None;
        }

        return Some(Gap {
            lower: board[start].unwrap(),
            upper: UPPER_BOUND,
            first_index: start + 1,
            last_index: NUM_SLOTS - 1,
        });
    }

    if end - start <= 1 {
        return None;
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
    strategies: &[(String, Arc<dyn Strategy>)],
) -> Vec<(String, GameResult)> {
    let mut rng = rng();

    // Generate 20 random numbers been 0 and UPPER_BOUND without replacement
    let mut numbers = Vec::new();
    while numbers.len() < NUM_SLOTS {
        let number = rng.random_range(0..UPPER_BOUND);
        if !numbers.contains(&number) {
            numbers.push(number);
        }
    }

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
                let chosen_slot = if strategy.want_full_control() {
                    strategy.choose_slot(
                        gap.lower,
                        gap.upper,
                        gap.first_index,
                        gap.last_index,
                        number,
                        &boards[i],
                    )
                } else if gap.first_index == gap.last_index || number == gap.lower + 1 {
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
                        &boards[i],
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
    strategies: &[(String, Arc<dyn Strategy>)],
    num_simulations: usize,
) -> Vec<(String, [usize; NUM_SLOTS + 1])> {
    // Clone the Arc pointers (cheap, reference count increases).
    let strategies: Vec<(String, Arc<dyn Strategy>)> = strategies.to_vec();

    let num_strategies = strategies.len();

    let histograms = (0..num_simulations)
        .into_par_iter()
        .map(|_| {
            let results = simulate_game_multi(&strategies);
            let mut local_hist = vec![[0; NUM_SLOTS + 1]; num_strategies];
            for (i, (_name, result)) in results.iter().enumerate() {
                local_hist[i][result.placed_count] += 1;
            }
            local_hist
        })
        .reduce(
            || vec![[0; NUM_SLOTS + 1]; num_strategies],
            |mut acc, local_hist| {
                for (i, hist) in local_hist.into_iter().enumerate() {
                    for (slot, count) in hist.into_iter().enumerate() {
                        acc[i][slot] += count;
                    }
                }
                acc
            },
        );

    strategies
        .into_iter()
        .enumerate()
        .map(|(i, (name, _))| (name, histograms[i]))
        .collect()
}
