use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::convert::TryFrom;
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

    let grid = Grid::load(&input)?;

    part1(&grid)?;
    part2(&grid)?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum GridPos {
    Empty,
    Tree,
}

pub struct Grid {
    grid: HashMap<(usize, usize), GridPos>,
    size_x: usize,
    size_y: usize,
}

impl TryFrom<char> for GridPos {
    type Error = String;

    fn try_from(c: char) -> std::result::Result<Self, Self::Error> {
        match c {
            '#' => Ok(Self::Tree),
            '.' => Ok(Self::Empty),
            _ => Err(format!("Invalid input for GridPos: {}", c)),
        }
    }
}

impl Grid {
    pub fn load(filename: &Path) -> Result<Self> {
        let mut size_x = 0;
        let mut size_y = 0;
        let mut grid = HashMap::new();
        for (y, line) in io::BufReader::new(File::open(&filename)?)
            .lines()
            .enumerate()
        {
            size_x = 0;
            for (x, content) in line?.chars().enumerate() {
                match GridPos::try_from(content) {
                    Ok(gp) => {
                        grid.insert((x, y), gp);
                    }
                    Err(err) => bail!(err),
                }
                size_x += 1;
            }
            size_y += 1;
        }

        Ok(Self {
            grid,
            size_x,
            size_y,
        })
    }

    pub fn get_pos(&self, pos: (usize, usize)) -> Result<GridPos> {
        self.get(pos.0, pos.1)
    }

    pub fn get(&self, x: usize, y: usize) -> Result<GridPos> {
        if y >= self.size_y {
            bail!("{} is beyond size {} in y-direction.", y, self.size_y);
        }

        self.grid
            .get(&(x % self.size_x, y))
            .cloned()
            .with_context(|| format!("Position not in grid: {}/{}", x % self.size_y, y))
    }

    pub fn size_x(&self) -> usize {
        self.size_x
    }

    pub fn size_y(&self) -> usize {
        self.size_y
    }

    pub fn count_trees(&self, slope: &Slope) -> Result<usize> {
        let mut pos = (0, 0);

        let mut num_trees = 0;

        while pos.1 < self.size_y {
            if let GridPos::Tree = self.get_pos(pos)? {
                num_trees += 1;
            }
            pos.0 += slope.right;
            pos.1 += slope.down;
        }
        Ok(num_trees)
    }
}

pub struct Slope {
    pub right: usize,
    pub down: usize,
}

fn part1(grid: &Grid) -> Result<()> {
    let num_trees = grid.count_trees(&Slope{ right: 3, down: 1})?;
    println!("Part 1: Encountered {} trees", num_trees);
    Ok(())
}

fn part2(grid: &Grid) -> Result<()> {
    let slopes = vec![
        Slope{ right: 1, down: 1},
        Slope{ right: 3, down: 1},
        Slope{ right: 5, down: 1},
        Slope{ right: 7, down: 1},
        Slope{ right: 1, down: 2},
    ];
    let mut result = 1;

    for slope in slopes.iter() {
        result *= grid.count_trees(slope)?;
    }
    println!("Part 2: Result is {}", result);
    Ok(())
}
