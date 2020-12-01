use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let numbers = parse_file(&opts.input)?;
    let result = find_result(numbers.as_ref(), 2020);

    match result {
        Some(v) => println!("Found result: {}", v),
        None => println!("Couldn't find result"),
    }

    Ok(())
}

fn find_result(numbers: &[u64], expected_sum: u64) -> Option<u64> {
    for x in numbers.iter() {
        for y in numbers.iter() {
            if x + y == expected_sum {
                return Some(x * y);
            }
        }
    }

    None
}

fn parse_file(path: impl AsRef<Path>) -> Result<Vec<u64>> {
    let mut file = File::open(path.as_ref()).map(BufReader::new)?;
    let mut line_buffer = String::new();
    let mut output = Vec::new();

    loop {
        let result = file.read_line(&mut line_buffer);

        if let Ok(bytes_read) = result {
            if bytes_read == 0 {
                break;
            }

            output.push(line_buffer.trim_end().parse()?);
        }

        line_buffer.clear();
    }

    Ok(output)
}
