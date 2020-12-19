use clap::Clap;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let (rules, messages) = parse(reader);
    let allowed_messages = expand_rule(&rules, 0);

    let valid_messages = messages
        .iter()
        .filter(|m| allowed_messages.contains(m.as_str()))
        .count();
    println!("valid messages: {}", valid_messages);

    Ok(())
}

type Rules = HashMap<usize, Vec<Vec<RulePart>>>;

#[derive(Debug)]
enum RulePart {
    Literal(char),
    Idx(usize),
}

fn parse(input: impl BufRead) -> (Rules, Vec<String>) {
    let mut parse_rules = true;

    input
        .lines()
        .filter_map(|l| l.ok())
        .fold((HashMap::new(), Vec::new()), |mut acc, l| {
            if l.trim().is_empty() {
                parse_rules = false;
                return acc;
            }

            if parse_rules {
                let mut line_parts = l.trim().splitn(2, ':');
                let rule_idx = line_parts.next().unwrap().parse::<usize>().unwrap();
                let rule_parts = line_parts.next().unwrap().split('|');

                let rule = rule_parts
                    .map(|rule_part| {
                        rule_part
                            .trim()
                            .split_whitespace()
                            .map(|s| {
                                if s.starts_with('"') && s.ends_with('"') {
                                    RulePart::Literal(s.trim_matches('"').chars().next().unwrap())
                                } else {
                                    RulePart::Idx(s.parse::<usize>().unwrap())
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                acc.0.insert(rule_idx, rule);
            } else {
                acc.1.push(String::from(l.trim()));
            }

            acc
        })
}

fn expand_rule(rules: &Rules, idx: usize) -> HashSet<String> {
    let mut cache = HashMap::new();

    fn expand(rules: &Rules, idx: usize, cache: &mut HashMap<usize, Vec<String>>) -> Vec<String> {
        if let Some(v) = cache.get(&idx) {
            return v.clone();
        }

        if let Some(rule) = rules.get(&idx) {
            let mut results = Vec::new();
            for sub_rule in rule {
                sub_rule
                    .iter()
                    .map(|rule_part| match rule_part {
                        RulePart::Literal(c) => vec![String::from(*c)],
                        RulePart::Idx(idx) => expand(rules, *idx, cache),
                    })
                    .multi_cartesian_product()
                    .map(|v| v.iter().map(|s| s.as_str()).join(""))
                    .for_each(|v| results.push(v));
            }

            cache.insert(idx, results);
        }

        cache.entry(idx).or_default().clone()
    }

    expand(rules, idx, &mut cache)
        .iter()
        .map(|s| s.clone())
        .collect::<HashSet<_>>()
}

#[cfg(test)]
mod tests {
    use crate::{expand_rule, parse, Rules};
    use std::collections::HashSet;

    #[test]
    fn test_matching_messages() {
        let input = r#"
            0: 4 1 5
            1: 2 3 | 3 2
            2: 4 4 | 5 5
            3: 4 5 | 5 4
            4: "a"
            5: "b"

            ababbb
            bababa
            abbbab
            aaabbb
            aaaabbb
        "#;

        let (rules, messages) = parse(input.trim().as_bytes());
        let allowed_messages: HashSet<String> = expand_rule(&rules, 0);

        println!("allowed messages: {:?}", allowed_messages);
        let valid_messages = messages
            .iter()
            .filter(|m| allowed_messages.contains(*m))
            .count();

        assert_eq!(2, valid_messages);
    }

    #[test]
    fn test_rule_expansion() {
        let input = r#"
            0: 4 1 5
            1: 2 3 | 3 2
            2: 4 4 | 5 5
            3: 4 5 | 5 4
            4: "a"
            5: "b"

            ababbb
            bababa
            abbbab
            aaabbb
            aaaabbb
        "#;

        let (rules, messages) = parse(input.trim().as_bytes());
        let allowed_messages: HashSet<String> = expand_rule(&rules, 0);
        let mut result = allowed_messages.into_iter().collect::<Vec<_>>();
        result.sort();

        let mut expected = vec![
            "aaaabb", "aaabab", "abbabb", "abbbab", "aabaab", "aabbbb", "abaaab", "ababbb",
        ];

        expected.sort();

        assert_eq!(expected, result);
    }
}
