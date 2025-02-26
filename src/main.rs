mod engine;
mod strategies;
mod strategy;

use engine::run_simulations_multi;
use std::{io::Write, sync::Arc};
use strategies::*;
use strategy::Strategy;

fn main() {
    let num_simulations = 1_000_000_000;
    let mut strategies: Vec<(String, Arc<dyn Strategy>)> = vec![
        ("FirstAvailable".to_string(), Arc::new(FirstAvailableStrategy)),
        ("LastAvailable".to_string(), Arc::new(LastAvailableStrategy)),
        ("Middle".to_string(), Arc::new(MiddleStrategy)),
        ("OptimalWin".to_string(), Arc::new(OptimalWinStrategy)),
        ("Binomial".to_string(), Arc::new(BinomialStrategy)),
        ("BinomialQuantized".to_string(), Arc::new(BinomialQuantizedStrategy)),
        ("LookupTable".to_string(), Arc::new(LookupTableStrategy::new("strategy.txt"))),
        ("LookupTableInt".to_string(), Arc::new(LookupTableStrategy::new("strategyint.txt"))),
        // ("Gaussian (σ=0.02)", Arc::new(GaussianStrategy::<20>)),
        // ("Gaussian (σ=0.05)", Arc::new(GaussianStrategy::<50>)),
        // ("Gaussian (σ=0.10)", Arc::new(GaussianStrategy::<100>)),
        // ("Gaussian (σ=0.20)", Arc::new(GaussianStrategy::<200>)),
        // ("Gaussian (σ=0.30)", Arc::new(GaussianStrategy::<300>)),
        // ("Gaussian (σ=0.40)", Arc::new(GaussianStrategy::<400>)),
        // ("Gaussian (σ=0.50)", Arc::new(GaussianStrategy::<500>)),
        // ("Gaussian (σ=0.60)", Arc::new(GaussianStrategy::<600>)),
        // ("Gaussian (σ=0.70)", Arc::new(GaussianStrategy::<700>)),
        // ("Gaussian (σ=0.80)", Arc::new(GaussianStrategy::<800>)),
        // ("Gaussian (σ=0.90)", Arc::new(GaussianStrategy::<900>)),
        // ("Gaussian (σ=1.00)", Arc::new(GaussianStrategy::<1000>)),
        // ("Gaussian (σ=2.00)", Arc::new(GaussianStrategy::<2000>)),
    ];

    // Add CautiousOptimal for 80 to 100 with a step of 5
    for i in (80..=100).step_by(5) {
        let strategy_name = format!("CautiousOptimal_{}", i);
        strategies.push((
            strategy_name.clone(),
            Arc::new(CautiousOptimalStrategy::new(i)),
        ));
    }

    let histograms = run_simulations_multi(&strategies, num_simulations);

    // Open csv output file.
    let mut file = std::fs::File::create("output.csv").unwrap();

    // Open detailed output csv file.
    let mut detailed_file = std::fs::File::create("detailed_output.csv").unwrap();

    // Write the header to the csv file.
    writeln!(
        file,
        "Strategy,Win rate (%),Average placements,Standard deviation"
    )
    .unwrap();

    // Write the header to the detailed output file which has the strategy name and histogram with one bucket per column.
    writeln!(
        detailed_file,
        "Strategy,{}",
        (0..=20)
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",")
    )
    .unwrap();

    for (strategy_name, histogram) in histograms {
        let total_placements: usize = (1..=20).map(|i| i * histogram[i]).sum();

        let win_rate = histogram[20] as f64 / num_simulations as f64;

        let avg_placements = total_placements as f64 / num_simulations as f64;

        // Calculate variance: sum of squared differences divided by num_simulations.
        let variance = (1..=20)
            .map(|i| {
                let diff = i as f64 - avg_placements;
                diff.powi(2) * histogram[i] as f64
            })
            .sum::<f64>()
            / num_simulations as f64;

        let std_deviation = variance.sqrt(); // Take square root of variance for standard deviation.

        // Write to csv file.
        writeln!(
            file,
            "{},{},{:.2},{:.4}",
            strategy_name, win_rate, avg_placements, std_deviation
        )
        .unwrap();

        // Write to detailed output file.
        writeln!(
            detailed_file,
            "{},{}",
            strategy_name,
            histogram
                .iter()
                .map(|&count| count.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
        .unwrap();
    }
    println!("Output written to output.csv");
}
