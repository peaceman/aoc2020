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
    let bus_ids = parse(reader);
    let timestamp = find_earliest_timestamp(&bus_ids);

    println!("earliest timestamp: {}", timestamp);

    Ok(())
}

fn parse(reader: impl BufRead) -> Vec<Option<u32>> {
    let mut lines = reader.lines();
    let _ = lines.next().unwrap().unwrap().parse::<u32>().unwrap();

    lines
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .split(',')
        .map(|s| match s {
            "x" => None,
            s => Some(s.parse::<u32>().unwrap()),
        })
        .collect::<Vec<_>>()
}

fn find_earliest_timestamp(bus_ids: &[Option<u32>]) -> u64 {
    match bus_ids.len() {
        0 => return Default::default(),
        1 => return bus_ids[0].unwrap_or_default() as u64,
        _ => {}
    }

    let first_bus = bus_ids[0].unwrap() as u64;
    let mut timestamp = first_bus;
    let mut step = first_bus;

    // thanks to u/PillarsBliz for the algo
    for (idx, id) in bus_ids
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, id)| id.is_some())
        .map(|(x, y)| (x, y.unwrap()))
    {
        let mut new_ts = timestamp;

        timestamp = loop {
            new_ts += step;

            if (new_ts + idx as u64) % id as u64 == 0 {
                break new_ts;
            }
        };

        step *= id as u64;
    }

    timestamp
}

#[cfg(test)]
mod tests {
    use crate::{find_earliest_timestamp, parse};

    #[test]
    fn test_find_earliest_timestamp() {
        let data = r#"939
            7,13,x,x,59,x,31,19"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(1068781, timestamp);

        let data = r#"939
            17,x,13,19"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(3417, timestamp);

        let data = r#"939
            67,7,59,61"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(754018, timestamp);

        let data = r#"939
            67,x,7,59,61"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(779210, timestamp);

        let data = r#"939
            67,7,x,59,61"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(1261476, timestamp);

        let data = r#"939
            1789,37,47,1889"#;

        let bus_ids = parse(data.as_bytes());
        let timestamp = find_earliest_timestamp(&bus_ids);
        assert_eq!(1202161486, timestamp);
    }
}
