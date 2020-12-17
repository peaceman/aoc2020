use clap::Clap;
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Deref, DerefMut};
use std::time::Instant;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;
    let mut grid = Grid::new();
    parse_grid_slice(reader, &mut grid);

    let start = Instant::now();
    simulate_cycles(&mut grid, 6);

    println!(
        "active cubes: {} elapsed: {:?}",
        grid.active_count(),
        start.elapsed()
    );

    Ok(())
}

#[derive(Debug)]
struct Grid(HashMap<Position, bool>);

impl Grid {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn active_bounds(&self) -> (Position, Position) {
        let active_positions = self.iter().filter(|(_k, &v)| v).collect::<Vec<_>>();
        let min = Position {
            x: active_positions.iter().map(|(p, _)| p.x).min().unwrap(),
            y: active_positions.iter().map(|(p, _)| p.y).min().unwrap(),
            z: active_positions.iter().map(|(p, _)| p.z).min().unwrap(),
            w: active_positions.iter().map(|(p, _)| p.w).min().unwrap(),
        };

        let max = Position {
            x: active_positions.iter().map(|(p, _)| p.x).max().unwrap(),
            y: active_positions.iter().map(|(p, _)| p.y).max().unwrap(),
            z: active_positions.iter().map(|(p, _)| p.z).max().unwrap(),
            w: active_positions.iter().map(|(p, _)| p.w).max().unwrap(),
        };

        (min, max)
    }

    fn active_count(&self) -> usize {
        self.iter().filter(|(_k, &v)| v).count()
    }
}

impl Deref for Grid {
    type Target = HashMap<Position, bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (min, max) = self.active_bounds();

        // writeln!(f, "min: {:?} max: {:?}", min, max);
        // writeln!(f, "keys: {:?}", self.keys().collect::<Vec<_>>());

        for z in min.z..=max.z {
            for w in min.w..=max.w {
                writeln!(f, "z={} w={}", z, w)?;

                for y in min.y..=max.y {
                    write!(f, "y: {} ", y)?;
                    for x in min.x..=max.x {
                        write!(
                            f,
                            "{}",
                            if *self.get(&Position { x, y, z, w }).unwrap_or(&false) {
                                '#'
                            } else {
                                '.'
                            }
                        )?;
                    }

                    write!(f, "\n")?;
                }
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Default, Copy, Clone, Ord, PartialOrd)]
struct Position {
    z: i32,
    y: i32,
    x: i32,
    w: i32,
}

impl Position {
    fn neighbours(&self) -> [Position; 80] {
        let mut n = [Position::default(); 80];

        let mut idx = 0;
        for ox in -1..=1 {
            for oy in -1..=1 {
                for oz in -1..=1 {
                    for ow in -1..=1 {
                        if ox == 0 && oy == 0 && oz == 0 && ow == 0 {
                            continue;
                        }

                        n[idx] = Self {
                            x: self.x + ox,
                            y: self.y + oy,
                            z: self.z + oz,
                            w: self.w + ow,
                        };

                        idx += 1;
                    }
                }
            }
        }

        n
    }
}

fn parse_grid_slice(reader: impl BufRead, target: &mut Grid) {
    reader.lines().filter_map(|l| l.ok()).fold(0, |y, l| {
        l.trim().chars().enumerate().for_each(|(x, c)| {
            target.insert(
                Position {
                    x: x as i32,
                    y: y as i32,
                    z: 0,
                    w: 0,
                },
                c == '#',
            );
        });

        y + 1
    });
}

fn simulate_cycles(grid: &mut Grid, cycles: usize) {
    // println!("before cycles\n{}", grid);

    for cycle in 0..cycles {
        simulate_cycle(grid);
        // println!("after {} cycle/s\n{}", cycle + 1, grid);
    }
}

fn simulate_cycle(grid: &mut Grid) {
    let mut deactivate = HashSet::new();
    let mut activate = HashSet::new();

    fn active_neighbours(pos: &Position, grid: &Grid) -> usize {
        pos.neighbours()
            .iter()
            .filter_map(|np| grid.get(np))
            .filter(|&&ns| ns)
            .count()
    }

    let mut check_pos = |pos: &Position, grid: &Grid| {
        let state = *grid.get(pos).unwrap_or(&false);
        let active_nb = active_neighbours(pos, grid);

        if state {
            if active_nb != 2 && active_nb != 3 {
                deactivate.insert(*pos);
            }
        } else {
            if active_nb == 3 {
                activate.insert(*pos);
            }
        }
    };

    let (min, max) = grid.active_bounds();
    // println!("min: {:?} max: {:?}", min, max);

    for z in min.z - 1..=max.z + 1 {
        for w in min.w - 1..=max.w + 1 {
            for y in min.y - 1..=max.y + 1 {
                for x in min.x - 1..=max.x + 1 {
                    check_pos(&Position { x, y, z, w }, grid)
                }
            }
        }
    }

    for pos in deactivate {
        // println!("deactivate: {:?}", pos);
        grid.insert(pos, false);
    }

    for pos in activate {
        // println!("activate: {:?}", pos);
        grid.insert(pos, true);
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_grid_slice, simulate_cycles, Grid};

    #[test]
    fn test_simulate_cycles() {
        let data = r#"
            .#.
            ..#
            ###
        "#;

        let mut grid = Grid::new();
        parse_grid_slice(data.trim().as_bytes(), &mut grid);

        // println!("{:#?}", grid);

        simulate_cycles(&mut grid, 6);

        assert_eq!(848, grid.active_count());
    }
}
