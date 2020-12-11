use clap::Clap;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    x: usize,
    y: usize,
}

struct SeatLayout {
    line_length: usize,
    data: Vec<GridPlace>,
}

impl SeatLayout {
    fn parse(mut reader: impl BufRead) -> Result<Self, ParseError> {
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
        if coords.x >= self.line_length {
            None
        } else {
            let idx = (coords.y * self.line_length) + (coords.x % self.line_length);

            self.data.get(idx)
        }
    }

    fn get_position_mut(&mut self, coords: &Point) -> Option<&mut GridPlace> {
        if coords.x >= self.line_length {
            None
        } else {
            let idx = (coords.y * self.line_length) + (coords.x % self.line_length);

            self.data.get_mut(idx)
        }
    }

    fn get_adjacent_positions(&self, coords: &Point) -> Vec<&GridPlace> {
        let mut result = Vec::with_capacity(8);

        for offset_y in -1..=1_i8 {
            for offset_x in -1..=1_i8 {
                let x = if offset_x.is_positive() {
                    coords.x.checked_add(offset_x as usize)
                } else {
                    coords.x.checked_sub(offset_x.abs() as usize)
                };

                let y = if offset_y.is_positive() {
                    coords.y.checked_add(offset_y as usize)
                } else {
                    coords.y.checked_sub(offset_y.abs() as usize)
                };

                match (x, y) {
                    (Some(x), Some(y)) => {
                        let pos = &Point { x, y };

                        if pos != coords {
                            if let Some(place) = self.get_position(&Point { x, y }) {
                                // println!("coords x: {:?}, y: {:?}", x, y);
                                result.push(place);
                            }
                        }
                    }
                    _ => {}
                }
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
            x: self.idx % self.seat_layout.line_length,
            y: self.idx / self.seat_layout.line_length,
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
                let adjacent_positions = seat_layout.get_adjacent_positions(&pos);
                let adjacent_occupied = adjacent_positions
                    .iter()
                    .filter(|v| matches!(v, GridPlace::OccupiedSeat))
                    .count();

                let new_place = match place {
                    GridPlace::EmptySeat if adjacent_occupied == 0 => GridPlace::OccupiedSeat,
                    GridPlace::OccupiedSeat if adjacent_occupied >= 4 => GridPlace::EmptySeat,
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
        assert_eq!(37, occupied_seats);
    }
}
