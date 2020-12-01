use anyhow::{Context, Result};
use std::collections::HashSet;
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
    println!("Input: {}", input.display());
    part1(&input)?;
    part2(&input)?;
    Ok(())
}

fn part1(input: &Path) -> Result<()> {
    let mut parsed: HashSet<usize> = HashSet::new();
    let target = 2020;

    for (i, line) in io::BufReader::new(File::open(input)?).lines().enumerate() {
        let num = line
            .with_context(|| format!("Could not read line {}", i))?
            .parse::<usize>()?;

        for other in parsed.iter() {
            if num + other == target {
                println!("Result: {}", num * other);
                return Ok(());
            }
        }
        parsed.insert(num);
    }
    Ok(())
}

fn part2(input: &Path) -> Result<()> {
    let mut parsed: HashSet<usize> = HashSet::new();
    let target = 2020;

    for (i, line) in io::BufReader::new(File::open(input)?).lines().enumerate() {
        let num = line
            .with_context(|| format!("Could not read line {}", i))?
            .parse::<usize>()?;

        for first in parsed.iter() {
            for second in parsed.iter() {
                if first == second {
                    continue;
                }
                if num + first + second == target {
                    println!("Result: {}", num * first * second);
                    return Ok(());
                }
            }
        }
        parsed.insert(num);
    }
    Ok(())
}
