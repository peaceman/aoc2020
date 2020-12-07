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

    let contained_by_index = {
        let mut index = HashMap::new();

        for (outer, v) in data.iter() {
            for (inner, _) in v.iter() {
                index
                    .entry(inner.clone())
                    .or_insert_with(|| Vec::new())
                    .push(outer.clone());
            }
        }

        index
    };

    let searched_bag = "shiny gold";
    let mut containers = HashSet::new();
    search_containers(&contained_by_index, searched_bag, &mut containers);

    println!("counter: {}", containers.len());

    Ok(())
}

fn strip_bag_suffix(input: &str) -> &str {
    input
        .trim()
        .trim_end_matches("bags")
        .trim_end_matches("bag")
        .trim()
}

fn search_containers(
    index: &HashMap<String, Vec<String>>,
    searched: &str,
    containers: &mut HashSet<String>,
) {
    match index.get(searched) {
        Some(v) => {
            for s in v {
                containers.insert(s.clone());
                search_containers(index, s, containers);
            }
        }
        None => {}
    }
}
