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
    let occurrences: u32 = line
        .password
        .chars()
        .filter(|c| *c == line.character)
        .count() as u32;

    return line.occurrences.contains(&occurrences);
}
