use clap::Clap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Sub};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let mut numbers = File::open(opts.input)
        .map(BufReader::new)?
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| line.parse::<u8>().ok())
        .collect::<Vec<_>>();

    let distribution = joltage_difference_distribution(&mut numbers, 0, 3, 3);
    println!("distribution: {:?}", distribution);
    println!("multiplied: {}", distribution[&1] * distribution[&3]);

    Ok(())
}

fn joltage_difference_distribution<T>(
    numbers: &mut [T],
    input_joltage: T,
    max_difference: T,
    device_adapter_difference: T,
) -> HashMap<T, usize>
where
    T: Sub<Output = T> + Add<Output = T> + Copy + Ord + Hash + Display,
{
    numbers.sort_unstable();

    let mut distribution = HashMap::new();
    let mut prev = input_joltage;

    for curr in numbers {
        let diff = *curr - prev;
        if diff > max_difference {
            panic!("Difference is greater than 3, {}", diff);
        }

        *distribution.entry(diff).or_default() += 1;
        prev = *curr;
    }

    // device adapter
    *distribution.entry(device_adapter_difference).or_default() += 1;

    distribution
}

#[cfg(test)]
mod tests {
    use crate::joltage_difference_distribution;

    #[test]
    fn test_part1_example_a() {
        let data = r#"
            16
            10
            15
            5
            1
            11
            7
            19
            6
            12
            4
        "#;

        let mut numbers = data
            .lines()
            .map(|line| line.trim())
            .filter_map(|line| line.parse::<u8>().ok())
            .collect::<Vec<_>>();

        let distribution = joltage_difference_distribution(&mut numbers, 0, 3, 3);

        assert_eq!(7, distribution[&1]);
        assert_eq!(5, distribution[&3]);
    }

    #[test]
    fn test_part1_example_b() {
        let data = r#"
            28
            33
            18
            42
            31
            14
            46
            20
            48
            47
            24
            23
            49
            45
            19
            38
            39
            11
            1
            32
            25
            35
            8
            17
            7
            9
            4
            2
            34
            10
            3
        "#;

        let mut numbers = data
            .lines()
            .map(|line| line.trim())
            .filter_map(|line| line.parse::<u8>().ok())
            .collect::<Vec<_>>();

        let distribution = joltage_difference_distribution(&mut numbers, 0, 3, 3);

        assert_eq!(22, distribution[&1]);
        assert_eq!(10, distribution[&3]);
    }
}
