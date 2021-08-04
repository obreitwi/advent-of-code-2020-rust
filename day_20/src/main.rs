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
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .nth(1)
            .with_context(|| "No input provided!")?,
    );

    let tileset = TileSet::read_from(&input)?;
    let pic = part1(tileset)?;

    /*
     * let first = TileSet.tiles.values().next().unwrap();
     * eprintln!("{:?}", first.edge_n());
     * eprintln!("{:?}", first.edge_e());
     * eprintln!("{:?}", first.edge_s());
     * eprintln!("{:?}", first.edge_w());
     */

    // eprintln!("{}", TileSet);

    Ok(())
}

fn part1(ts: TileSet) -> Result<Picture> {
    let adj = ts.adjacencies();
    eprintln!("{:#?}", adj);

    let corners: Vec<_> = adj
        .iter()
        .filter_map(|(idx, vec)| if vec.len() == 2 { Some(*idx) } else { None })
        .collect();

    println!("{:?}", corners);
    let corners_prod = corners.iter().cloned().product::<usize>();
    println!("{}", corners_prod);

    let pic = Picture::assemble(ts)?;
    let corners_assembled: Vec<_> = pic
        .grid
        .iter()
        .filter_map(|(pos, weak_tile)| {
            if pic.num_neighbors(*pos) == 2 {
                Some(weak_tile.upgrade().unwrap().borrow().idx)
            } else {
                None
            }
        })
        .collect();
    println!("{:?}", corners_assembled);
    let corners_assembled_prod = corners_assembled.iter().cloned().product::<usize>();
    println!("{}", corners_assembled_prod);

    assert_eq!(corners_prod, corners_assembled_prod);

    Ok(pic)
}

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
enum Orientation {
    North,
    East,
    South,
    West,
}

const ORIENTATIONS: [Orientation; 4] = [
    Orientation::North,
    Orientation::East,
    Orientation::South,
    Orientation::West,
];

impl From<Orientation> for usize {
    fn from(o: Orientation) -> usize {
        use Orientation::*;
        match o {
            North => 0,
            East => 1,
            South => 2,
            West => 3,
        }
    }
}

impl Orientation {
    fn opposite(&self) -> Orientation {
        use Orientation::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}

#[derive(Debug, Hash, PartialEq)]
struct Tile {
    idx: usize,
    data: Vec<char>,
    size: usize,
    orientation: Orientation,
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

