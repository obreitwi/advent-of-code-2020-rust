use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, line_ending, none_of, not_line_ending, one_of, space0,
    },
    combinator::{map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let picture = Picture::read_from(&input)?;
    part1(&picture)?;

    /*
     * let first = picture.tiles.values().next().unwrap();
     * eprintln!("{:?}", first.edge_n());
     * eprintln!("{:?}", first.edge_e());
     * eprintln!("{:?}", first.edge_s());
     * eprintln!("{:?}", first.edge_w());
     */

    // eprintln!("{}", picture);

    Ok(())
}

fn part1(pic: &Picture) -> Result<()> {
    let adj = pic.adjacencies();
    eprintln!("{:#?}", adj);

    let corners: Vec<_> = adj
        .iter()
        .filter_map(|(idx, vec)| if vec.len() == 2 { Some(*idx) } else { None })
        .collect();

    println!("{:?}", corners);
    println!("{}", corners.iter().cloned().product::<usize>());

    Ok(())
}

#[derive(Debug, PartialEq, Hash)]
enum Orientation {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Hash, PartialEq)]
struct Tile {
    idx: usize,
    data: Vec<char>,
    size: usize,
    orientation: Orientation,
}

impl AsRef<Orientation> for Orientation {
    fn as_ref(&self) -> &Orientation {
        self
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        write!(f, "Tile {}:\n", self.idx)?;
        for line in self.lines().into_iter() {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

fn reverse<T>(mut v: Vec<T>) -> Vec<T> {
    v.reverse();
    v
}

impl Tile {
    fn parse(i: &str) -> IResult<&str, Self> {
        let mut parse_idx = map_res(
            delimited(tag("Tile "), digit1, tuple((char(':'), line_ending))),
            |l: &str| l.parse::<usize>(),
        );
        let (i, idx) = parse_idx(i)?;
        let (_, first) = not_line_ending(i)?;
        let size = first.chars().count();

        eprintln!("Parsed index {}, found size: {}", idx, size);

        let (i, lines) = separated_list1(line_ending, take_while1(|c| c == '.' || c == '#'))(i)?;

        let mut data = Vec::with_capacity(size * size);
        for line in lines {
            data.extend(line.chars());
        }
        Ok((
            i,
            Self {
                idx,
                size,
                data,
                orientation: Orientation::North,
            },
        ))
    }

    fn edge<O: AsRef<Orientation>>(&self, side: O) -> Vec<char> {
        use Orientation::*;
        match self.orientation {
            North => match side.as_ref() {
                //  >N>
                // v   v
                // W   E
                // v   v
                //  >S>
                North => self.edge_raw(North), 
                East => self.edge_raw(East),   //
                South => self.edge_raw(South), //
                West => self.edge_raw(West),   //
            },

            East => match side.as_ref() {
                //  <W<
                // v   v
                // S   N
                // v   v
                //  <E<
                North => reverse(self.edge_raw(West)),
                East => self.edge_raw(North),
                South => reverse(self.edge_raw(East)),
                West => self.edge_raw(South),
            },
            South => match side.as_ref() {
                //   <S<
                //  ^   ^
                //  E   W
                //  ^   ^
                //   <N<
                North => reverse(self.edge_raw(South)),
                East => reverse(self.edge_raw(West)),
                South => reverse(self.edge_raw(North)),
                West => reverse(self.edge_raw(East)),
            },
            West => match side.as_ref() {
                //  <S<
                // ^   ^
                // E   W
                // ^   ^
                //  <N<
                North => self.edge_raw(East),
                East => reverse(self.edge_raw(South)),
                South => self.edge_raw(West),
                West => reverse(self.edge_raw(North)),
            },
        }
    }

    fn edge_raw<O: AsRef<Orientation>>(&self, side: O) -> Vec<char> {
        use Orientation::*;

        match side.as_ref() {
            North => self.data.iter().take(self.size).cloned().collect(),
            East => {
                let mut rv = Vec::with_capacity(self.size);
                for i in 0..self.size {
                    rv.push(self.data[i * self.size + self.size - 1]);
                }
                rv
            }
            South => self
                .data
                .iter()
                .skip(self.size * (self.size - 1))
                .take(self.size)
                .cloned()
                .collect(),
            West => {
                let mut rv = Vec::with_capacity(self.size);
                for i in 0..self.size {
                    rv.push(self.data[i * self.size]);
                }
                rv
            }
        }
    }

    fn edges(&self) -> Vec<Vec<char>> {
        use Orientation::*;
        let mut rv = Vec::with_capacity(4);
        rv.push(self.edge(North));
        rv.push(self.edge(East));
        rv.push(self.edge(South));
        rv.push(self.edge(West));
        rv
    }

    fn edge_match(i: &[char], j: &[char]) -> bool {
        i.iter().zip(j.iter()).all(|(i, j)| i == j)
            || i.iter().rev().zip(j.iter()).all(|(i, j)| i == j)
    }

    fn count_matching_edges(&self, other: &Self) -> usize {
        let edges_self = self.edges();
        let edges_other = other.edges();
        let mut count = 0;

        for e_i in edges_self.iter() {
            for e_j in edges_other.iter() {
                if Self::edge_match(&e_i[..], &e_j[..]) {
                    count += 1;
                }
            }
        }
        count
    }

    fn lines(&self) -> Vec<String> {
        let mut rv = Vec::new();

        for i in 0..self.size {
            rv.push(
                self.data
                    .iter()
                    .skip(i * self.size)
                    .take(self.size)
                    .collect(),
            );
        }
        rv
    }
}

#[derive(Debug)]
struct Picture {
    tiles: HashMap<usize, Tile>,
}

impl fmt::Display for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        for tile in self.tiles.values() {
            write!(f, "{}\n", tile)?;
        }
        Ok(())
    }
}

impl Picture {
    fn read_from(input: &Path) -> Result<Self> {
        let input = read_to_string(input)?;
        eprintln!("Read to string:\n{}", input);
        let parsed = separated_list1(many1(line_ending), Tile::parse)(&input).finish();

        match parsed {
            Ok((i, tiles)) => {
                if i.len() > 2 {
                    bail!("ERR: {} bytes left..\n{}", i.len(), i);
                }

                Ok(Self {
                    tiles: tiles.into_iter().map(|t| (t.idx, t)).collect(),
                })
            }
            Err(e) => bail!("Error while parsing: {}", e),
        }
    }

    fn adjacencies(&self) -> HashMap<usize, Vec<usize>> {
        let mut rv = HashMap::new();

        for tile in self.tiles.values() {
            for to_check in self.tiles.values() {
                if tile.idx == to_check.idx {
                    continue;
                }
                let num_matches = tile.count_matching_edges(to_check);

                if num_matches > 1 {
                    eprintln!("{} <-> {}: {}", tile.idx, to_check.idx, num_matches);
                }

                if num_matches > 0 {
                    rv.entry(tile.idx).or_insert(Vec::new()).push(to_check.idx);
                }
            }
        }
        rv
    }
}
