use anyhow::{bail, Context, Result};
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

    let groups = Group::read_from(&input)?;

    part1(&groups[..])?;
    part2(&groups[..])?;

    Ok(())
}

struct Group {
    answers_common: HashSet<char>,
    answers_unique: HashSet<char>,
}

impl Group {
    pub fn read_from(path: &Path) -> Result<Vec<Self>> {
        let mut answers_common = HashSet::new();
        let mut answers_unique = HashSet::new();
        let mut rv = Vec::new();
        let mut first_entry = true;

        for line in io::BufReader::new(File::open(&path)?).lines() {
            match line? {
                line if line.is_empty() => {
                    rv.push(Self {
                        answers_common,
                        answers_unique,
                    });
                    answers_unique = HashSet::new();
                    answers_common = HashSet::new();
                    first_entry = true;
                }
                line => {
                    let current: HashSet<_> = line.chars().collect();
                    answers_unique = answers_unique.union(&current).cloned().collect();
                    if first_entry {
                        answers_common = current;
                    } else {
                        answers_common = answers_common.intersection(&current).cloned().collect();
                    }
                    first_entry = false;
                }
            }
        }
        rv.push(Self {
            answers_common,
            answers_unique,
        });

        Ok(rv)
    }

    pub fn count_common(&self) -> usize {
        self.answers_common.len()
    }

    pub fn count_unique(&self) -> usize {
        self.answers_unique.len()
    }
}

fn part1(groups: &[Group]) -> Result<()> {
    let num: usize = groups.iter().map(|g| g.count_unique()).sum();
    println!("(part1) Number of yes-answers: {}", num);
    Ok(())
}

fn part2(groups: &[Group]) -> Result<()> {
    let num: usize = groups.iter().map(|g| g.count_common()).sum();
    println!("(part1) Number of common yes-answers: {}", num);
    Ok(())
}
