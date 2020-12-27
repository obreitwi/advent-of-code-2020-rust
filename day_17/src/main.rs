use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, none_of, one_of},
    combinator::value,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let cubes = ConwayCubes3D::read_from(&input)?;
    eprintln!("Cubes: {:#?}", cubes);
    part1(&cubes);

    let cubes = ConwayCubes4D::read_from(&input)?;
    part2(&cubes);

    Ok(())
}

fn part1(cubes: &ConwayCubes3D) {
    let updated = cubes.run_updates(6);

    println!(
        "(part1) After 6 iterations: {} active cubes",
        updated.num_active()
    );
}

fn part2(cubes: &ConwayCubes4D) {
    let updated = cubes.run_updates(6);

    println!(
        "(part1) After 6 iterations: {} active cubes",
        updated.num_active()
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CubeState {
    Active,
    Inactive,
}

impl CubeState {
    fn parse(i: &str) -> IResult<&str, CubeState> {
        alt((
            value(CubeState::Inactive, char('.')),
            value(CubeState::Active, char('#')),
        ))(i)
    }
}

type Position3D = (i64, i64, i64);

#[derive(Debug, Clone)]
struct ConwayCubes3D {
    active: HashSet<Position3D>,
}

impl ConwayCubes3D {
    fn read_from(input: &Path) -> Result<Self> {
        let input = read_to_string(input)?;
        match Self::parse(&input).finish() {
            Ok((i, cubes)) => {
                if i.len() == 0 {
                    Ok(cubes)
                } else {
                    bail!("Did not consume all of input, {} bytes left!", i.len());
                }
            }
            Err(e) => {
                bail!("Error during parsing: {}", e);
            }
        }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, lines) = separated_list1(line_ending, many1(CubeState::parse))(i)?;

        let mut active = HashSet::new();

        for (idx_y, line) in lines.iter().enumerate() {
            for (idx_x, state) in line.iter().enumerate() {
                if let CubeState::Active = state {
                    active.insert((idx_x as i64, idx_y as i64, 0));
                }
            }
        }
        let (i, _) = many0(line_ending)(i)?;
        Ok((i, Self { active }))
    }

    fn update(&self) -> Self {
        let mut num_neighbors: HashMap<Position3D, usize> = HashMap::new();

        for (x, y, z) in self.active.iter() {
            for dx in -1..2 {
                for dy in -1..2 {
                    for dz in -1..2 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        *num_neighbors.entry((x + dx, y + dy, z + dz)).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut updated = HashSet::new();

        let of_interest: HashMap<Position3D, usize> = num_neighbors
            .into_iter()
            .filter_map(|(k, v)| if v == 2 || v == 3 { Some((k, v)) } else { None })
            .collect();

        for (pos, count) in of_interest.into_iter() {
            if count == 3 {
                // is always active
                updated.insert(pos);
            } else if self.active.contains(&pos) {
                // only remains active
                updated.insert(pos);
            }
        }

        Self { active: updated }
    }

    fn run_updates(&self, count: usize) -> Self {
        let mut retval = self.clone();
        for _ in 0..count {
            retval = retval.update();
        }
        retval
    }

    fn num_active(&self) -> usize {
        self.active.len()
    }
}

type Position4D = (i64, i64, i64, i64);

#[derive(Debug, Clone)]
struct ConwayCubes4D {
    active: HashSet<Position4D>,
}

impl ConwayCubes4D {
    fn read_from(input: &Path) -> Result<Self> {
        let input = read_to_string(input)?;
        match Self::parse(&input).finish() {
            Ok((i, cubes)) => {
                if i.len() == 0 {
                    Ok(cubes)
                } else {
                    bail!("Did not consume all of input, {} bytes left!", i.len());
                }
            }
            Err(e) => {
                bail!("Error during parsing: {}", e);
            }
        }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, lines) = separated_list1(line_ending, many1(CubeState::parse))(i)?;

        let mut active = HashSet::new();

        for (idx_y, line) in lines.iter().enumerate() {
            for (idx_x, state) in line.iter().enumerate() {
                if let CubeState::Active = state {
                    active.insert((idx_x as i64, idx_y as i64, 0, 0));
                }
            }
        }
        let (i, _) = many0(line_ending)(i)?;
        Ok((i, Self { active }))
    }

    fn update(&self) -> Self {
        let mut num_neighbors: HashMap<Position4D, usize> = HashMap::new();

        for (x, y, z, w) in self.active.iter() {
            for dx in -1..2 {
                for dy in -1..2 {
                    for dz in -1..2 {
                        for dw in -1..2 {
                            if dx == 0 && dy == 0 && dz == 0 && dw == 0 {
                                continue;
                            }
                            *num_neighbors
                                .entry((x + dx, y + dy, z + dz, w + dw))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        let mut updated = HashSet::new();

        let of_interest: HashMap<Position4D, usize> = num_neighbors
            .into_iter()
            .filter_map(|(k, v)| if v == 2 || v == 3 { Some((k, v)) } else { None })
            .collect();

        for (pos, count) in of_interest.into_iter() {
            if count == 3 {
                // is always active
                updated.insert(pos);
            } else if self.active.contains(&pos) {
                // only remains active
                updated.insert(pos);
            }
        }

        Self { active: updated }
    }

    fn run_updates(&self, count: usize) -> Self {
        let mut retval = self.clone();
        for _ in 0..count {
            retval = retval.update();
        }
        retval
    }

    fn num_active(&self) -> usize {
        self.active.len()
    }
}
