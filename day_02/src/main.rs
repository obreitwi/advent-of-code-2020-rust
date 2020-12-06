use anyhow::{bail, Context, Result};
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let mut correct: usize = 0;
    for line in io::BufReader::new(File::open(&input)?).lines() {
        if validate_part1(&line?)? {
            correct += 1;
        }
    }
    println!("Correct passwords (part 1): {}", correct);

    let mut correct: usize = 0;
    for line in io::BufReader::new(File::open(&input)?).lines() {
        if validate_part2(&line?)? {
            correct += 1;
        }
    }
    println!("Correct passwords (part 2): {}", correct);
    Ok(())
}

fn validate_part1(s: &str) -> Result<bool> {
    let s: Vec<_> = s.split_whitespace().collect();

    if s.len() != 3 {
        bail!("Expected 3 parts, found {}.", s.len());
    }

    let range = s[0];
    let needed = s[1];
    let password = s[2];

    let range: Vec<_> = range.split('-').collect();
    if range.len() != 2 {
        bail!("Invalid range specified: {}", s[0]);
    }

    let occu_min: usize = range[0].parse()?;
    let occu_max: usize = range[1].parse()?;

    let needed: char = needed
        .strip_suffix(':')
        .with_context(|| "Could not strip colon!")?
        .chars()
        .next()
        .with_context(|| "No char in password policy")?;

    let count = password.chars().filter(|c| c == &needed).count();

    Ok(occu_min <= count && count <= occu_max)
}

fn validate_part2(s: &str) -> Result<bool> {
    let s: Vec<_> = s.split_whitespace().collect();

    if s.len() != 3 {
        bail!(format!("Expected 3 parts, found {}.", s.len()));
    }

    let positions = s[0];
    let needed = s[1];
    let password = s[2];

    let positions: Vec<_> = positions.split('-').collect();
    if positions.len() != 2 {
        bail!(format!("Invalid positions specified: {}", s[0]));
    }

    let pos_1: usize = positions[0].parse()?;
    let pos_2: usize = positions[1].parse()?;

    let needed: char = needed
        .strip_suffix(':')
        .with_context(|| "Could not strip colon!")?
        .chars()
        .next()
        .with_context(|| "No char in password policy")?;

    let chars: Vec<_> = password.chars().collect();

    let pos_1_contains = chars[pos_1 - 1] == needed;
    let pos_2_contains = chars[pos_2 - 1] == needed;

    Ok(pos_1_contains as usize + pos_2_contains as usize == 1)
}
