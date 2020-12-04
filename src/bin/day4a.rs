use clap::Clap;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    birth_year: String,
    issue_year: String,
    expiration_year: String,
    height: String,
    hair_color: String,
    eye_color: String,
    passport_id: String,
    country_id: Option<String>,
}

impl TryFrom<HashMap<String, String>> for Passport {
    type Error = PassportParseError;

    fn try_from(mut value: HashMap<String, String>) -> std::result::Result<Self, Self::Error> {
        use PassportParseError::MissingRequiredField;

        Ok(Self {
            birth_year: value.remove("byr").ok_or(MissingRequiredField("byr"))?,
            issue_year: value.remove("iyr").ok_or(MissingRequiredField("iyr"))?,
            expiration_year: value.remove("eyr").ok_or(MissingRequiredField("eyr"))?,
            height: value.remove("hgt").ok_or(MissingRequiredField("hgt"))?,
            hair_color: value.remove("hcl").ok_or(MissingRequiredField("hcl"))?,
            eye_color: value.remove("ecl").ok_or(MissingRequiredField("ecl"))?,
            passport_id: value.remove("pid").ok_or(MissingRequiredField("pid"))?,
            country_id: value.remove("cid"),
        })
    }
}

#[derive(ThisError, Debug)]
enum PassportParseError {
    #[error("Missing required field {0}")]
    MissingRequiredField(&'static str),
}
