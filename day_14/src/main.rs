use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, one_of, space0},
    multi::many1,
    Finish, IResult,
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let program = Program::read_from(&input)?;

    // println!("Read: {:#?}", program);

    part1(&program);

    Ok(())
}

#[derive(Clone, Debug)]
struct Bitmask {
    // inverted bitmask to filter out all bits set to specific value
    bitmask_0_inv: u64,
    bitmask_1: u64,
}

impl Bitmask {
    pub fn empty() -> Bitmask {
        let bitmask_0_inv = (1 << 36) - 1;
        let bitmask_1 = 0;

        Self {
            bitmask_1,
            bitmask_0_inv,
        }
    }

    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("mask")(i)?;
        let (i, _) = space0(i)?;
        let (i, _) = char('=')(i)?;
        let (i, _) = space0(i)?;

        let (i, mask) = many1(one_of("X01"))(i)?;

        let mut bitmask_0_inv = 0;
        let mut bitmask_1 = 0;

        for m in mask.iter() {
            bitmask_0_inv = bitmask_0_inv << 1;
            bitmask_1 = bitmask_1 << 1;

            match m {
                'X' => {
                    bitmask_0_inv += 1;
                }
                '0' => {
                }
                '1' => {
                    bitmask_0_inv += 1;
                    bitmask_1 += 1;
                }
                _ => {
                    panic!("Cannot happen!");
                }
            };
        }
        Ok((
            i,
            Self {
                bitmask_1,
                bitmask_0_inv,
            },
        ))
    }
}

#[derive(Debug)]
struct Assignment {
    idx: usize,
    value: u64,
}

impl Assignment {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("mem[")(i)?;
        let (i, idx) = digit1(i)?;
        let (i, _) = char(']')(i)?;
        let (i, _) = space0(i)?;
        let (i, _) = char('=')(i)?;
        let (i, _) = space0(i)?;
        let (i, value) = digit1(i)?;

        Ok((
            i,
            Self {
                idx: idx.parse::<usize>().unwrap(),
                value: value.parse::<u64>().unwrap(),
            },
        ))
    }
}

#[derive(Debug)]
enum Instruction {
    Mask(Bitmask),
    Mem(Assignment),
}

impl Instruction {
    pub fn parse(i: &str) -> IResult<&str, Instruction> {
        let (_, tag) = alt((tag("mem"), tag("mask")))(i)?;
        match tag {
            "mem" => {
                let (i, mem) = Assignment::parse(i)?;
                Ok((i, Instruction::Mem(mem)))
            }
            "mask" => {
                let (i, mask) = Bitmask::parse(i)?;
                Ok((i, Instruction::Mask(mask)))
            }
            _ => {
                panic!("Cannot happen.")
            }
        }
    }
}

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn read_from(input: &Path) -> Result<Self> {
        let mut instructions = Vec::new();

        for line in io::BufReader::new(File::open(&input)?).lines() {
            match Instruction::parse(&line?).finish() {
                Ok((_, instr)) => {
                    instructions.push(instr);
                }
                Err(e) => {
                    bail!("Invalid assignment line: {}", e);
                }
            }
        }

        Ok(Self { instructions })
    }

    pub fn run(&self) -> HashMap<usize, u64> {
        use Instruction::*;
        let mut memory = HashMap::new();

        let mut mask = Bitmask::empty();

        for instr in self.instructions.iter() {
            match instr {
                Mem(assign) => {
                    let value = assign.value & mask.bitmask_0_inv | mask.bitmask_1;
                    memory.insert(assign.idx, value);
                }
                Mask(m) => {
                    mask = m.clone();
                }
            }
        }

        memory
    }
}

fn part1(prog: &Program) {
    let mem = prog.run();
    let sum: u64 = mem.values().sum();

    println!("(part1) Sum of all elements: {}", sum);
}
