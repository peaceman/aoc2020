use clap::Clap;
use scan_fmt::scan_fmt;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::RangeInclusive;
use std::time::Instant;

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
    // println!("my_ticket: {:#?}", my_ticket);
    // println!("nearby_tickets: {:#?}", nearby_tickets);

    println!(
        "ticket scanning error rate: {:?}",
        determine_ticket_scanning_error_rate(&field_rules, &nearby_tickets)
    );

    let start = Instant::now();
    let valid_tickets = nearby_tickets
        .into_iter()
        .filter(|t| validate_ticket(&field_rules, t))
        .collect::<Vec<_>>();

    let field_positions = determine_field_positions(&field_rules, &valid_tickets);
    println!("field_positions: {:#?}", field_positions);
    let result = field_positions
        .iter()
        .filter(|(field, _pos)| field.starts_with("departure"))
        .map(|(_field, &pos)| my_ticket[pos])
        .fold(1u64, |acc, curr| acc * curr as u64);

    println!("multiplication result: {} in {:?}", result, start.elapsed());

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
                            my_ticket = ticket;
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

fn validate_ticket(field_rules: &FieldRules, ticket: &Vec<u32>) -> bool {
    ticket.iter().all(|n| {
        field_rules
            .values()
            .any(|r| r.iter().any(|r| r.contains(n)))
    })
}

fn determine_field_positions(
    field_rules: &FieldRules,
    tickets: &[Vec<u32>],
) -> HashMap<String, usize> {
    if tickets.is_empty() {
        return HashMap::new();
    }

    let mut potential_positions: HashMap<&String, HashSet<usize>> = HashMap::new();

    for pos in 0..tickets[0].len() {
        for (field, rules) in field_rules {
            if tickets
                .iter()
                .map(|t| t[pos])
                .all(|tn| rules.iter().any(|r| r.contains(&tn)))
            {
                potential_positions.entry(field).or_default().insert(pos);
            }
        }
    }

    let mut determined_positions = HashSet::new();
    let mut field_positions = HashMap::new();

    loop {
        if potential_positions.is_empty() {
            break;
        }

        let matched_fields = potential_positions
            .iter()
            .filter_map(|(&field, potential_positions)| {
                let pos_diff = potential_positions
                    .difference(&determined_positions)
                    .collect::<Vec<_>>();

                if pos_diff.len() == 1 {
                    let pos = *pos_diff[0];
                    field_positions.insert(String::from(field), pos);
                    determined_positions.insert(pos);
                    Some(field)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for field in matched_fields {
            potential_positions.remove(field);
        }
    }

    field_positions
}

#[cfg(test)]
mod tests {
    use crate::{
        determine_field_positions, determine_ticket_scanning_error_rate, parse, FieldRules,
    };

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

    #[test]
    fn test_determine_field_positions() {
        let data = r#"
            class: 0-1 or 4-19
            row: 0-5 or 8-19
            seat: 0-13 or 16-19
            
            your ticket:
            11,12,13
            
            nearby tickets:
            3,9,18
            15,1,5
            5,14,9
        "#;

        let (field_rules, my_ticket, nearby_tickets) = parse(data.as_bytes());
        let field_positions = determine_field_positions(&field_rules, &nearby_tickets);

        // println!("field_positions: {:#?}", field_positions);

        assert_eq!(Some(&0), field_positions.get("row"));
        assert_eq!(Some(&1), field_positions.get("class"));
        assert_eq!(Some(&2), field_positions.get("seat"));
    }
}
