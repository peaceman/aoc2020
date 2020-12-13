use clap::Clap;
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
    let (arrival, bus_ids) = parse(reader);
    let (bus_id, departure) = determine_earliest_bus(&arrival, &bus_ids).unwrap();

    println!("bus id: {}, departure: {}", bus_id, departure);
    println!("{}", bus_id * (departure - arrival));

    Ok(())
}

fn parse(reader: impl BufRead) -> (u32, Vec<u32>) {
    let mut lines = reader.lines();
    let arrival = lines.next().unwrap().unwrap().parse::<u32>().unwrap();

    (
        arrival,
        lines
            .next()
            .unwrap()
            .unwrap()
            .trim()
            .split(',')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect::<Vec<_>>(),
    )
}

fn determine_earliest_bus<'a>(arrival: &u32, bus_ids: &'a [u32]) -> Option<(&'a u32, u32)> {
    bus_ids
        .iter()
        .map(|id| {
            let x = (*arrival as f32 / *id as f32).ceil() as u32;

            (id, id * x)
        })
        .min_by_key(|(_id, departure)| *departure)
}

#[cfg(test)]
mod tests {
    use crate::{determine_earliest_bus, parse};

    #[test]
    fn test_determine_earliest_bus() {
        let data = r#"939
            7,13,x,x,59,x,31,19"#;

        let (arrival, bus_ids) = parse(data.as_bytes());
        let (bus_id, departure) = determine_earliest_bus(&arrival, &bus_ids).unwrap();

        assert_eq!(295, bus_id * (departure - arrival));
    }
}
