use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
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

    let instructions = Instruction::read_from(&input)?;

    part1(&instructions[..]);
    part2(&instructions[..]);

    Ok(())
}

fn part1(insts: &[Instruction]) {
    let mut ship = Ship::new();
    ship.run_all(insts);
    let dist = ship.manhattan();
    println!("(part1) manhattan distance: {}", dist);
}

fn part2(insts: &[Instruction]) {
    let mut ship = ShipV2::new();
    ship.run_all(insts);
    let dist = ship.manhattan();
    println!("(part2) manhattan distance: {}", dist);
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Forward,
    Left,
    Right,
    North,
    South,
    East,
    West,
}

impl Action {
    pub fn parser(i: &str) -> IResult<&str, Self> {
        use Action::*;
        let (i, c) = alt((
            char('N'),
            char('S'),
            char('E'),
            char('W'),
            char('F'),
            char('L'),
            char('R'),
        ))(i)?;

        let parsed = match c {
            'N' => North,
            'S' => South,
            'E' => East,
            'W' => West,
            'F' => Forward,
            'L' => Left,
            'R' => Right,
            _ => {
                panic!("Parser failed, cannot not happen!");
            }
        };
        Ok((i, parsed))
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, PartialEq)]
struct Instruction {
    action: Action,
    units: usize,
}

impl Instruction {
    pub fn parser(i: &str) -> IResult<&str, Self> {
        let (i, action) = Action::parser(i)?;
        let (i, num) = digit1(i)?;
        let units = num.parse().unwrap();
        Ok((i, Self { action, units }))
    }

    pub fn read_from(input: &Path) -> Result<Vec<Self>> {
        let mut rv = Vec::new();

        for line in io::BufReader::new(File::open(&input)?).lines() {
            let line = line?;
            match Instruction::parser(&line).finish() {
                Ok((_, instruction)) => {
                    rv.push(instruction);
                }
                Err(e) => bail!("Failed parsing instruction {}: {}", line, e),
            }
        }
        Ok(rv)
    }
}

struct Ship {
    x: i64,
    y: i64,
    facing: Direction,
}

impl Ship {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            facing: Direction::East,
        }
    }

    pub fn run(&mut self, inst: &Instruction) {
        use Action::*;
        match inst.action {
            North => {
                self.y += inst.units as i64;
            }
            South => {
                self.y -= inst.units as i64;
            }
            East => {
                self.x += inst.units as i64;
            }
            West => {
                self.x -= inst.units as i64;
            }
            Forward => match self.facing {
                Direction::North => {
                    self.y += inst.units as i64;
                }
                Direction::South => {
                    self.y -= inst.units as i64;
                }
                Direction::East => {
                    self.x += inst.units as i64;
                }
                Direction::West => {
                    self.x -= inst.units as i64;
                }
            },
            Right => {
                let turns = inst.units / 90;
                for _ in 0..turns {
                    self.facing = match self.facing {
                        Direction::North => Direction::East,
                        Direction::South => Direction::West,
                        Direction::East => Direction::South,
                        Direction::West => Direction::North,
                    }
                }
            }
            Left => {
                let turns = inst.units / 90;
                for _ in 0..turns {
                    self.facing = match self.facing {
                        Direction::North => Direction::West,
                        Direction::South => Direction::East,
                        Direction::East => Direction::North,
                        Direction::West => Direction::South,
                    }
                }
            }
        }
    }

    pub fn run_all(&mut self, insts: &[Instruction]) {
        for inst in insts.iter() {
            self.run(inst);
        }
    }

    pub fn manhattan(&self) -> usize {
        self.x.abs() as usize + self.y.abs() as usize
    }
}

struct ShipV2 {
    x: i64,
    y: i64,
    waypoint_x: i64,
    waypoint_y: i64,
}

impl ShipV2 {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            waypoint_x: 10,
            waypoint_y: 1,
        }
    }

    pub fn run(&mut self, inst: &Instruction) {
        use Action::*;
        let units = inst.units as i64;
        match inst.action {
            North => {
                self.waypoint_y += units;
            }
            South => {
                self.waypoint_y -= units;
            }
            East => {
                self.waypoint_x += units;
            }
            West => {
                self.waypoint_x -= units;
            }
            Forward => {
                self.x += units * self.waypoint_x;
                self.y += units * self.waypoint_y;
            }
            Right => {
                let turns = inst.units / 90;
                for _ in 0..turns {
                    let old_x = self.waypoint_x;
                    let old_y = self.waypoint_y;
                    self.waypoint_x = old_y;
                    self.waypoint_y = -old_x;
                }
            }
            Left => {
                let turns = inst.units / 90;
                for _ in 0..turns {
                    let old_x = self.waypoint_x;
                    let old_y = self.waypoint_y;
                    self.waypoint_x = -old_y;
                    self.waypoint_y = old_x;
                }
            }
        }
    }

    pub fn run_all(&mut self, insts: &[Instruction]) {
        for inst in insts.iter() {
            self.run(inst);
        }
    }

    pub fn manhattan(&self) -> usize {
        self.x.abs() as usize + self.y.abs() as usize
    }
}
