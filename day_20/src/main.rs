#![allow(unused_imports)]
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
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    let tileset = TileSet::read_from(&input)?;

    let pic = part1(tileset)?;

    eprintln!("{:#?}", pic);

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

    // Corners are tiles with exactly to matching neighbors
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
    println!("Corners assembled: {:?}", corners_assembled);
    let corners_assembled_prod = corners_assembled.iter().cloned().product::<usize>();
    println!("Corners assembled product: {}", corners_assembled_prod);

    assert_eq!(
        corners_prod, corners_assembled_prod,
        "New way of assembling image does not lead to the same corners!"
    );

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

#[derive(Debug, Hash, PartialEq, Clone)]
struct Tile {
    idx: usize,
    data: Edge,
    size: usize,
    orientation: Orientation,
    flipped: bool,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Tile {}:", self.idx)?;
        for line in self.lines().into_iter() {
            writeln!(f, "{}", line)?;
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
                flipped: false,
            },
        ))
    }

    fn flip(&mut self) {
        self.flipped = !self.flipped;
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

    pub fn column(&self, idx: usize) -> Edge {
        assert!(idx < self.size, "Invalid column access in #{}", idx);
        use Orientation::*;
        match self.orientation {
            North => self.column_with_flipped(idx),
            East => self.row_with_flipped(self.size - 1 - idx),
            South => reverse(self.column_with_flipped(self.size - 1 - idx)),
            West => reverse(self.row_with_flipped(idx)),
        }
    }

    pub fn row(&self, idx: usize) -> Edge {
        assert!(idx < self.size, "Invalid row access in #{}", idx);
        use Orientation::*;
        match self.orientation {
            North => self.row_with_flipped(idx),
            //  >N>
            // v   v
            // W   E
            // v   v
            //  >S>
            East => reverse(self.column_with_flipped(idx)),
            //  <W<
            // v   v
            // S   N
            // v   v
            //  <E<
            South => reverse(self.row_with_flipped(self.size - 1 - idx)),
            //   <S<
            //  ^   ^
            //  E   W
            //  ^   ^
            //   <N<
            West => self.column_with_flipped(self.size - 1 - idx),
            //  >E>
            // ^   ^
            // N   S
            // ^   ^
            //  >W>
        }
    }

    fn column_with_flipped(&self, idx: usize) -> Edge {
        let col = self.column_from_data(idx);
        match self.flipped {
            true => reverse(col),
            false => col,
        }
    }

    fn row_with_flipped(&self, idx: usize) -> Edge {
        match self.flipped {
            false => self.row_from_data(idx),
            true => self.row_from_data(self.size - 1 - idx),
        }
    }

    fn column_from_data(&self, idx: usize) -> Edge {
        let mut c = 0;
        self.data
            .iter()
            .skip(idx)
            .filter(|_| {
                c += 1;
                c % self.size == 1 // 1 because we want the first to be retained
            })
            .cloned()
            .collect()
    }

    fn row_from_data(&self, idx: usize) -> Edge {
        self.data
            .iter()
            .skip(idx * self.size)
            .take(self.size)
            .cloned()
            .collect()
    }

    fn edge(&self, side: Orientation) -> Edge {
        use Orientation::*;
        match side {
            North => self.row(0),
            East => self.column(self.size - 1),
            South => self.row(self.size - 1),
            West => self.column(0),
        }
    }

    fn edges(&self) -> Vec<Edge> {
        use Orientation::*;
        vec![
            self.edge(North),
            self.edge(East),
            self.edge(South),
            self.edge(West),
        ]
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
            rv.push(self.row(i).into_iter().collect());
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
            writeln!(f, "{}", tile.borrow())?;
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
                    rv.entry(tile.idx)
                        .or_insert_with(Vec::new)
                        .push(to_check.idx);
                }
            }
        }
        rv
    }
}

type Edge = Vec<char>;
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
            .with_context(|| "No tiles given!")?;
        eprintln!("Inserting #{} at {:?}", first, (0, 0));
        queue.push_back(((0, 0), first));
        assembled.insert(first);

        'all: while let Some((pos, current_idx)) = queue.pop_front() {
            eprintln!("Checking #{} at {:?}", current_idx, pos);
            let current_tile_rc = pic.tiles.get(&current_idx).unwrap();
            let current_tile = current_tile_rc.borrow();
            #[allow(clippy::needless_collect)] // needed to borrow pic mutable later
            let others: Vec<_> = pic
                .tiles
                .keys()
                .filter(|idx| !assembled.contains(idx))
                .cloned()
                .collect();
            eprintln!("others: {:#?}", others);
            'tiles: for other_idx in others.into_iter() {
                let other_tile_rc = pic.tiles.get(&other_idx).unwrap();
                for idx_try in 0..ORIENTATIONS.len() * 2 {
                    eprintln!("Current:\n{}", current_tile);
                    eprintln!("Other:\n{}", other_tile_rc.borrow());
                    'sides: for side in ORIENTATIONS.iter() {
                        if pic.neighbor(pos, *side).is_some() {
                            continue 'sides;
                        }

                        if pic.check_match(&current_tile, *side, &other_tile_rc.borrow()) {
                            assembled.insert(other_idx);

                            let pos_neighbor = advance(pos, *side);
                            eprintln!(
                                "Inserting #{} at {:?}",
                                other_tile_rc.borrow().idx,
                                pos_neighbor
                            );
                            pic.grid.insert(pos_neighbor, Rc::downgrade(other_tile_rc));
                            queue.push_back((pos_neighbor, other_idx));

                            if pic.num_neighbors(pos) == 4 {
                                continue 'all;
                            } else {
                                continue 'tiles;
                            }
                        }
                    }
                    let mut other_tile = other_tile_rc.borrow_mut();
                    other_tile.rotate();
                    if idx_try == ORIENTATIONS.len() - 1 {
                        other_tile.flip();
                    }
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
    fn check_match(&self, tile_current: &Tile, side: Orientation, tile_other: &Tile) -> bool {
        let other_tile = tile_other;
        let this_edge = tile_current.edge(side);
        let other_edge = other_tile.edge(side.opposite());
        if Tile::edge_match(&this_edge[..], &other_edge[..]) {
            eprintln!(
                "Found match between {} and {}",
                tile_current.idx, tile_other.idx
            );
            true
        } else {
            false
        }
    }

    fn center(map: HashMap<Position, Weak<MutTile>>) -> HashMap<Position, Weak<MutTile>> {
        let x_min = map.keys().map(|k| k.0).min().expect("No tiles in map.");
        let y_min = map.keys().map(|k| k.1).min().expect("No tiles in map.");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_test() -> Result<()> {
        use Orientation::*;
        let ts = TileSet::read_from(&PathBuf::from("debug-single.txt"))?;
        let mut tile = ts.tiles.get(&1337).unwrap().borrow_mut().clone();

        for _ in 0..2 {
            for _ in 0..ORIENTATIONS.len() {
                eprintln!("Orientation: {:?}", tile.orientation);
                eprintln!("{}", tile);
                assert_eq!(tile.edge(North), tile.row(0));
                assert_eq!(tile.edge(South), tile.row(tile.size - 1));
                assert_eq!(tile.edge(West), tile.column(0));
                assert_eq!(tile.edge(East), tile.column(tile.size - 1));
                tile.rotate();
            }
            eprintln!("Flipping!");
            tile.flip();
        }
        Ok(())
    }
}
