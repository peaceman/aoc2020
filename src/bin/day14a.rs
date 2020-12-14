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
    zeros: u64,
    ones: u64,
}

impl FromStr for Bitmask {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut zeros = 0;
        let mut ones = 0;

        for c in s.trim().chars() {
            zeros <<= 1;
            ones <<= 1;

            match c {
                '1' => ones |= 0b1,
                '0' => zeros |= 0b1,
                'X' => {}
                _ => return Err(()),
            }
        }

        Ok(Self {
            zeros: zeros,
            ones: ones,
        })
    }
}

impl Bitmask {
    fn apply(&self, value: u64) -> u64 {
        (value & !self.zeros) | self.ones
    }
}

fn execute(reader: impl BufRead) -> u64 {
    let mut mem = HashMap::new();
    let mut mask = Bitmask::default();

    reader.lines().filter_map(|l| l.ok()).for_each(|l| {
        let l = l.trim();

        if let Ok((string_mask)) = scan_fmt!(l, "mask = {}", String) {
            mask = string_mask.parse().unwrap();
            return;
        }

        if let Ok((address, value)) = scan_fmt!(l, "mem[{d}] = {d}", u64, u64) {
            mem.insert(address, mask.apply(value));
        }
    });

    mem.values().sum()
}

#[cfg(test)]
mod tests {
    use crate::{execute, Bitmask};

    #[test]
    fn test_bitmask() {
        let data = [
            (11, "XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X", 73),
            (101, "XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X", 101),
            (0, "XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X", 64),
        ];

        for (value, mask, result) in &data {
            let bitmask = mask.parse::<Bitmask>().unwrap();

            assert_eq!(*result, bitmask.apply(*value));
        }
    }

    #[test]
    fn test_execute() {
        let data = r#"
            mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
            mem[8] = 11
            mem[7] = 101
            mem[8] = 0
        "#;

        let result = execute(data.as_bytes());
        assert_eq!(165, result);
    }
}
