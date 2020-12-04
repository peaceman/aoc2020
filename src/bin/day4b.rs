use crate::PassportParseError::ValidationFailure;
use clap::Clap;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::RangeInclusive;
use std::str::FromStr;
use thiserror::Error as ThisError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let reader = File::open(&opts.input).map(BufReader::new)?;
    let passports = parse_passports(reader);

    println!("parsed {} passports", passports.len());

    Ok(())
}

fn parse_passports(mut reader: impl BufRead) -> Vec<Passport> {
    let mut passports = Vec::new();
    let mut line_buffer = String::new();

    let mut data: Option<HashMap<String, String>> = None;

    fn option_to_passport(
        passports: &mut Vec<Passport>,
        option: &mut Option<HashMap<String, String>>,
    ) {
        if let Some(passport) = option.take().and_then(|kv| {
            kv.try_into()
                .map_err(|e| {
                    eprintln!("conversion error: {:?}", e);
                    e
                })
                .ok()
        }) {
            passports.push(passport);
        }
    }

    loop {
        let read_result = reader.read_line(&mut line_buffer);
        match read_result {
            Err(e) => {
                eprintln!("Encountered an error during line reading: {:?}", e);
                data = None;
            }
            Ok(0) => {
                option_to_passport(&mut passports, &mut data);
                break;
            }
            Ok(_) => {
                if line_buffer.trim().is_empty() {
                    option_to_passport(&mut passports, &mut data);
                }

                parse_kv_line_into_map(&line_buffer, data.get_or_insert_with(HashMap::new))
            }
        }

        line_buffer.clear();
    }

    passports
}

fn parse_kv_line_into_map(line: &str, data: &mut HashMap<String, String>) {
    for kv in line.split_ascii_whitespace() {
        parse_kv_into_map(kv, data)
    }
}

fn parse_kv_into_map(kv: &str, data: &mut HashMap<String, String>) {
    let mut it = kv.split(':');
    let kv = (it.next(), it.next());

    match kv {
        (Some(key), Some(value)) => {
            data.insert(String::from(key), String::from(value));
        }
        _ => {}
    };
}

struct Passport {
    birth_year: BirthYear,
    issue_year: IssueYear,
    expiration_year: ExpirationYear,
    height: Height,
    hair_color: HairColor,
    eye_color: EyeColor,
    passport_id: PassportId,
    country_id: Option<String>,
}

impl TryFrom<HashMap<String, String>> for Passport {
    type Error = PassportParseError;

    fn try_from(mut value: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        use PassportParseError::MissingRequiredField;

        fn from_str<T>(
            values: &mut HashMap<String, String>,
            field_name: &'static str,
        ) -> std::result::Result<T, PassportParseError>
        where
            T: FromStr<Err = String>,
        {
            values
                .remove(field_name)
                .ok_or(MissingRequiredField(field_name))
                .and_then(|s| {
                    s.parse().map_err(|e| ValidationFailure {
                        field: field_name,
                        message: e,
                    })
                })
        }

        Ok(Self {
            birth_year: from_str(&mut value, "byr")?,
            issue_year: from_str(&mut value, "iyr")?,
            expiration_year: from_str(&mut value, "eyr")?,
            height: from_str(&mut value, "hgt")?,
            hair_color: from_str(&mut value, "hcl")?,
            eye_color: from_str(&mut value, "ecl")?,
            passport_id: from_str(&mut value, "pid")?,
            country_id: value.remove("cid"),
        })
    }
}

pub struct BirthYear(u32);

impl FromStr for BirthYear {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_RANGE: RangeInclusive<u32> = 1920..=2002;

        s.parse::<u32>()
            .map_err(|e| e.to_string())
            .and_then(|year| match VALID_RANGE.contains(&year) {
                true => Ok(year),
                false => Err(format!(
                    "Given year {} is not within the expected range {:?}",
                    year, VALID_RANGE
                )),
            })
            .map(|year| BirthYear(year))
    }
}

pub struct IssueYear(u32);

impl FromStr for IssueYear {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_RANGE: RangeInclusive<u32> = 2010..=2020;

