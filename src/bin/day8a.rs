use clap::Clap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error as ThisError;

#[derive(Clap)]
struct Opts {
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let instructions = File::open(opts.input)
        .map(BufReader::new)?
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| match line.parse::<Instruction>() {
            Ok(instruction) => Some(instruction),
            Err(e) => {
                eprintln!("Failed to parse instruction: {:?}", e);
                None
            }
        })
        .collect::<Vec<_>>();

    let result = run_instructions(&instructions);
    println!("instruction result: {:?}", result);

    Ok(())
}

fn run_instructions(instructions: &[Instruction]) -> i32 {
    let mut acc = 0;
    let mut instruction_pointer = 0;
    let mut executed_instructions = HashSet::new();

    while let Some(instruction) = instructions.get(instruction_pointer) {
        if executed_instructions.contains(&instruction_pointer) {
            return acc;
        } else {
            executed_instructions.insert(instruction_pointer);
        }

        println!(
            "run instruction: {:?} at {:?}",
            instruction, instruction_pointer
        );

        match instruction.operation {
            Operation::Acc => acc += instruction.argument,
            Operation::Jmp => {
                if instruction.argument.is_positive() {
                    instruction_pointer += instruction.argument as usize;
                } else {
                    instruction_pointer -= instruction.argument.abs() as usize;
                }
            }
            Operation::Nop => {}
        }

        match instruction.operation {
            Operation::Acc | Operation::Nop => instruction_pointer += 1,
            _ => {}
        }
    }

    acc
}

#[derive(Debug)]
struct Instruction {
    operation: Operation,
    argument: i32,
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ' ');
        let operation = split
            .next()
            .map(|s| s.parse::<Operation>())
            .unwrap_or_else(|| {
                Err(ParseError::MissingInstructionPart {
                    instruction: String::from(s),
                    missing_part: "operation",
                })
            })?;

        let argument = split
            .next()
            .map(|s| s.parse::<i32>().map_err(|e| e.into()))
            .unwrap_or_else(|| {
                Err(ParseError::MissingInstructionPart {
                    instruction: String::from(s),
                    missing_part: "argument",
                })
            })?;

        Ok(Self {
            operation,
            argument,
        })
    }
}

#[derive(Debug)]
enum Operation {
    Acc,
    Jmp,
    Nop,
}

impl FromStr for Operation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "acc" => Self::Acc,
            "jmp" => Self::Jmp,
            "nop" => Self::Nop,
            _ => return Err(ParseError::UnknownOperation(String::from(s))),
        })
    }
}

#[derive(ThisError, Debug)]
enum ParseError {
    #[error("Missing part {missing_part} of instruction {instruction}")]
    MissingInstructionPart {
        instruction: String,
        missing_part: &'static str,
    },
    #[error("Unknown operation {0}")]
    UnknownOperation(String),
    #[error("Failed to parse argument")]
    InvalidArgument(#[from] ParseIntError),
}
