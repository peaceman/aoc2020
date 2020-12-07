use clap::Clap;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let mut data: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let reader = File::open(opts.input).map(BufReader::new)?;

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let (container, content) = {
                    let mut si = line.splitn(2, "contain");
                    (si.next().unwrap(), si.next().unwrap())
                };

                let content = content.trim().trim_end_matches('.');
                let content = match content {
                    "no other bags" => HashMap::new(),
                    _ => content
                        .split(',')
                        .map(|s| {
                            let mut si = s.trim().splitn(2, ' ');
                            (
                                si.next().unwrap().parse::<usize>().unwrap(),
                                String::from(strip_bag_suffix(si.next().unwrap())),
                            )
                        })
                        .map(|v| (v.1, v.0))
                        .collect(),
                };

                data.insert(String::from(strip_bag_suffix(container)), content);
            }
            Err(e) => {
                eprintln!("Encountered an error during line reading: {:?}", e);
                continue;
            }
        }
    }

    let inspected = "shiny gold";
    let counter = search_content(&data, &inspected);

    println!("counter: {}", counter);

    Ok(())
}

fn strip_bag_suffix(input: &str) -> &str {
    input
        .trim()
        .trim_end_matches("bags")
        .trim_end_matches("bag")
        .trim()
}

fn search_content(data: &HashMap<String, HashMap<String, usize>>, inspected: &str) -> usize {
    match data.get(inspected) {
        None => 0,
        Some(v) => v
            .iter()
            .map(|(inspected, count)| {
                let content_amount = search_content(data, inspected);
                println!("{} contains {}", inspected, content_amount);

                count + (content_amount * count)
            })
            .sum(),
    }
}