        s.parse::<u32>()
            .map_err(|e| e.to_string())
            .and_then(|year| match VALID_RANGE.contains(&year) {
                true => Ok(year),
                false => Err(format!(
                    "Given year {} is not within the expected range {:?}",
                    year, VALID_RANGE
                )),
            })
            .map(|year| IssueYear(year))
    }
}

pub struct ExpirationYear(u32);

impl FromStr for ExpirationYear {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_RANGE: RangeInclusive<u32> = 2020..=2030;

        s.parse::<u32>()
            .map_err(|e| e.to_string())
            .and_then(|year| match VALID_RANGE.contains(&year) {
                true => Ok(year),
                false => Err(format!(
                    "Given year {} is not within the expected range {:?}",
                    year, VALID_RANGE
                )),
            })
            .map(|year| ExpirationYear(year))
    }
}

pub enum Height {
    Cm(HeightCm),
    In(HeightIn),
}

impl FromStr for Height {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.ends_with("in") {
            Ok(s.strip_suffix("in").unwrap().parse::<HeightIn>()?.into())
        } else if s.ends_with("cm") {
            Ok(s.strip_suffix("cm").unwrap().parse::<HeightCm>()?.into())
        } else {
            Err(format!("Failed to detect measurement unit {}", s))
        }
    }
}

impl From<HeightCm> for Height {
    fn from(v: HeightCm) -> Self {
        Self::Cm(v)
    }
}

impl From<HeightIn> for Height {
    fn from(v: HeightIn) -> Self {
        Self::In(v)
    }
}

pub struct HeightCm(u32);

impl FromStr for HeightCm {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_RANGE: RangeInclusive<u32> = 150..=193;

        s.parse::<u32>()
            .map_err(|e| e.to_string())
            .and_then(|number| match VALID_RANGE.contains(&number) {
                true => Ok(number),
                false => Err(format!(
                    "Given number {} is not within the expected range {:?}",
                    number, VALID_RANGE,
                )),
            })
            .map(|number| Self(number))
    }
}

pub struct HeightIn(u32);

impl FromStr for HeightIn {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_RANGE: RangeInclusive<u32> = 59..=76;

        s.parse::<u32>()
            .map_err(|e| e.to_string())
            .and_then(|number| match VALID_RANGE.contains(&number) {
                true => Ok(number),
                false => Err(format!(
                    "Given number {} is not within the expected range {:?}",
                    number, VALID_RANGE,
                )),
            })
            .map(|number| Self(number))
    }
}

pub struct HairColor(String);

impl FromStr for HairColor {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        static VALID_LETTERS: RangeInclusive<char> = 'a'..='z';
        static VALID_NUMBERS: RangeInclusive<char> = '0'..='9';

        s.strip_prefix("#")
            .ok_or_else(|| format!("Missing # prefix in: {}", s))
            .and_then(|s| {
                for c in s.chars() {
                    if !VALID_LETTERS.contains(&c) && !VALID_NUMBERS.contains(&c) {
                        return Err(format!("Invalid character {}", c));
                    }
                }

                Ok(s)
            })
            .map(|s| Self(format!("#{}", s)))
    }
}

pub enum EyeColor {
    Amb,
    Blu,
    Brn,
    Gry,
    Grn,
    Hzl,
    Oth,
}

impl FromStr for EyeColor {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use EyeColor::*;

        Ok(match s {
            "amb" => Amb,
            "blu" => Blu,
            "brn" => Brn,
            "gry" => Gry,
            "grn" => Grn,
            "hzl" => Hzl,
            "oth" => Oth,
            _ => Err(format!("Unknown eye color: {}", s))?,
        })
    }
}

pub struct PassportId(String);

impl FromStr for PassportId {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.len() != 9 {
            Err(format!(
                "Invalid passport id length, expected 9 got {}",
                s.len()
            ))?;
        }

        let invalid_chars: Vec<char> = s.chars().filter(|c| !('0'..='9').contains(c)).collect();
        if !invalid_chars.is_empty() {
            Err(format!("Found invalid characters: {:?}", invalid_chars))?
        }

        Ok(Self(String::from(s)))
    }
}

#[derive(ThisError, Debug)]
enum PassportParseError {
    #[error("Missing required field {0}")]
    MissingRequiredField(&'static str),
    #[error("Failed validation for field {field}: {message}")]
    ValidationFailure {
        field: &'static str,
        message: String,
    },
}