    fn rotate(&mut self) {
        use Orientation::*;
        self.orientation = match self.orientation {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }

    fn edge(&self, side: Orientation) -> Vec<char> {
        use Orientation::*;
        match self.orientation {
            North => match side {
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

            East => match side {
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
            South => match side {
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
            West => match side {
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

    fn edge_raw(&self, side: Orientation) -> Vec<char> {
        use Orientation::*;

        match side {
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

    fn edge_match_approx(i: &[char], j: &[char]) -> bool {
        i.iter().zip(j.iter()).all(|(i, j)| i == j)
            || i.iter().rev().zip(j.iter()).all(|(i, j)| i == j)
    }

    fn edge_match(i: &[char], j: &[char]) -> bool {
        i.iter().zip(j.iter()).all(|(i, j)| i == j)
    }

    fn count_matching_edges(&self, other: &Self) -> usize {
        let edges_self = self.edges();
        let edges_other = other.edges();
        let mut count = 0;

        for e_i in edges_self.iter() {
            for e_j in edges_other.iter() {
                if Self::edge_match_approx(&e_i[..], &e_j[..]) {
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
struct TileSet {
    tiles: HashMap<usize, Rc<MutTile>>,
}

impl fmt::Display for TileSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        for tile in self.tiles.values() {
            write!(f, "{}\n", tile.borrow())?;
        }
        Ok(())
    }
}

impl TileSet {
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
                    tiles: tiles
                        .into_iter()
                        .map(|t| (t.idx, Rc::new(RefCell::new(t))))
                        .collect(),
                })
            }
            Err(e) => bail!("Error while parsing: {}", e),
        }
    }

    fn adjacencies(&self) -> HashMap<usize, Vec<usize>> {
        let mut rv = HashMap::new();

        for tile in self.tiles.values() {
            for to_check in self.tiles.values() {
                let tile = tile.borrow();
                let to_check = to_check.borrow();
                if tile.idx == to_check.idx {
                    continue;
                }
                let num_matches = tile.count_matching_edges(&to_check);

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

type Position = (i64, i64);
type MutTile = RefCell<Tile>;

fn advance(pos: Position, side: Orientation) -> Position {
    use Orientation::*;
    let (x, y) = pos;
    match side {
        North => (x, y + 1),
        East => (x + 1, y),
        South => (x, y - 1),
        West => (x - 1, y),
    }
}

#[derive(Debug)]
struct Picture {
    tiles: HashMap<usize, Rc<MutTile>>,
    grid: HashMap<Position, Weak<MutTile>>,
}

impl Picture {
    fn assemble(tileset: TileSet) -> Result<Self> {
        let tiles = tileset.tiles;
        let grid = HashMap::new();
        let mut assembled: HashSet<usize> = HashSet::new();

        let mut pic = Self { grid, tiles };

        let mut queue: VecDeque<(Position, usize)> = VecDeque::new();
        let first = pic
            .tiles
            .keys()
            .cloned()
            .next()
            .with_context(|| format!("No tiles given!"))?;
        queue.push_back(((0, 0), first));
        assembled.insert(first);

        'all: while let Some((pos, current_idx)) = queue.pop_front() {
            eprintln!("Checking #{} at {:?}", current_idx, pos);
            let current_tile = { pic.tiles.get(&current_idx).unwrap().borrow() };
            let others: Vec<_> = pic
                .tiles
                .keys()
                .filter(|idx| !assembled.contains(idx))
                .cloned()
                .collect();
            'tiles: for other_idx in others.into_iter() {
                let other_tile_rc = pic.tiles.get(&other_idx).unwrap();
                for _ in 0..4 {
                    'sides: for side in ORIENTATIONS.iter() {
                        if pic.neighbor(pos, *side).is_some() {
                            continue 'sides;
                        }

                        if pic.insert_if_match(pos, &current_tile, *side, *other_tile_rc) {
                            queue.push_back((advance(pos, *side), other_idx));

                            if pic.num_neighbors(pos) == 4 {
                                continue 'all;
                            } else {
                                continue 'tiles;
                            }
                        }
                    }
                    other_tile.rotate();
                }
            }
        }
        let centered_grid = Self::center(pic.grid);
        Ok(Self {
            grid: centered_grid,
            tiles: pic.tiles,
        })
    }

    /// Check if tile_current and tile_other match on side. 
    ///
    /// Insert tile_other into grid if they do.
    fn insert_if_match(&mut self, pos: Position, tile_current: &Tile, side: Orientation, tile_other: Rc<MutTile>) -> bool
    {
        let other_tile = tile_other.borrow();
        let this_edge = tile_current.edge(side);
        let other_edge = other_tile.edge(side.opposite());
        if Tile::edge_match(&this_edge[..], &other_edge[..]) {
            eprintln!(
                "Found match between {} and {}",
                tile_current.idx, tile_other.borrow().idx
            );

            let pos_neighbor = advance(pos, side);
            eprintln!("Inserting #{} at {:?}", other_tile.idx, pos_neighbor);
            self.grid.insert(pos_neighbor, Rc::downgrade(tile_other));
            assembled.insert(other_idx);
            true
        }
        else {
            false
        }
    }

    fn center(map: HashMap<Position, Weak<MutTile>>) -> HashMap<Position, Weak<MutTile>> {
        let x_min = map.keys().map(|k| k.0).min().unwrap();
        let y_min = map.keys().map(|k| k.1).min().unwrap();
        map.into_iter()
            .map(|((x, y), v)| ((x - x_min, y - y_min), v))
            .collect()
    }

    pub fn neighbor(&self, pos: Position, side: Orientation) -> Option<Weak<MutTile>> {
        let pos_neighbor = advance(pos, side);
        self.grid.get(&pos_neighbor).cloned()
    }

    pub fn num_neighbors(&self, pos: Position) -> usize {
        ORIENTATIONS
            .iter()
            .filter(|o| self.neighbor(pos, **o).is_some())
            .count()
    }
}
