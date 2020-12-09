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
    let start = Instant::now();
    let invalid_number = find_invalid_number(&numbers, preamble_length);
    println!(
        "invalid number: {}, elapsed: {:?}",
        invalid_number,
        Instant::now().duration_since(start)
    );

    let start = Instant::now();
    let encryption_weakness = find_encryption_weakness(&numbers, invalid_number).unwrap();
    println!(
        "encryption weakness: {}, elapsed: {:?}",
        encryption_weakness,
        Instant::now().duration_since(start)
    );

    Ok(())
}

fn find_invalid_number(numbers: &[u64], preamble_length: usize) -> u64 {
    for idx_num in preamble_length..numbers.len() {
        let num = numbers[idx_num];
        let mut number_is_valid = false;

        for idx_a in (idx_num - preamble_length)..idx_num {
            let a = numbers[idx_a];

            if numbers[(idx_a + 1)..idx_num].iter().any(|b| a + b == num) {
                number_is_valid = true;
                break;
            }
        }

        if !number_is_valid {
            return num;
        }
    }

    return 0;
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
