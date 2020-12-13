use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    Finish, IResult,
};
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

    let busses = Busses::read_from(&input)?;

    part1(&busses);

    Ok(())
}

fn part1(busses: &Busses) {
    let (bus, ttw) = busses.get_next_bus_wait_time();

    println!("Next line: {}", bus);
    println!("Time to wait: {}", ttw);

    println!("(part1) Answer: {}", bus * ttw);
}

struct Busses {
    starttime: usize,
    busses: Vec<usize>,
}

impl Busses {
    pub fn read_from(input: &Path) -> Result<Self>
    {
        let mut lines = io::BufReader::new(File::open(&input)?).lines();
        let starttime = lines.next().unwrap()?.parse()?;

        let line_busses = lines.next().with_context(|| "No line with bus information provided.")??;

        let mut busses = Vec::new();
        for potential_bus in line_busses.split(",")
        {
            match potential_bus
            {
                "x" => {/* skip */}
                id => {
                    busses.push(id.parse().with_context(|| format!("Invalid bus line: {}", id))?);
                }
            }
        }

        busses.sort();

        Ok(Self {starttime, busses})
    }

    pub fn get_next_bus_wait_time(&self) -> (usize, usize) {
        let mut next_bus = 0;
        let mut time_to_wait = usize::MAX;

        for bus in self.busses.iter() {
            let wait_time = bus - self.starttime % bus;
            if wait_time < time_to_wait
            {
                time_to_wait = wait_time;
                next_bus = bus.clone();
            }
        }

        (next_bus, time_to_wait)
    }
}
