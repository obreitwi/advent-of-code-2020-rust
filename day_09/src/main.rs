use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
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
    let debug_numbers = vec![
        35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309, 576,
    ];
    assert_eq!(check_first_invalid(&debug_numbers[..], 5), Some(127));
    let debug_continuous = find_range(&debug_numbers[..], 127)?;
    // debug_continuous.debug();
    assert_eq!(debug_continuous.min() + debug_continuous.max(), 62);

    let numbers = read_numbers(&input)?;
    let target = part1(&numbers)?;
    part2(&numbers, target)?;

    Ok(())
}

fn read_numbers(path: &Path) -> Result<Vec<u64>> {
    let mut numbers = Vec::new();
    for line in io::BufReader::new(File::open(&path)?).lines() {
        numbers.push(line?.parse()?);
    }
    Ok(numbers)
}

struct CodingXMAS {
    preamble: VecDeque<u64>,
    sums_to_count: HashMap<u64, u64>,
}

impl CodingXMAS {
    pub fn new(initial: &[u64]) -> Self {
        let preamble = VecDeque::with_capacity(initial.len());
        let sums_to_count = HashMap::new();

        let mut new = Self {
            preamble,
            sums_to_count,
        };

        for num in initial.iter() {
            new.add(*num);
        }
        new
    }

    pub fn add(&mut self, num: u64) {
        if self.preamble.len() == self.preamble.capacity() {
            self.remove_oldest();
        }
        for existing in self.preamble.iter() {
            *self.sums_to_count.entry(num + existing).or_insert(0) += 1;
        }
        self.preamble.push_back(num);
    }

    pub fn check(&self, num: u64) -> bool {
        self.sums_to_count.contains_key(&num)
    }

    fn remove_oldest(&mut self) {
        let oldest = self.preamble.pop_front().unwrap();
        for existing in self.preamble.iter() {
            let sum = oldest + existing;
            match self.sums_to_count.get(&sum).unwrap() {
                1 => {
                    self.sums_to_count.remove(&sum).unwrap();
                }
                _ => {
                    self.sums_to_count.entry(sum).and_modify(|c| *c -= 1);
                }
            }
        }
    }
}

fn check_first_invalid(numbers: &[u64], size_preamble: usize) -> Option<u64> {
    let mut coding = CodingXMAS::new(&numbers[..size_preamble]);

    for num in numbers[size_preamble..].iter() {
        if !coding.check(*num) {
            return Some(*num);
        } else {
            coding.add(*num);
        }
    }
    None
}

fn part1(numbers: &[u64]) -> Result<u64> {
    let first_invalid =
        check_first_invalid(numbers, 25).with_context(|| "Found no invalid numbers!")?;
    println!("(part1) First invalid number: {}", first_invalid);
    Ok(first_invalid)
}

struct ContinuousXMAS {
    entries: VecDeque<u64>,
    sum: u64,
}

impl ContinuousXMAS {
    pub fn new(first: u64, second: u64) -> Self {
        let sum = first + second;
        let mut entries = VecDeque::new();
        entries.push_back(first);
        entries.push_back(second);
        Self { entries, sum }
    }

    pub fn add(&mut self, num: u64) {
        self.entries.push_back(num);
        self.sum += num;
    }

    pub fn debug(&self) {
        println!("Entries: {:#?}", self.entries)
    }

    pub fn max(&self) -> u64 {
        *self.entries.iter().max().unwrap()
    }

    pub fn min(&self) -> u64 {
        *self.entries.iter().min().unwrap()
    }

    pub fn sum(&self) -> u64 {
        self.sum
    }

    pub fn remove_oldest(&mut self) {
        if let Some(oldest) = self.entries.pop_front() {
            self.sum -= oldest;
        }
    }
}

fn find_range(numbers: &[u64], target: u64) -> Result<ContinuousXMAS> {
    let mut continuous = ContinuousXMAS::new(numbers[0], numbers[1]);
    let mut iter = numbers[2..].iter();

    while continuous.sum() != target {
        if continuous.sum() < target {
            continuous.add(*iter.next().with_context(|| "Ran out of numbers.")?);
        } else {
            continuous.remove_oldest();
        }
    }
    Ok(continuous)
}

fn part2(numbers: &[u64], target: u64) -> Result<()> {
    let continuous = find_range(numbers, target)?;
    println!(
        "(part2) Sum of first and last element: {}",
        continuous.min() + continuous.max()
    );

    Ok(())
}
