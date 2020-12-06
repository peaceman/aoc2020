use clap::Clap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let mut unique_yes_answers: HashSet<char> = HashSet::new();
    let mut yes_answers_sum = 0;

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    yes_answers_sum += unique_yes_answers.len();
                    unique_yes_answers.clear();
                } else {
                    for c in line.chars() {
                        unique_yes_answers.insert(c);
                    }
                }
            }
            Err(e) => {
                eprintln!("Encountered an error during line reading: {:?}", e);
            }
        }
    }

    yes_answers_sum += unique_yes_answers.len();
    unique_yes_answers.clear();

    println!("yes answers sum: {}", yes_answers_sum);

    Ok(())
}
