use clap::Clap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Sub};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let mut numbers = File::open(opts.input)
        .map(BufReader::new)?
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| line.parse::<u8>().ok())
        .collect::<Vec<_>>();

    numbers.sort_unstable();

    let arrangements = count_arrangements(
        0,
        numbers.last().unwrap() + 3,
        &mut numbers,
        3,
        &mut HashMap::new(),
    );

    println!("arrangements: {:?}", arrangements);

    Ok(())
}

fn count_arrangements<T>(
    start: T,
    goal: T,
    numbers: &[T],
    max_difference: T,
    cache: &mut HashMap<T, usize>,
) -> usize
where
    T: Sub<Output = T> + Add<Output = T> + Copy + Ord + Hash + Display,
{
    if cache.contains_key(&start) {
        return cache[&start];
    }

    let mut arrangements = if goal - start <= max_difference { 1 } else { 0 };

    for (idx, &value) in numbers
        .iter()
        .take_while(|&&v| v - start <= max_difference)
        .enumerate()
    {
        arrangements += count_arrangements(value, goal, &numbers[idx + 1..], max_difference, cache);
    }

    cache.insert(start, arrangements);
    arrangements
}

#[cfg(test)]
mod tests {
    use crate::count_arrangements;
    use std::collections::HashMap;

    #[test]
    fn test_part2_example_a() {
        let data = r#"
            16
            10
            15
            5
            1
            11
            7
            19
            6
            12
            4
        "#;

        let mut numbers = data
            .lines()
            .map(|line| line.trim())
            .filter_map(|line| line.parse::<u8>().ok())
            .collect::<Vec<_>>();

        numbers.sort_unstable();

        let arrangements = count_arrangements(
            0,
            numbers.last().unwrap() + 3,
            &mut numbers,
            3,
            &mut HashMap::new(),
        );

        assert_eq!(8, arrangements);
    }

    #[test]
    fn test_part2_example_b() {
        let data = r#"
            28
            33
            18
            42
            31
            14
            46
            20
            48
            47
            24
            23
            49
            45
            19
            38
            39
            11
            1
            32
            25
            35
            8
            17
            7
            9
            4
            2
            34
            10
            3
        "#;

        let mut numbers = data
            .lines()
            .map(|line| line.trim())
            .filter_map(|line| line.parse::<u8>().ok())
            .collect::<Vec<_>>();

        numbers.sort_unstable();

        let arrangements = count_arrangements(
            0,
            numbers.last().unwrap() + 3,
            &mut numbers,
            3,
            &mut HashMap::new(),
        );

        assert_eq!(19208, arrangements);
    }
}
