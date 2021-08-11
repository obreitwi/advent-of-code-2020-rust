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
    let input = read_to_string(&PathBuf::from("input.txt"))?;
    part1(&input)?;
    // part2(&input)?;
    Ok(())
}

fn part1(input: &str) -> Result<()> {
    let mut grid = Grid::from(input);
    grid.process_all_paths();
    println!("Number of black tiles: {}", grid.count_black_tiles());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl Coordinate {
    fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

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
struct PathToTile {
    directions: Vec<Direction>,
}

impl PathToTile {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        map(many1(Direction::parse), |directions| Self { directions })(i)
    }
}

#[derive(Debug)]
struct Grid {
    paths: Vec<PathToTile>,
    black_tiles: HashSet<Coordinate>,
}

impl Grid {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(separated_list1(line_ending, PathToTile::parse), |paths| {
            Self {
                paths,
                black_tiles: HashSet::new(),
            }
        })(i)
    }

    fn count_black_tiles(&self) -> usize {
        self.black_tiles.len()
    }

    fn process_all_paths(&mut self) {
        while let Some(path) = self.paths.pop() {
            self.process_path(&path);
        }
    }

    fn process_path(&mut self, path: &PathToTile) {
        let mut coord = Coordinate::origin();
        for dir in path.directions.iter() {
            coord.apply(dir);
        }
        self.flip(coord);
    }

    fn flip(&mut self, coordinate: Coordinate) {
        if !self.black_tiles.remove(&coordinate) {
            self.black_tiles.insert(coordinate);
        }
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
        let mut grid = Grid::from(input.as_str());
        grid.process_all_paths();
        assert_eq!(grid.count_black_tiles(), 10);
        Ok(())
    }
}
