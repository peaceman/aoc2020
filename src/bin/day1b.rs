use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Matcher<'a> = Box<dyn Fn(Option<&[u64]>, u64) -> Option<u64> + 'a>;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let numbers = parse_file(&opts.input)?;

    let func = gen_matcher(numbers.as_slice(), 3);
    let result = func(None, 2020);

    match result {
        Some(v) => println!("Found result: {}", v),
        None => println!("Couldn't find result"),
    }

    Ok(())
}

fn gen_matcher(numbers: &[u64], recursion_level: u64) -> Matcher {
    let mut func = None;

    for _ in 0..recursion_level {
        let closure = Box::new({
            let func = func.take();
            move |current_numbers: Option<&[u64]>, expected_result: u64| -> Option<u64> {
                find_match(numbers, expected_result, current_numbers, func.as_ref())
            }
        });

        func = Some(closure);
    }

    func.unwrap()
}

fn find_match(
    input_numbers: &[u64],
    expected_result: u64,
    current_numbers: Option<&[u64]>,
    matcher: Option<&Matcher>,
) -> Option<u64> {
    for number in input_numbers {
        let mut current_numbers = current_numbers.map_or_else(Vec::new, |v| v.to_vec());
        current_numbers.push(number.clone());

        match matcher {
            Some(matcher) => {
                let matcher_result = matcher(Some(&current_numbers), expected_result);
                if matcher_result.is_some() {
                    return matcher_result;
                }
            }
            None => {
                if current_numbers.iter().sum::<u64>() == expected_result {
                    return Some(current_numbers.iter().fold(1, |acc, x| acc * x));
                }
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
