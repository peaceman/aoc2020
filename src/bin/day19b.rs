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

    let (mut rules, messages) = parse(reader);

    let input = r#"
            8: 42 | 42 8
            11: 42 31 | 42 11 31
        "#;

    let (new_rules, _) = parse(input.trim().as_bytes());
    rules.extend(new_rules);

    let valid_messages = messages
        .iter()
        .filter(|m| is_valid_message(&rules, 0, m.as_ref()))
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

fn is_valid_message(rules: &Rules, idx: usize, msg: &str) -> bool {
    fn try_validation(rules: &Rules, idx: usize, msg: &str, level: usize) -> Option<usize> {
        rules.get(&idx).and_then(|sub_rules| {
            sub_rules
                .iter()
                .filter_map(|sub_rule| {
                    let mut char_idx = 0;

                    for rule_part in sub_rule {
                        if char_idx >= msg.len() {
                            println!(
                                "reached end of message {} {:?} {:?}",
                                idx, sub_rule, rule_part
                            );
                            return None;
                        }

                        println!(
                            "{:->width$} {} {:?} {:?} {:?}",
                            ">",
                            msg,
                            idx,
                            sub_rule,
                            rule_part,
                            width = level
                        );

                        match rule_part {
                            RulePart::Literal(c) => {
                                if msg.chars().nth(char_idx).unwrap() != *c {
                                    return None;
                                }

                                char_idx += 1;
                            }
                            RulePart::Idx(rule_idx) => {
                                match try_validation(rules, *rule_idx, &msg[char_idx..], level + 1)
                                {
                                    Some(chars_matched) => char_idx += chars_matched,
                                    None => {
                                        return None;
                                    }
                                }
                            }
                        }
                    }

                    Some(char_idx)
                })
                .take(1)
                .next()
        })
    }

    try_validation(rules, idx, msg, 0)
        .map(|len| {
            println!("{} == {}", len, msg.len());
            len == msg.len()
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::{expand_rule, is_valid_message, parse, RulePart, Rules};
    use std::collections::HashSet;

    #[test]
    fn test_matching_messages() {
        // let input = r#"
        //     0: 4 1 5
        //     1: 2 3 | 3 2
        //     2: 4 4 | 5 5
        //     3: 4 5 | 5 4
        //     4: "a"
        //     5: "b"
        //
        //     ababbb
        //     bababa
        //     abbbab
        //     aaabbb
        //     aaaabbb
        // "#;
        //
        // let (rules, messages) = parse(input.trim().as_bytes());
        //
        // assert_eq!(
        //     2,
        //     messages
        //         .iter()
        //         .filter(|msg| is_valid_message(&rules, 0, msg.as_str()))
        //         .count()
        // );

        let input = r#"
            42: 9 14 | 10 1
            9: 14 27 | 1 26
            10: 23 14 | 28 1
            1: "a"
            11: 42 31
            5: 1 14 | 15 1
            19: 14 1 | 14 14
            12: 24 14 | 19 1
            16: 15 1 | 14 14
            31: 14 17 | 1 13
            6: 14 14 | 1 14
            2: 1 24 | 14 4
            0: 8 11
            13: 14 3 | 1 12
            15: 1 | 14
            17: 14 2 | 1 7
            23: 25 1 | 22 14
            28: 16 1
            4: 1 1
            20: 14 14 | 1 15
            3: 5 14 | 16 1
            27: 1 6 | 14 18
            14: "b"
            21: 14 1 | 1 14
            25: 1 1 | 1 14
            22: 14 14
            8: 42
            26: 14 22 | 1 20
            18: 15 15
            7: 14 5 | 1 21
            24: 14 1
            
            abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa
            bbabbbbaabaabba
            babbbbaabbbbbabbbbbbaabaaabaaa
            aaabbbbbbaaaabaababaabababbabaaabbababababaaa
            bbbbbbbaaaabbbbaaabbabaaa
            bbbababbbbaaaaaaaabbababaaababaabab
            ababaaaaaabaaab
            ababaaaaabbbaba
            baabbaaaabbaaaababbaababb
            abbbbabbbbaaaababbbbbbaaaababb
            aaaaabbaabaaaaababaa
            aaaabbaaaabbaaa
            aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
            babaaabbbaaabaababbaabababaaab
            aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba        
        "#;

        let (mut rules, messages) = parse(input.trim().as_bytes());

        // assert_eq!(
        //     3,
        //     messages
        //         .iter()
        //         .filter(|msg| is_valid_message(&rules, 0, msg.as_str()))
        //         .count()
        // );

        let input = r#"
            8: 42 | 42 8
            11: 42 31 | 42 11 31
        "#;

        let (new_rules, _) = parse(input.trim().as_bytes());
        rules.extend(new_rules);

        assert_eq!(false, is_valid_message(&rules, 0, "aaaabbaaaabbaaa"));
        //
        // let valid_messages = messages
        //     .iter()
        //     .filter(|msg| is_valid_message(&rules, 0, msg.as_str()))
        //     .collect::<Vec<_>>();
        // //
        // // println!("hypu");
        // assert_eq!(
        //     true,
        //     is_valid_message(&rules, 0, "babbbbaabbbbbabbbbbbaabaaabaaa")
        // );
        //
        // println!("valid messages: {:#?}", valid_messages);
        // assert_eq!(12, valid_messages.len());
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
