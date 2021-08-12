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
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() -> Result<()> {
    let input = read_to_string(&PathBuf::from("input.txt"))?;
    part1(&input)?;
    part2(&input)?;
    Ok(())
}

fn part1(input: &str) -> Result<()> {
    let grid = Grid::from(input);
    println!("Number of black tiles: {}", grid.count_black_tiles());
    Ok(())
}

fn part2(input: &str) -> Result<()> {
    let mut grid = Grid::from(input);
    grid.update_n_days(100);
    Ok(())
}

#[derive(Debug, Clone, Copy, EnumIter)]
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

    fn apply(mut self, dir: &Direction) -> Self {
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
        self
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
    black_tiles: HashSet<Coordinate>,
}

impl Grid {
    fn parse(i: &str) -> IResult<&str, Self> {
        let mut grid = Self {
            black_tiles: HashSet::new(),
        };
        let (i, mut paths) = separated_list1(line_ending, PathToTile::parse)(i)?;
        while let Some(path) = paths.pop() {
            grid.process_path(&path);
        }
        Ok((i, grid))
    }

    fn count_black_tiles(&self) -> usize {
        self.black_tiles.len()
    }

    fn process_path(&mut self, path: &PathToTile) {
        let mut coord = Coordinate::origin();
        for dir in path.directions.iter() {
            coord = coord.apply(dir);
        }
        self.flip(coord);
    }

    fn flip(&mut self, coordinate: Coordinate) {
        if !self.black_tiles.remove(&coordinate) {
            self.black_tiles.insert(coordinate);
        }
    }

    pub fn tile_to_num_neighbors(&self) -> HashMap<Coordinate, usize> {
        let mut map = HashMap::new();
        for tile in self.black_tiles.iter() {
            for dir in Direction::iter() {
                let neighbor = tile.apply(&dir);
                *map.entry(neighbor).or_insert(0) += 1;
            }
        }
        map
    }

    pub fn update_day(&mut self) {
        let neighbors = self.tile_to_num_neighbors();
        let mut updated_black_tiles = HashSet::new();

        for bt in self.black_tiles.iter() {
            let num_neighbors = neighbors.get(bt).unwrap_or(&0);
            if *num_neighbors == 1 || *num_neighbors == 2 {
                updated_black_tiles.insert(*bt);
            }
        }
        for wt in neighbors
            .iter()
            .filter_map(|(tile, count)| if *count == 2 { Some(tile) } else { None })
        {
            updated_black_tiles.insert(*wt);
        }

        self.black_tiles = updated_black_tiles;
    }

    pub fn update_n_days(&mut self, num_days: usize) {
        for i in 0..num_days {
            self.update_day();
            eprintln!("Day {:>3}: {:>4}", i + 1, self.count_black_tiles());
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
        let grid = Grid::from(input.as_str());
        assert_eq!(grid.count_black_tiles(), 10);
        Ok(())
    }

    #[test]
    fn part2_debug() -> Result<()> {
        let input = read_to_string(&PathBuf::from("debug.txt"))?;
        let mut grid = Grid::from(input.as_str());
        assert_eq!(grid.count_black_tiles(), 10);
        grid.update_n_days(100);
        assert_eq!(grid.count_black_tiles(), 2208);
        Ok(())
    }
}
