use crate::boarding_pass::Seat;
use clap::Clap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts: Opts = Opts::parse();

    let reader = File::open(opts.input).map(BufReader::new)?;

    let mut seats = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| match line.parse() {
            Ok(seat) => Some(seat),
            Err(e) => {
                eprintln!("Failed to parse boarding pass: {} error: {:?}", line, e);
                None
            }
        })
        .collect::<Vec<Seat>>();

    seats.sort_by_key(|s| s.id);

    let missing_seat_id = {
        let mut last_seat_id = None;
        let mut seat_iter = seats.iter();

        loop {
            match seat_iter.next() {
                Some(seat) => {
                    match last_seat_id {
                        None => {}
                        Some(last_seat_id) => {
                            if last_seat_id + 1 != seat.id {
                                break Some(last_seat_id + 1);
                            }
                        }
                    }

                    last_seat_id = Some(seat.id)
                }
                None => break None,
            }
        }
    };

    println!("highest seat id: {:?}", seats.last().unwrap().id);
    println!("missing seat id: {:?}", missing_seat_id);

    Ok(())
}

mod boarding_pass {
    use std::str::FromStr;
    use thiserror::Error as ThisError;

    #[derive(Debug, PartialEq)]
    pub struct Seat {
        pub row: u8,
        pub column: u8,
        pub id: u32,
    }

    impl FromStr for Seat {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            decode(s)
        }
    }

    fn decode(input: &str) -> Result<Seat, Error> {
        let expected_input_length = Part::Row.get_str_len() + Part::Column.get_str_len();
        if input.len() != expected_input_length {
            return Err(Error::InvalidInputLength { given: input.len() });
        }

        let row = decode_part(&input[0..Part::Row.get_str_len()], &Part::Row)?;
        let column = decode_part(&input[Part::Row.get_str_len()..], &Part::Column)?;
        Ok(Seat {
            row,
            column,
            id: (row as u32 * 8 + column as u32),
        })
    }

    #[derive(Debug)]
    pub struct PartEncoding {
        low: char,
        high: char,
    }

    enum Part {
        Column,
        Row,
    }

    impl Part {
        fn get_str_len(&self) -> usize {
            match self {
                Part::Column => 3,
                Part::Row => 7,
            }
        }

        fn get_encoding(&self) -> &'static PartEncoding {
            match self {
                Part::Row => &PartEncoding {
                    low: 'F',
                    high: 'B',
                },
                Part::Column => &PartEncoding {
                    low: 'L',
                    high: 'R',
                },
            }
        }

        fn get_max_value(&self) -> u8 {
            match self {
                Part::Column => 8,
                Part::Row => 128,
            }
        }
    }

    fn decode_part(input: &str, part: &Part) -> Result<u8, Error> {
        let mut range = (1, part.get_max_value());
        let encoding = part.get_encoding();

        for c in input.chars() {
            let mid = range.0 + ((range.1 - range.0) / 2);
            if c == encoding.low {
                range.1 = mid;
            } else if c == encoding.high {
                range.0 = mid + 1;
            } else {
                Err(Error::UnexpectedCharacter {
                    given: c,
                    expected: encoding,
                })?;
            }
        }

        Ok(range.0 - 1)
    }

    #[derive(ThisError, Debug)]
    pub enum Error {
        #[error(
            "Encountered an unexpected character during decoding {given} expected: {expected:?}"
        )]
        UnexpectedCharacter {
            given: char,
            expected: &'static PartEncoding,
        },
        #[error("Got an input with invalid length: {given}")]
        InvalidInputLength { given: usize },
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_decode_part() -> std::result::Result<(), Box<dyn std::error::Error>> {
            assert_eq!(44, decode_part("FBFBBFF", &Part::Row)?);
            assert_eq!(5, decode_part("RLR", &Part::Column)?);
            assert_eq!(70, decode_part("BFFFBBF", &Part::Row)?);
            assert_eq!(7, decode_part("RRR", &Part::Column)?);

            Ok(())
        }

        #[test]
        fn test_parse_seat() -> std::result::Result<(), Box<dyn std::error::Error>> {
            assert_eq!(
                Seat {
                    row: 70,
                    column: 7,
                    id: 567
                },
                decode("BFFFBBFRRR")?
            );

            assert_eq!(
                Seat {
                    row: 14,
                    column: 7,
                    id: 119
                },
                decode("FFFBBBFRRR")?
            );

            assert_eq!(
                Seat {
                    row: 102,
                    column: 4,
                    id: 820
                },
                decode("BBFFBBFRLL")?
            );

            Ok(())
        }
    }
}
