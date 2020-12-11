use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
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

    let numbers = read_numbers(&input)?;

    part1(&numbers)?;
    part2(&numbers)?;

    Ok(())
}

fn read_numbers(path: &Path) -> Result<Vec<u64>> {
    let mut numbers = Vec::new();

    numbers.push(0); // beginning
    for line in io::BufReader::new(File::open(&path)?).lines() {
        numbers.push(line?.parse()?);
    }
    numbers.push(numbers.iter().max().unwrap() + 3); // final charger
    Ok(numbers)
}

fn diff(numbers: &[u64]) -> Vec<u64> {
    let mut sorted: Vec<u64> = numbers.iter().map(|v| *v).collect();

    sorted.sort();

    let mut iter_pre = sorted.iter();
    let mut iter_post = sorted.iter();
    iter_post.next();

    let mut rv = Vec::new();

    while let Some(post) = iter_post.next() {
        let pre = iter_pre.next().unwrap();

        let diff = post - pre;
        rv.push(diff);
    }
    rv
}

fn _count_combinations_v1_wrong(diffs: &[u64], max_diff: u64) -> u64 {
    println!("Counting for: {:?}", diffs);

    let mut iter = diffs.iter();

    let mut current = iter.next().unwrap();

    let mut counts = 1;
    let mut idx_current: usize = 0;

    while let Some(next) = iter.next() {
        if current + next <= max_diff {
            let idx_next = idx_current + 1;
            let smaller: Vec<u64> = diffs
                .iter()
                .enumerate()
                .filter_map(|(i, e)| match i {
                    i if i == idx_next => None,
                    i if i == idx_current => Some(current + next),
                    _ => Some(*e),
                })
                .collect();

            counts += _count_combinations_v1_wrong(&smaller[..], max_diff);
        }
        current = next;
        idx_current += 1;
    }

    counts
}

fn count_combinations(numbers: &[u64], max_diff: u64) -> u64 {
    let mut numbers: Vec<_> = numbers.iter().map(|e| *e).collect();
    numbers.sort();
    count_combinations_inner(&numbers[..], max_diff)
}

fn count_combinations_inner(numbers: &[u64], max_diff: u64) -> u64 {
    if numbers.len() == 1 {
        return 1;
    } else {
        let current = numbers[0];

        let mut counts = 0;
        let mut idx_next = 1;
        while idx_next < numbers.len() && numbers[idx_next] <= current + max_diff {
            counts += count_combinations_inner(&numbers[idx_next..], max_diff);

            idx_next += 1;
        }

        counts
    }
}

struct AdapterChain {
    adapters_sorted: Vec<u64>,
    adapters: HashSet<u64>,
    max_diff: u64,
}

impl AdapterChain {
    pub fn new(numbers: &[u64], max_diff: u64) -> Self {
        let mut adapters_sorted: Vec<_> = numbers.iter().map(|e| *e).collect();
        adapters_sorted.sort();

        let adapters: HashSet<_> = numbers.iter().map(|e| *e).collect();

        Self {
            adapters,
            adapters_sorted,
            max_diff,
        }
    }

    pub fn count(&self) -> u64 {
        let mut adapter_to_count: HashMap<u64, u64> = HashMap::new();
    }

    fn links_from(&self, adapter: u64) -> Vec<u64> {
        let mut rv = Vec::new();
        for diff in 1..(self.max_diff + 1) {
            if diff >= adapter {
                let link = adapter - diff;
                if self.contains(link) {
                    rv.push(link);
                }
            }
        }
        rv
    }

    fn links_to(&self, adapter: u64) -> Vec<u64> {
        let mut rv = Vec::new();
        for diff in 1..(self.max_diff + 1) {
            let link = adapter + diff;
            if self.contains(link) {
                rv.push(link);
            }
        }
        rv
    }
}

fn part1(numbers: &[u64]) -> Result<()> {
    let mut diff_to_count: HashMap<u64, u64> = HashMap::new();

    for d in diff(numbers) {
        *diff_to_count.entry(d).or_insert(0) += 1;
    }

    let num_1: u64 = *diff_to_count.get(&1).unwrap_or(&0);
    let num_3: u64 = *diff_to_count.get(&3).unwrap_or(&0);
    println!("(part1) Number of 1-jumps: {}", num_1);
    println!("(part1) Number of 3-jumps: {}", num_3);

    println!("(part1) Multiplication of the two: {}", num_1 * num_3);

    Ok(())
}

fn part2(numbers: &[u64]) -> Result<()> {
    let counts = count_combinations(&numbers[..], 3);

    println!("Found {} combinations..", counts);

    Ok(())
}
