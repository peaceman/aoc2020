use clap::Clap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let mut yes_answers_sum = 0;
    let mut line_iter = reader.lines().peekable();

    while let Some(_) = line_iter.peek() {
        yes_answers_sum += count_yes_answers(&mut line_iter);
    }

    println!("yes answers sum: {}", yes_answers_sum);

    Ok(())
}

fn count_yes_answers<'a>(line_iter: impl Iterator<Item = Result<String, std::io::Error>>) -> usize {
    let mut answers: HashMap<char, usize> = HashMap::new();
    let mut person_count = 0;

    for line in line_iter {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    break;
                } else {
                    for c in line.chars() {
                        *answers.entry(c).or_insert(0) += 1;
                    }

                    person_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Encountered an error during line reading: {:?}", e);
            }
        }
    }

    answers.iter().filter(|(_k, v)| **v == person_count).count()
}
