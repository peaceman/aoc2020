use clap::Clap;

use scan_fmt::scan_fmt;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let result = execute(reader);
    println!("result: {:?}", result);

    Ok(())
}

#[derive(Debug, Default)]
struct Bitmask {
    floating: Vec<u32>,
    ones: u64,
}

impl FromStr for Bitmask {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut ones = 0u64;
        let bitlen = s.len();
        let mut floating = Vec::new();

        for (idx, c) in s.trim().chars().enumerate() {
            ones <<= 1;

            match c {
                '1' => ones |= 0b1,
                '0' => {}
                'X' => floating.push((bitlen - idx - 1) as u32),
                _ => return Err(()),
            }
        }

        Ok(Self {
            floating: floating,
            ones: ones,
        })
    }
}

impl Bitmask {
    fn apply(&self, value: u64) -> Vec<u64> {
        let value = value | self.ones;

        fn apply_floating(value: u64, floating: &[u32], output: &mut Vec<u64>) {
            if floating.is_empty() {
                return;
            }

            let pos = floating[0];

            // floating variation forced one
            let forced_one = value | (0b1 << pos);
            // floating variation forced zero
            let forced_zero = value & !(0b1 << pos);

            if floating.len() == 1 {
                output.push(forced_one);
                output.push(forced_zero);
            } else {
                apply_floating(forced_one, &floating[1..], output);
                apply_floating(forced_zero, &floating[1..], output);
            }
        }

        let mut output = Vec::new();
        apply_floating(value, &self.floating, &mut output);

        output
    }
}

fn execute(reader: impl BufRead) -> u64 {
    let mut mem = HashMap::new();
    let mut mask = Bitmask::default();

    reader.lines().filter_map(|l| l.ok()).for_each(|l| {
        let l = l.trim();

        if let Ok(string_mask) = scan_fmt!(l, "mask = {}", String) {
            mask = string_mask.parse().unwrap();
            return;
        }

        if let Ok((address, value)) = scan_fmt!(l, "mem[{d}] = {d}", u64, u64) {
            for address in mask.apply(address) {
                mem.insert(address, value);
            }
        }
    });

    mem.values().sum()
}

#[cfg(test)]
mod tests {
    use crate::{execute, Bitmask};

    #[test]
    fn test_bitmask() {
        let data: &[(u64, &str, &[u64])] = &[
            (
                42,
                "000000000000000000000000000000X1001X",
                &[26, 27, 58, 59],
            ),
            (
                26,
                "00000000000000000000000000000000X0XX",
                &[16, 17, 18, 19, 24, 25, 26, 27],
            ),
        ];

        for (value, mask, expected_result) in data {
            let bitmask = mask.parse::<Bitmask>().unwrap();
            println!("{:?}", bitmask);

            let mut result = bitmask.apply(*value);
            result.sort();

            assert_eq!(*expected_result, result);
        }
    }

    #[test]
    fn test_execute() {
        let data = r#"
            mask = 000000000000000000000000000000X1001X
            mem[42] = 100
            mask = 00000000000000000000000000000000X0XX
            mem[26] = 1
        "#;

        let result = execute(data.as_bytes());
        assert_eq!(208, result);
    }
}
