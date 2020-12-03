use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, AddAssign};
use thiserror::Error as ThisError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let map = {
        let file = File::open(opts.input)?;
        let file_size = file.metadata()?.len();
        TreeMap::parse(BufReader::new(file), file_size)
    }?;

    let slope = Point { x: 3, y: 1 };
    let mut current_position = Point { x: 0, y: 0 };

    let mut tree_counter = 0;
    while let Ok(is_tree) = map.get_position(&current_position) {
        if *is_tree {
            tree_counter += 1;
        }

        current_position += slope;
    }

    println!("Encountered {} trees", tree_counter);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

struct TreeMap {
    lines: usize,
    line_length: usize,
    data: Vec<bool>,
}

impl TreeMap {
    fn parse(mut reader: impl BufRead, input_length: u64) -> Result<Self> {
        let mut data = Vec::with_capacity(input_length as usize);
        let mut line_buffer = String::new();
        let mut line_length = None;
        let mut lines: usize = 0;

        loop {
            let line = reader.read_line(&mut line_buffer);
            match line {
                Ok(0) => break,
                Ok(bytes_read) => {
                    if line_length.is_none() {
                        line_length = Some(bytes_read - 1);
                    }

                    for char in line_buffer.trim_end().chars() {
                        data.push(match char {
                            '#' => true,
                            _ => false,
                        });
                    }

                    lines += 1;
                    line_buffer.clear();
                }
                Err(e) => {
                    eprintln!("Encountered an error during tree map file parsing: {:?}", e);
                    line_buffer.clear()
                }
            }
        }

        Ok(TreeMap {
            line_length: line_length.ok_or(TreeMapError::MissingLineLength)?,
            lines,
            data,
        })
    }

    fn get_position(&self, coords: &Point) -> Result<&bool> {
        if coords.y >= self.lines {
            return Err(TreeMapError::EndOfMap.into());
        }

        let idx = (coords.y * self.line_length) + (coords.x % self.line_length);

        println!("checking idx: {} for coords: {:?}", idx, coords);

        self.data.get(idx).ok_or(TreeMapError::EndOfMap.into())
    }
}

#[derive(ThisError, Debug)]
enum TreeMapError {
    #[error("Reached end of map")]
    EndOfMap,
    #[error("Failed to determine line length")]
    MissingLineLength,
}
