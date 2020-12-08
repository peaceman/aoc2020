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

    let result = run_instructions(&instructions, RunState::default());
    println!("instruction result: {:?}", result);

    Ok(())
}

fn run_instructions(instructions: &[Instruction], mut state: RunState) -> Result<i32, RunError> {
    while let Some(instruction) = instructions.get(state.instruction_pointer) {
        if !state
            .executed_instructions
            .insert(state.instruction_pointer)
        {
            return Err(RunError::DetectedLoop {
                instruction_pointer: state.instruction_pointer,
                accumulator: state.accumulator,
            });
        }

        if matches!(instruction.operation, Operation::Jmp | Operation::Nop) {
            // try original
            let mut try_state = state.clone();
            run_instruction(
                instruction,
                &mut try_state.accumulator,
                &mut try_state.instruction_pointer,
            );

            let try_result = run_instructions(instructions, try_state);
            if let Ok(v) = try_result {
                return Ok(v);
            } else {
                println!(
                    "detected a loop in the original instruction, try to correct: {:?} at {}",
                    instruction, state.instruction_pointer
                );
            }

            // try corrected
            let mut try_state = state.clone();
            let corrected_instruction = Instruction {
                operation: match instruction.operation {
                    Operation::Jmp => Operation::Nop,
                    Operation::Nop => Operation::Jmp,
                    _ => panic!("Can't correct an acc instruction"),
                },
                ..*instruction
            };

            run_instruction(
                &corrected_instruction,
                &mut try_state.accumulator,
                &mut try_state.instruction_pointer,
            );

            return run_instructions(instructions, try_state);
        } else {
            run_instruction(
                instruction,
                &mut state.accumulator,
                &mut state.instruction_pointer,
            );
        }
    }

    Ok(state.accumulator)
}

fn run_instruction(instruction: &Instruction, acc: &mut i32, instruction_pointer: &mut usize) {
    println!(
        "run instruction: {:?} at {:?}",
        instruction, instruction_pointer
    );

    match instruction.operation {
        Operation::Acc => *acc += instruction.argument,
        Operation::Jmp => {
            if instruction.argument.is_positive() {
                *instruction_pointer += instruction.argument as usize;
            } else {
                *instruction_pointer -= instruction.argument.abs() as usize;
            }
        }
        Operation::Nop => {}
    }

    match instruction.operation {
        Operation::Acc | Operation::Nop => *instruction_pointer += 1,
        _ => {}
    }
}

#[derive(Debug, Clone, Default)]
struct RunState {
    accumulator: i32,
    instruction_pointer: usize,
    executed_instructions: HashSet<usize>,
}

#[derive(ThisError, Debug)]
enum RunError {
    #[error("Detected instruction loop at pointer: {instruction_pointer}")]
    DetectedLoop {
        instruction_pointer: usize,
        accumulator: i32,
    },
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

#[derive(Debug, Clone, Copy)]
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
