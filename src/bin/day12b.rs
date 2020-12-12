use clap::Clap;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error as ThisError;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts: Opts = Opts::parse();

    let reader = File::open(opts.input).map(BufReader::new)?;
    let instructions = parse_instructions(reader);
    let mut ship = Ship::new(Point { x: 0, y: 0 });
    ship.follow_navigation_instructions(&instructions);

    println!(
        "ship location: {:?}, manhattan distance: {}",
        ship.location,
        ship.calc_manhattan_distance()
    );

    Ok(())
}

fn parse_instructions(reader: impl BufRead) -> Vec<NavigationInstruction> {
    reader
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|l| match l.trim().is_empty() {
            false => Some(l),
            true => None,
        })
        .filter_map(|l| match l.trim().parse::<NavigationInstruction>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!(
                    "Failed to parse navigation instruction {}, error: {:?}",
                    l, e
                );
                None
            }
        })
        .collect()
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Point<T> {
    x: T,
    y: T,
}

impl Point<i32> {
    fn rotate(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();

        Point {
            x: (self.x as f32 * c + self.y as f32 * s).round() as i32,
            y: (-(self.x as f32) * s + self.y as f32 * c).round() as i32,
        }
    }
}

struct Ship {
    location: Point<i32>,
    starting_location: Point<i32>,
    waypoint: Point<i32>,
}

impl Ship {
    fn new(starting_location: Point<i32>) -> Self {
        Self {
            location: starting_location.clone(),
            starting_location,
            waypoint: Point { x: 10, y: 1 },
        }
    }

    fn calc_manhattan_distance(&self) -> u32 {
        (self.location.x - self.starting_location.x).abs() as u32
            + (self.location.y - self.starting_location.y).abs() as u32
    }

    fn follow_navigation_instructions(&mut self, instructions: &[NavigationInstruction]) {
        instructions.iter().for_each(|v| self.navigate(v));
    }

    fn navigate(&mut self, instruction: &NavigationInstruction) {
        match &instruction.action {
            NavigationAction::Turn(dir) => {
                let angle = (i8::from(dir) as i16 * instruction.value as i16) as f32;

                self.waypoint = self.waypoint.rotate(angle.to_radians());
            }
            NavigationAction::Move(NavigationActionMove::Absolute(dir)) => {
                let multiplier = dir.as_point_offset_multiplier();

                self.waypoint.x += instruction.value as i32 * multiplier.x as i32;
                self.waypoint.y += instruction.value as i32 * multiplier.y as i32;
            }
            NavigationAction::Move(NavigationActionMove::Forward) => {
                for _ in 0..instruction.value {
                    self.location.x += self.waypoint.x;
                    self.location.y += self.waypoint.y;
                }
            }
        }
    }
}

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn as_point_offset_multiplier(&self) -> &Point<i8> {
        match self {
            Direction::North => &Point { x: 0, y: 1 },
            Direction::South => &Point { x: 0, y: -1 },
            Direction::East => &Point { x: 1, y: 0 },
            Direction::West => &Point { x: -1, y: 0 },
        }
    }
}

impl TryFrom<i16> for Direction {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        use Direction::*;

        let value = value % 360;
        let value = match value.cmp(&0) {
            Ordering::Less => value + 360,
            _ => value,
        };

        Ok(match value {
            0 => North,
            90 => East,
            180 => South,
            270 => West,
            _ => return Err(()),
        })
    }
}

impl From<&Direction> for u16 {
    fn from(v: &Direction) -> Self {
        match v {
            Direction::North => 0,
            Direction::South => 180,
            Direction::East => 90,
            Direction::West => 270,
        }
    }
}

#[derive(Debug)]
struct NavigationInstruction {
    value: u16,
    action: NavigationAction,
}

impl FromStr for NavigationInstruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            action: s.get(..1).unwrap_or("").parse()?,
            value: s.get(1..).unwrap_or("").parse()?,
        })
    }
}

#[derive(Debug)]
enum NavigationAction {
    Turn(TurnDirection),
    Move(NavigationActionMove),
}

#[derive(Debug)]
enum TurnDirection {
    Left,
    Right,
}

impl From<&TurnDirection> for i8 {
    fn from(v: &TurnDirection) -> Self {
        match v {
            TurnDirection::Left => -1,
            TurnDirection::Right => 1,
        }
    }
}

#[derive(Debug)]
enum NavigationActionMove {
    Absolute(Direction),
    Forward,
}

impl FromStr for NavigationAction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Direction::*;
        use NavigationAction::*;
        use NavigationActionMove::*;
        use TurnDirection::*;

        Ok(match s {
            "N" => Move(Absolute(North)),
            "S" => Move(Absolute(South)),
            "E" => Move(Absolute(East)),
            "W" => Move(Absolute(West)),
            "L" => Turn(Left),
            "R" => Turn(Right),
            "F" => Move(Forward),
            s => return Err(ParseError::UnrecognizedNavigationAction(String::from(s))),
        })
    }
}

#[derive(ThisError, Debug)]
enum ParseError {
    #[error("Unrecognized navigation action {0}")]
    UnrecognizedNavigationAction(String),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

#[cfg(test)]
mod tests {
    use crate::{parse_instructions, Point, Ship};

    #[test]
    fn test_navigation() {
        let data = r#"
            F10
            N3
            F7
            R90
            F11
        "#;

        let instructions = parse_instructions(data.as_bytes());
        let mut ship = Ship::new(Point::default());

        ship.follow_navigation_instructions(&instructions);
        println!("ship location: {:?}", ship.location);

        assert_eq!(286, ship.calc_manhattan_distance())
    }

    #[test]
    fn test_rotation() {
        let x: Point<i32> = Point { x: 10, y: 4 };
        let r = x.rotate(90f32.to_radians());

        assert_eq!(Point { x: 4, y: -10 }, r);
    }
}
