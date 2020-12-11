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

    Ok(())
}

#[derive(Clone)]
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

    fn read_line(line: &str) -> Result<Vec<Position>> {
        let mut rv: Vec<_> = Vec::new();
        for c in line.chars() {
            rv.push(Position::parse(c)?);
        }
        Ok(rv)
    }

    fn get_neighbors(&self, x: i64, y: i64) -> Vec<Position> {
        let mut rv = Vec::new();
        for dy in (-1)..2
        {
            for dx in (-1)..2
            {
                if dx == 0 && dy == 0
                {
                    continue;
                }
                rv.push(self.get_pos(x + dx, y + dy));
            }
        }
        rv
    }

    fn get_pos(&self, x: i64, y: i64) -> Position
    {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height
        {
            Position::Floor
        }
        else {
            self.lines[y as usize][x as usize].clone()
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
