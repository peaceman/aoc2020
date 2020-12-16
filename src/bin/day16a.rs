use clap::Clap;
use scan_fmt::scan_fmt;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::RangeInclusive;

#[derive(Clap)]
struct Opts {
    input: String,
}

type FieldRules = HashMap<String, [RangeInclusive<u32>; 2]>;

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;
    let (field_rules, my_ticket, nearby_tickets) = parse(reader);

    println!("field_rules: {:#?}", field_rules);
    println!("my_ticket: {:#?}", my_ticket);
    println!("nearby_tickets: {:#?}", nearby_tickets);

    println!(
        "ticket scanning error rate: {:?}",
        determine_ticket_scanning_error_rate(&field_rules, &nearby_tickets)
    );

    Ok(())
}

fn parse(reader: impl BufRead) -> (FieldRules, Vec<u32>, Vec<Vec<u32>>) {
    enum State {
        Rules,
        MyTicket,
        NearbyTickets,
    }

    let mut state = State::Rules;

    reader.lines().filter_map(|l| l.ok()).fold(
        (HashMap::new(), Vec::new(), Vec::new()),
        |(mut rules, mut my_ticket, mut nearby_tickets), line| {
            let line = line.trim();
            match state {
                _ if line.is_empty() => {}
                _ if line.starts_with("your ticket") => {
                    state = State::MyTicket;
                }
                _ if line.starts_with("nearby tickets") => {
                    state = State::NearbyTickets;
                }
                State::Rules => {
                    if let Ok((class, from_a, to_a, from_b, to_b)) = scan_fmt!(
                        &line,
                        "{[^:]}: {d}-{d} or {d}-{d}",
                        String,
                        u32,
                        u32,
                        u32,
                        u32
                    ) {
                        rules.insert(
                            class,
                            [
                                RangeInclusive::new(from_a, to_a),
                                RangeInclusive::new(from_b, to_b),
                            ],
                        );
                    }
                }
                State::MyTicket | State::NearbyTickets => {
                    let ticket: Vec<u32> = line
                        .split(',')
                        .map(|s| s.parse::<u32>())
                        .collect::<Result<Vec<u32>, _>>()
                        .unwrap();

                    match state {
                        State::MyTicket => {
                            std::mem::replace(&mut my_ticket, ticket);
                        }
                        State::NearbyTickets => nearby_tickets.push(ticket),
                        _ => unreachable!(),
                    }
                }
            }

            (rules, my_ticket, nearby_tickets)
        },
    )
}

fn determine_ticket_scanning_error_rate(field_rules: &FieldRules, tickets: &[Vec<u32>]) -> u32 {
    tickets.iter().fold(0, |error_rate, ticket| {
        error_rate
            + ticket
                .iter()
                .filter(|n| {
                    !field_rules
                        .values()
                        .any(|r| r[0].contains(n) || r[1].contains(n))
                })
                .sum::<u32>()
    })
}

#[cfg(test)]
mod tests {
    use crate::{determine_ticket_scanning_error_rate, parse, FieldRules};

    #[test]
    fn test_determine_ticket_scanning_error_rate() {
        let data = r#"
            class: 1-3 or 5-7
            row: 6-11 or 33-44
            seat: 13-40 or 45-50

            your ticket:
            7,1,14

            nearby tickets:
            7,3,47
            40,4,50
            55,2,20
            38,6,12
        "#;

        let (field_rules, my_ticket, nearby_tickets) = parse(data.as_bytes());
        let error_rate = determine_ticket_scanning_error_rate(&field_rules, &nearby_tickets);

        assert_eq!(71, error_rate);
    }
}
