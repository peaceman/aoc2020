use clap::Clap;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let start = Instant::now();
    let result = play_game(reader, 30_000_000);
    println!("result: {:?} elapsed: {:?}", result, start.elapsed());

    Ok(())
}

fn play_game(input: impl BufRead, rounds: usize) -> u64 {
    let mut last_number = None;
    let mut memory: HashMap<u64, usize> = HashMap::new();
    let mut round = 0;

    fn record_number(
        n: u64,
        last_number: &mut Option<u64>,
        memory: &mut HashMap<u64, usize>,
        round: &mut usize,
    ) -> u64 {
        *last_number = Some(n);

        let next_number = match memory.insert(n, *round) {
            Some(prev_round) => *round - prev_round,
            None => 0,
        } as u64;

        *round += 1;

        next_number
    };

    input
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .split(',')
        .map(|s| s.parse::<u64>().unwrap())
        .for_each(|n| {
            record_number(n, &mut last_number, &mut memory, &mut round);
        });

    let mut next_number = 0;
    loop {
        if round >= rounds {
            break;
        }

        // println!(
        //     "next_number: {} last_number: {:?} round: {}",
        //     next_number, last_number, round
        // );

        next_number = record_number(next_number, &mut last_number, &mut memory, &mut round);
    }

    last_number.unwrap()
}

#[cfg(test)]
mod tests {
    use crate::play_game;

    #[test]
    fn test_play_game() {
        let data = &[
            ("0,3,6", 30_000_000, 175594),
            // ("1,3,2", 30_000_000, 2578),
            // ("2,1,3", 30_000_000, 3544142),
            // ("1,2,3", 30_000_000, 261214),
            // ("2,3,1", 30_000_000, 6895259),
            // ("3,2,1", 30_000_000, 18),
            // ("3,1,2", 30_000_000, 362),
        ];

        for (s, rounds, expected) in data {
            assert_eq!(*expected, play_game(s.as_bytes(), *rounds as usize));
        }
    }
}
