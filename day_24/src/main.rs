#![allow(unused_imports)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space0},
    combinator::{map, map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::collections::{HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    // part1(&input)?;
    // part2(&input)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

/*
 * Cube coordinates:
 *       -z
 *    /      \
 *  +y       +x
 *   |        |
 *  -x       -y
 *    \      /
 *       +z
 */

#[derive(Debug, Clone, Copy)]
struct Coordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl Coordinate {
    fn apply(&mut self, dir: &Direction) {
        use Direction::*;

        match *dir {
            West => {
                self.y += 1;
                self.x -= 1;
            }
            East => {
                self.x += 1;
                self.y -= 1;
            }
            NorthWest => {
                self.y += 1;
                self.z -= 1;
            }
            NorthEast => {
                self.x += 1;
                self.z -= 1;
            }
            SouthWest => {
                self.z += 1;
                self.x -= 1;
            }
            SouthEast => {
                self.z += 1;
                self.y -= 1;
            }
        }
    }
}

impl Direction {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            Self::parse_west,
            Self::parse_east,
            Self::parse_north_west,
            Self::parse_north_east,
            Self::parse_south_west,
            Self::parse_south_east,
        ))(i)
    }

    fn parse_west(i: &str) -> IResult<&str, Self> {
        value(Self::West, tag("w"))(i)
    }

    fn parse_east(i: &str) -> IResult<&str, Self> {
        value(Self::East, tag("e"))(i)
    }

    fn parse_north_west(i: &str) -> IResult<&str, Self> {
        value(Self::NorthWest, tag("nw"))(i)
    }

    fn parse_north_east(i: &str) -> IResult<&str, Self> {
        value(Self::NorthEast, tag("ne"))(i)
    }

    fn parse_south_west(i: &str) -> IResult<&str, Self> {
        value(Self::SouthWest, tag("sw"))(i)
    }

    fn parse_south_east(i: &str) -> IResult<&str, Self> {
        value(Self::SouthEast, tag("se"))(i)
    }
}

#[derive(Debug)]
struct Tile {
    directions: Vec<Direction>,
}

impl Tile {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        map(many1(Direction::parse), |directions| Self { directions })(i)
    }
}

#[derive(Debug)]
struct Grid {
    tiles: Vec<Tile>,
}

impl Grid {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(separated_list1(line_ending, Tile::parse), |tiles| Self {
            tiles,
        })(i)
    }
}

impl From<&str> for Grid {
    fn from(i: &str) -> Self {
        match Self::parse(i).finish() {
            Ok((i, cc)) => {
                assert!(i == "\n", "Did not consume full string.");
                cc
            }
            Err(e) => {
                panic!("Error parsing Grid: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_debug() -> Result<()> {
        let input = read_to_string(&PathBuf::from("debug.txt"))?;
        let grid = Grid::from(input.as_str());
        Ok(())
    }
}
