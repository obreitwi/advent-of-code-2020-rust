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
    part2(&program);

    Ok(())
}

const BITWIDTH: usize = 36;

#[derive(Clone, Debug)]
struct Bitmask {
    // inverted bitmask to filter out all bits set to specific value
    bitmask_0_inv: u64,
    bitmask_1: u64,

    floating: Vec<usize>, // floating bits, LSB is 0, MSB is BITWIDTH
}

impl Bitmask {
    pub fn empty() -> Bitmask {
        let bitmask_0_inv = (1 << BITWIDTH) - 1;
        let bitmask_1 = 0;

        Self {
            bitmask_1,
            bitmask_0_inv,
            floating: Vec::new(),
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
        let mut floating = Vec::new();

        for (i, m) in mask.iter().enumerate() {
            bitmask_0_inv = bitmask_0_inv << 1;
            bitmask_1 = bitmask_1 << 1;

            match m {
                'X' => {
                    bitmask_0_inv += 1;
                    floating.push(BITWIDTH - 1 - i);
                }
                '0' => {}
                '1' => {
                    bitmask_0_inv += 1;
                    bitmask_1 += 1;
                }
                _ => {
                    panic!("Cannot happen!");
                }
            };
        }
        floating.sort();
        Ok((
            i,
            Self {
                bitmask_1,
                bitmask_0_inv,
                floating,
            },
        ))
    }

    pub fn generate_addresses(&self, base: usize) -> Vec<usize> {
        let mut rv = Vec::new();

        let base = base | self.bitmask_1 as usize;

        for count in 0..(1 << self.floating.len()) {
            let mut current = base;
            for (count_idx, float_idx) in self.floating.iter().enumerate() {
                // delete bit at float_idx in current
                if current & (1 << float_idx) > 0 {
                    current -= 1 << float_idx;
                }

                // write bit at count_idx from count to float_idx in current
                if count & (1 << count_idx) > 0 {
                    current += 1 << float_idx;
                }
            }
            rv.push(current);
        }

        rv
    }
}

#[derive(Debug)]
struct Assignment {
    address: usize,
    value: u64,
}

impl Assignment {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("mem[")(i)?;
        let (i, address) = digit1(i)?;
        let (i, _) = char(']')(i)?;
        let (i, _) = space0(i)?;
        let (i, _) = char('=')(i)?;
        let (i, _) = space0(i)?;
        let (i, value) = digit1(i)?;

        Ok((
            i,
            Self {
                address: address.parse::<usize>().unwrap(),
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
                    memory.insert(assign.address, value);
                }
                Mask(m) => {
                    mask = m.clone();
                }
            }
        }

        memory
    }

    pub fn run_v2(&self) -> HashMap<usize, u64> {
        use Instruction::*;
        let mut memory = HashMap::new();

        let mut mask = Bitmask::empty();

        for instr in self.instructions.iter() {
            match instr {
                Mem(assign) => {
                    for address in mask.generate_addresses(assign.address)
                    {
                        memory.insert(address, assign.value);
                    }
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

fn part2(prog: &Program) {
    let mem = prog.run_v2();
    let sum: u64 = mem.values().sum();

    println!("(part2) Sum of all elements: {}", sum);
}
