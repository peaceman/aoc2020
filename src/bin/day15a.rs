use clap::Clap;
use std::collections::{HashMap, VecDeque};
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let result = play_game(reader, 2020);
    println!("result: {:?}", result);

    Ok(())
}

fn play_game(input: impl BufRead, rounds: usize) -> u64 {
    let mut last_number = None;
    let mut memory: HashMap<u64, VecDeque<usize>> = HashMap::new();
    let mut round = 0;

    fn record_number(
        n: u64,
        last_number: &mut Option<u64>,
        memory: &mut HashMap<u64, VecDeque<usize>>,
        round: &mut usize,
    ) {
        *last_number = Some(n);

        let q = memory.entry(n).or_default();
        q.push_back(*round);

        if q.len() == 3 {
            q.pop_front();
        }

        *round += 1;
    };

    input
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .split(',')
        .map(|s| s.parse::<u64>().unwrap())
        .for_each(|n| record_number(n, &mut last_number, &mut memory, &mut round));

    loop {
        if round >= rounds {
            break;
        }

        let q = &memory[last_number.as_ref().unwrap()];
        let next_number = match q.len() {
            1 => 0,
            2 => q[1] - q[0],
            _ => unreachable!(),
        } as u64;

        println!(
            "next_number: {} last_number: {:?} round: {}",
            next_number, last_number, round
        );

        record_number(next_number, &mut last_number, &mut memory, &mut round);
    }

    last_number.unwrap()
}

#[cfg(test)]
mod tests {
    use crate::play_game;

    #[test]
    fn test_play_game() {
        let data = &[
            ("1,3,2", 2020, 1),
            ("2,1,3", 2020, 10),
            ("1,2,3", 2020, 27),
            ("2,3,1", 2020, 78),
            ("3,2,1", 2020, 438),
            ("3,1,2", 2020, 1836),
        ];

        for (s, rounds, expected) in data {
            assert_eq!(*expected, play_game(s.as_bytes(), *rounds as usize));
        }
    }
}
