use clap::Clap;
use scan_fmt::scan_fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::RangeInclusive;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clap)]
struct Opts {
    input: String,
}

#[derive(Debug)]
struct LineData {
    occurrences: RangeInclusive<u32>,
    character: char,
    password: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let reader = File::open(&opts.input).map(BufReader::new)?;
    let valid_passwords = reader
        .lines()
        .filter_map(|line| {
            line.ok()
                .and_then(|line| scan_fmt!(&line, "{d}-{d} {}: {}", u32, u32, char, String).ok())
                .map(|line| LineData {
                    occurrences: line.0..=line.1,
                    character: line.2,
                    password: line.3,
                })
        })
        .filter(passes_policy)
        .map(|l| println!("valid password: {:?}", l))
        .count();

    println!("valid passwords: {}", valid_passwords);

    Ok(())
}

fn passes_policy(line: &LineData) -> bool {
    let idx_a = line.occurrences.start() - 1;
    let idx_b = line.occurrences.end() - 1;

    char_at_position_matches(&line.password, idx_a as usize, &line.character)
        ^ char_at_position_matches(&line.password, idx_b as usize, &line.character)
}

fn char_at_position_matches(text: &str, idx: usize, char: &char) -> bool {
    text.chars().nth(idx).map(|c| c == *char).unwrap_or(false)
}
