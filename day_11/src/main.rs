use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
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

    let grid = Grid::read_from(&input)?;
    println!("{}", grid);

    part1(grid.clone())?;
    part2(grid.clone())?;

    Ok(())
}

fn part1(grid: Grid) -> Result<()> {
    let fixed = grid.update_till_fixed();
    let num_occupied = fixed.count(Position::Occupied);

    println!("(part1) Number of occupied seats: {}", num_occupied);

    Ok(())
}

fn part2(grid: Grid) -> Result<()> {
    let fixed = grid.update_directional_till_fixed();
    let num_occupied = fixed.count(Position::Occupied);

    println!("(part2) Number of occupied seats: {}", num_occupied);

    Ok(())
}

#[derive(Clone, PartialEq)]
enum Position {
    Floor,
    Empty,
    Occupied,
}

impl Position {
    pub fn parse(c: char) -> Result<Self> {
        use Position::*;
        if c == '.' {
            Ok(Floor)
        } else if c == 'L' {
            Ok(Empty)
        } else if c == '#' {
            Ok(Occupied)
        } else {
            bail!("Invalid char for position: {}", c);
        }
    }

    pub fn count_in(&self, slice: &[Position]) -> usize {
        slice.iter().filter(move |e| self == *e).count()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Position::*;
        let c = match self {
            Floor => '.',
            Empty => 'L',
            Occupied => '#',
        };
        write!(f, "{}", c)
    }
}

#[derive(Clone, PartialEq)]
struct Grid {
    lines: Vec<Vec<Position>>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn read_from(path: &Path) -> Result<Self> {
        let mut raw_lines = io::BufReader::new(File::open(&path)?).lines();

        let mut lines = Vec::new();
        let first_raw_line = raw_lines.next().with_context(|| "Input has no lines")??;
        let first_line = Self::read_line(&first_raw_line)?;

        let width = first_line.len();
        lines.push(first_line);

        for raw in raw_lines {
            let parsed = Self::read_line(&raw?)?;
            if parsed.len() != width {
                bail!("Lines do not have the same width..");
            }
            lines.push(parsed);
        }

        let height = lines.len();
        Ok(Self {
            lines,
            width,
            height,
        })
    }

    pub fn count(&self, position: Position) -> usize {
        let mut count = 0;
        for line in self.lines.iter() {
            count += position.count_in(&line[..]);
        }
        count
    }

    pub fn update(&self) -> Self {
        let mut lines = Vec::with_capacity(self.height);

        for y in 0..(self.height as i64) {
            let mut newline = Vec::with_capacity(self.width);
            for x in 0..(self.width as i64) {
                newline.push(self.update_at(x, y));
            }
            lines.push(newline);
        }

        Self { lines, ..*self }
    }

    pub fn update_directional(&self) -> Self {
        let mut lines = Vec::with_capacity(self.height);

        for y in 0..(self.height as i64) {
            let mut newline = Vec::with_capacity(self.width);
            for x in 0..(self.width as i64) {
                newline.push(self.update_directional_at(x, y));
            }
            lines.push(newline);
        }

        Self { lines, ..*self }
    }

    pub fn update_till_fixed(self) -> Self {
        let mut old = self;
        let mut step = 0;

        println!("Initial: {}", old);

        loop {
            step += 1;
            let new = old.update();
            println!("After step #{}: {}", step, new);

            if old == new {
                return new;
            } else {
                old = new;
            }
        }
    }

    pub fn update_directional_till_fixed(self) -> Self {
        let mut old = self;
        let mut step = 0;

        println!("Initial: {}", old);

        loop {
            step += 1;
            let new = old.update_directional();
            println!("After step #{}: {}", step, new);

            if old == new {
                return new;
            } else {
                old = new;
            }
        }
    }

    fn read_line(line: &str) -> Result<Vec<Position>> {
        let mut rv: Vec<_> = Vec::new();
        for c in line.chars() {
            rv.push(Position::parse(c)?);
        }
        Ok(rv)
    }

    fn reached_edge(&self, x: i64, y: i64) -> bool {
        x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64
    }

    fn get_neighbors(&self, x: i64, y: i64) -> Vec<Position> {
        let mut rv = Vec::new();
        for dy in (-1)..2 {
            for dx in (-1)..2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                rv.push(self.get_pos(x + dx, y + dy));
            }
        }
        rv
    }

    fn get_neighbors_directional(&self, x: i64, y: i64) -> Vec<Position> {
        let mut rv = Vec::new();
        for dy in (-1)..2 {
            for dx in (-1)..2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut multiplier = 1;
                while !self.reached_edge(x + dx * multiplier, y + dy * multiplier) {
                    let neighbour_x = x + multiplier * dx;
                    let neighbour_y = y + multiplier * dy;
                    match self.get_pos(neighbour_x, neighbour_y) {
                        Position::Floor if self.reached_edge(neighbour_x, neighbour_y) => {
                            rv.push(Position::Floor)
                        }
                        Position::Floor => {
                            multiplier += 1;
                            continue;
                        }
                        other => {
                            rv.push(other);
                            break;
                        }
                    }
                }
            }
        }
        rv
    }

    fn get_pos(&self, x: i64, y: i64) -> Position {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            Position::Floor
        } else {
            self.lines[y as usize][x as usize].clone()
        }
    }

    fn update_at(&self, x: i64, y: i64) -> Position {
        use Position::*;

        let get_num_occupied = || -> usize { Occupied.count_in(&self.get_neighbors(x, y)[..]) };

        match self.get_pos(x, y) {
            Floor => Floor,
            Empty => {
                if get_num_occupied() == 0 {
                    Occupied
                } else {
                    Empty
                }
            }
            Occupied => {
                if get_num_occupied() >= 4 {
                    Empty
                } else {
                    Occupied
                }
            }
        }
    }

    fn update_directional_at(&self, x: i64, y: i64) -> Position {
        use Position::*;

        let get_num_occupied =
            || -> usize { Occupied.count_in(&self.get_neighbors_directional(x, y)[..]) };

        match self.get_pos(x, y) {
            Floor => Floor,
            Empty => {
                if get_num_occupied() == 0 {
                    Occupied
                } else {
                    Empty
                }
            }
            Occupied => {
                if get_num_occupied() >= 5 {
                    Empty
                } else {
                    Occupied
                }
            }
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Grid ({}x{})\n", self.width, self.height)?;
        for line in self.lines.iter() {
            for pos in line.iter() {
                write!(f, "{}", pos)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
