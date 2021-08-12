#![allow(unused_imports)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space0},
    combinator::{map, map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = read_to_string(&PathBuf::from("input.txt"))?;
    part1(&input)?;

    Ok(())
}

fn part1(input: &str) -> Result<()> {
    if let Ok((_, nums)) = parse_input(input).finish()
    {
        assert_eq!(nums.len(), 2);
        let loop_size_first = HandShake::find_loop_size(7, nums[0]);
        let loop_size_second = HandShake::find_loop_size(7, nums[1]);

        println!("Loop sizes: {}/{}", loop_size_first, loop_size_second);
        let encryption_key = {
            let mut hs = HandShake::new(nums[0]);
            hs.transform_n(loop_size_second);
            hs.value
        };

        println!("Encryption Key: {}", encryption_key);
    }
    else
    {
        panic!("Could not parse input numbers.");
    }
    Ok(())
}

fn parse_input(i: &str) -> IResult<&str, Vec<usize>> {
    map(separated_list1(line_ending, digit1), |vec: Vec<&str>| {
        vec.into_iter()
            .map(|s: &str| s.parse::<usize>().unwrap())
            .collect()
    })(i)
}

struct HandShake {
    value: usize,
    subject_number: usize,
}

impl HandShake {
    pub fn new(subject_number: usize) -> Self {
        Self {
            subject_number,
            value: 1,
        }
    }

    fn transform(&mut self) {
        self.value = (self.value * self.subject_number) % DIVIDER;
    }

    fn transform_n(&mut self, num: usize) {
        for _ in 0..num {
            self.transform();
        }
    }

    fn find_loop_size(subject_number: usize, target: usize) -> usize {
        let mut hs = HandShake::new(subject_number);
        let mut loop_count = 0;
        while hs.value != target {
            hs.transform();
            loop_count += 1;
        }
        loop_count
    }
}

const DIVIDER: usize = 20201227;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_part1() {
        assert_eq!(HandShake::find_loop_size(7, 5764801), 8);
        assert_eq!(HandShake::find_loop_size(7, 17807724), 11);

        let mut hs = HandShake::new(17807724);
        hs.transform_n(8);
        assert_eq!(hs.value, 14897079);
    }
}
