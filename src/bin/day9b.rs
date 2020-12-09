use clap::Clap;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Range;
use std::time::Instant;

#[derive(Clap)]
struct Opts {
    input: String,
    preamble_length: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let numbers = File::open(opts.input)
        .map(BufReader::new)?
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty())
        .filter_map(|line| match line.parse::<u64>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("Failed to parse line: {:?}", e);
                None
            }
        })
        .collect::<Vec<_>>();

    let preamble_length = opts.preamble_length;
    let invalid_number = find_invalid_number(&numbers, preamble_length);
    let encryption_weakness = find_encryption_weakness(&numbers, invalid_number);

    println!("encryption weakness: {:?}", encryption_weakness);

    Ok(())
}

fn find_invalid_number(numbers: &[u64], preamble_length: usize) -> u64 {
    let start = Instant::now();
    let sums = calculate_permutation_sums(&numbers);

    let mut num = 0;
    for idx in preamble_length..numbers.len() {
        num = numbers[idx];

        if !is_number_valid(num, idx - preamble_length, &sums) {
            break;
        }
    }

    println!(
        "num: {:?}, elapsed: {:?}",
        num,
        Instant::now().duration_since(start)
    );

    num
}

fn calculate_permutation_sums(numbers: &[u64]) -> HashMap<u64, HashSet<(usize, usize)>> {
    let mut sums = HashMap::new();

    for (idx_a, v_a) in numbers.iter().enumerate() {
        for (idx_b, v_b) in numbers.iter().enumerate() {
            // skip same values
            if v_a == v_b {
                continue;
            }

            let sum = v_a + v_b;
            sums.entry(sum)
                .or_insert_with(HashSet::new)
                .insert((idx_a, idx_b));
        }
    }

    sums
}

fn is_number_valid(
    num: u64,
    first_valid_index: usize,
    sums: &HashMap<u64, HashSet<(usize, usize)>>,
) -> bool {
    sums.get(&num)
        .map(|sources| {
            let valid_range = first_valid_index..;
            sources
                .iter()
                .any(|indices| valid_range.contains(&indices.0) && valid_range.contains(&indices.1))
        })
        .unwrap_or(false)
}

fn find_encryption_weakness(numbers: &[u64], invalid_number: u64) -> Option<u64> {
    for (idx_a, &v_a) in numbers.iter().enumerate() {
        let mut current_sum = v_a;
        let mut sum_operands = vec![v_a];

        for v_b in numbers[idx_a + 1..].iter() {
            current_sum += *v_b;

            match current_sum.cmp(&invalid_number) {
                Ordering::Less => {
                    sum_operands.push(*v_b);
                }
                Ordering::Greater => break,
                Ordering::Equal => {
                    sum_operands.sort_unstable();
                    return Some(sum_operands.first().unwrap() + sum_operands.last().unwrap());
                }
            }
        }
    }

    None
}
