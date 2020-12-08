use anyhow::{bail, Context, Error, Result};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::str;

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );
    let instructions = Instruction::read_from(&input)?;
    part1(&instructions)?;
    part2(&instructions)?;

    Ok(())
}

fn part1(instructions: &Vec<Instruction>) -> Result<()> {
    if let RunResult::Loop(acc_at_loop) = run(instructions)? {
        println!("(part1) Accumulator at loop: {}", acc_at_loop);
    } else {
        bail!("Did not terminate!");
    }
    Ok(())
}

fn part2(instructions: &Vec<Instruction>) -> Result<()> {
    use RunResult::*;
    for idx in 0..instructions.len() {
        if let Some(flipped) = flip_at(instructions, idx) {
            match run(&flipped)? {
                Halt(acc) => {
                    println!("Flipping at {} returns: {}", idx, acc);
                    return Ok(());
                }
                Loop(_) => {
                    continue;
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum Instruction {
    Jmp(i64),
    Acc(i64),
    Nop(i64),
}

impl TryFrom<&str> for Instruction {
    type Error = Error;

    fn try_from(i: &str) -> Result<Self> {
        use Instruction::*;
        let split: Vec<_> = i.split_whitespace().collect();

        if split.len() != 2 {
            bail!("Expected pair, found {} items", split.len());
        } else {
            let num: i64 = split[1]
                .parse()
                .with_context(|| format!("Invalid number: {}", split[1]))?;
            let instruction = match split[0] {
                "jmp" => Jmp(num),
                "acc" => Acc(num),
                "nop" => Nop(num),
                other => {
                    bail!("Invalid instruction: {}", other);
                }
            };
            Ok(instruction)
        }
    }
}

impl Instruction {
    pub fn read_from(path: &Path) -> Result<Vec<Self>> {
        let mut instructions = Vec::new();
        for line in io::BufReader::new(File::open(&path)?).lines() {
            instructions.push(Self::try_from(line?.as_str())?);
        }
        Ok(instructions)
    }
}

enum RunResult {
    Loop(i64),
    Halt(i64),
}

fn run(instructions: &Vec<Instruction>) -> Result<RunResult> {
    let mut visited: HashSet<i64> = HashSet::new();
    let mut idx: i64 = 0;
    let mut acc: i64 = 0;

    use Instruction::*;
    use RunResult::*;
    loop {
        if idx < 0 {
            bail!("Instruction index went negative: {}", idx);
        } else if idx as usize >= instructions.len() {
            return Ok(Halt(acc));
        }

        if visited.contains(&idx) {
            return Ok(Loop(acc));
        } else {
            visited.insert(idx);
        }
        match instructions[idx as usize] {
            Jmp(count) => {
                idx += count;
            }
            Acc(count) => {
                idx += 1;
                acc += count;
            }
            Nop(_) => {
                idx += 1;
            }
        }
    }
}

/// Flip the instruction at idx, returning a copy of the vector.
/// Returns None if instruction at idx does not support flicking.
fn flip_at(instructions: &Vec<Instruction>, idx: usize) -> Option<Vec<Instruction>> {
    use Instruction::*;
    if let Some(flipped) = match instructions[idx] {
        Jmp(count) => Some(Nop(count)),
        Acc(_) => None,
        Nop(count) => Some(Jmp(count)),
    } {
        let mut rv = instructions.clone();
        rv[idx] = flipped;
        Some(rv)
    } else {
        None
    }
}
