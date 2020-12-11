use clap::Clap;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;
use std::str::{FromStr, Utf8Error};
use std::time::Instant;
use thiserror::Error as ThisError;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts: Opts = Opts::parse();

    let reader = File::open(opts.input).map(BufReader::new)?;
    let mut seat_layout = SeatLayout::parse(reader)?;

    let start = Instant::now();
    apply_seating_rules(&mut seat_layout);
    let elapsed = Instant::now().duration_since(start);

    let occupied_seats = count_occupied_seats(&seat_layout);
    println!(
        "occupied seats: {} | elapsed: {:?}",
        occupied_seats, elapsed
    );

    Ok(())
}

#[derive(Debug)]
enum GridPlace {
    Floor,
    EmptySeat,
    OccupiedSeat,
}

impl FromStr for GridPlace {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "." => Self::Floor,
            "L" => Self::EmptySeat,
            "#" => Self::OccupiedSeat,
            s => return Err(ParseError::InvalidGridPlace(String::from(s))),
        })
    }
}

impl fmt::Display for GridPlace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Floor => ".",
                Self::EmptySeat => "L",
                Self::OccupiedSeat => "#",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn is_negative(&self) -> bool {
        self.x.is_negative() || self.y.is_negative()
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

struct SeatLayout {
    line_length: usize,
    data: Vec<GridPlace>,
}

impl SeatLayout {
    fn parse(reader: impl BufRead) -> Result<Self, ParseError> {
        let mut data = Vec::new();
        let mut line_length = None;

        for line in reader.lines() {
            if let Err(e) = line {
                return Err(e.into());
            }

            let line = line.unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            for char in line.chars() {
                data.push(
                    std::str::from_utf8(&[char as u8])
                        .map_err(|e| e.into())
                        .and_then(|s| s.parse())?,
                );
            }

            if line_length.is_none() {
                line_length = Some(line.chars().count());
            }
        }

        Ok(Self {
            line_length: line_length.unwrap_or_default(),
            data,
        })
    }

    fn get_position(&self, coords: &Point) -> Option<&GridPlace> {
        if coords.is_negative() {
            return None;
        }

        let ll: i32 = self.line_length as i32;

        if coords.x >= ll {
            None
        } else {
            let idx = (coords.y * ll) + (coords.x % ll);

            self.data.get(idx as usize)
        }
    }

    fn get_position_mut(&mut self, coords: &Point) -> Option<&mut GridPlace> {
        if coords.is_negative() {
            return None;
        }

        let ll: i32 = self.line_length as i32;

        if coords.x >= ll {
            None
        } else {
            let idx = (coords.y * ll) + (coords.x % ll);

            self.data.get_mut(idx as usize)
        }
    }

    fn get_visible_seats(&self, coords: &Point) -> Vec<&GridPlace> {
        let mut result = Vec::with_capacity(8);
        let directions = [
            &(-1, -1),
            &(0, -1),
            &(1, -1),
            &(-1, 0),
            &(1, 0),
            &(-1, 1),
            &(0, 1),
            &(1, 1),
        ];

        for (x, y) in directions.iter() {
            let mut pos = coords.clone();

            let seat = loop {
                pos = &pos + &Point { x: *x, y: *y };

                match self.get_position(&pos) {
                    Some(GridPlace::Floor) => continue,
                    Some(v) => break Some(v),
                    None => break None,
                }
            };

            if let Some(place) = seat {
                result.push(place);
            }
        }

        result
    }

    fn iter(&self) -> SeatLayoutIter {
        SeatLayoutIter::new(self)
    }
}

impl fmt::Display for SeatLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.data.chunks(self.line_length) {
            for place in line {
                write!(f, "{}", place)?;
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}

struct SeatLayoutIter<'a> {
    seat_layout: &'a SeatLayout,
    idx: usize,
}

impl<'a> SeatLayoutIter<'a> {
    fn new(seat_layout: &'a SeatLayout) -> Self {
        Self {
            seat_layout,
            idx: 0,
        }
    }
}

impl<'a> Iterator for SeatLayoutIter<'a> {
    type Item = (Point, &'a GridPlace);

    fn next(&mut self) -> Option<Self::Item> {
        if self.seat_layout.line_length == 0 {
            return None;
        }

        let place = self.seat_layout.data.get(self.idx)?;
        let pos = Point {
            x: (self.idx % self.seat_layout.line_length) as i32,
            y: (self.idx / self.seat_layout.line_length) as i32,
        };

        self.idx += 1;

        Some((pos, place))
    }
}

fn apply_seating_rules(seat_layout: &mut SeatLayout) {
    loop {
        let changeset = seat_layout
            .iter()
            .filter_map(|(pos, place)| {
                let visible_seats = seat_layout.get_visible_seats(&pos);
                let visible_occupied = visible_seats
                    .iter()
                    .filter(|v| matches!(v, GridPlace::OccupiedSeat))
                    .count();

                let new_place = match place {
                    GridPlace::EmptySeat if visible_occupied == 0 => GridPlace::OccupiedSeat,
                    GridPlace::OccupiedSeat if visible_occupied >= 5 => GridPlace::EmptySeat,
                    _ => return None,
                };

                Some((pos, new_place))
            })
            .collect::<Vec<_>>();

        if changeset.is_empty() {
            break;
        }

        for (pos, place) in changeset {
            *seat_layout.get_position_mut(&pos).unwrap() = place;
        }

        // println!("{}", seat_layout);
    }
}

fn count_occupied_seats(seat_layout: &SeatLayout) -> usize {
    seat_layout
        .iter()
        .filter(|(_pos, place)| matches!(place, GridPlace::OccupiedSeat))
        .count()
}

#[derive(ThisError, Debug)]
enum ParseError {
    #[error("invalid grid place: {0}")]
    InvalidGridPlace(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    EncodingError(#[from] Utf8Error),
}

#[cfg(test)]
mod test {
    use crate::{apply_seating_rules, count_occupied_seats, GridPlace, Point, SeatLayout};

    #[test]
    fn test_apply_seating_rules() {
        let data = r#"
            L.LL.LL.LL
            LLLLLLL.LL
            L.L.L..L..
            LLLL.LL.LL
            L.LL.LL.LL
            L.LLLLL.LL
            ..L.L.....
            LLLLLLLLLL
            L.LLLLLL.L
            L.LLLLL.LL
        "#;

        let mut seat_layout = SeatLayout::parse(data.as_bytes()).unwrap();
        apply_seating_rules(&mut seat_layout);

        let occupied_seats = count_occupied_seats(&seat_layout);
        assert_eq!(26, occupied_seats);
    }
}
