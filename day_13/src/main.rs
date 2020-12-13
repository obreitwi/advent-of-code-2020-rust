use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    Finish, IResult,
};
use std::collections::HashMap;
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

    assert_eq!(Schedule::from("17,x,13,19")?.find_first_matching(), 3417);
    assert_eq!(Schedule::from("67,7,59,61")?.find_first_matching(), 754018);
    assert_eq!(
        Schedule::from("67,x,7,59,61")?.find_first_matching(),
        779210
    );
    assert_eq!(
        Schedule::from("67,7,x,59,61")?.find_first_matching(),
        1261476
    );
    assert_eq!(
        Schedule::from("1789,37,47,1889")?.find_first_matching(),
        1202161486
    );

    let sched = Schedule::read_from(&input)?;
    part2(&sched);

    Ok(())
}

fn part1(busses: &Busses) {
    let (bus, ttw) = busses.get_next_bus_wait_time();

    println!("Next line: {}", bus);
    println!("Time to wait: {}", ttw);

    println!("(part1) Answer: {}", bus * ttw);
}

fn part2(sched: &Schedule) {
    let timestep = sched.find_first_matching();
    println!("(part2) first timestep found: {}", timestep)
}

struct Busses {
    starttime: usize,
    busses: Vec<usize>,
}

impl Busses {
    pub fn read_from(input: &Path) -> Result<Self> {
        let mut lines = io::BufReader::new(File::open(&input)?).lines();
        let starttime = lines.next().unwrap()?.parse()?;

        let line_busses = lines
            .next()
            .with_context(|| "No line with bus information provided.")??;

        let mut busses = Vec::new();
        for potential_bus in line_busses.split(",") {
            match potential_bus {
                "x" => { /* skip */ }
                id => {
                    busses.push(
                        id.parse()
                            .with_context(|| format!("Invalid bus line: {}", id))?,
                    );
                }
            }
        }

        busses.sort();

        Ok(Self { starttime, busses })
    }

    pub fn get_next_bus_wait_time(&self) -> (usize, usize) {
        let mut next_bus = 0;
        let mut time_to_wait = usize::MAX;

        for bus in self.busses.iter() {
            let wait_time = bus - self.starttime % bus;
            if wait_time < time_to_wait {
                time_to_wait = wait_time;
                next_bus = bus.clone();
            }
        }

        (next_bus, time_to_wait)
    }
}

struct Schedule {
    bus_to_offset: HashMap<usize, usize>,
}

impl Schedule {
    pub fn read_from(input: &Path) -> Result<Self> {
        let mut lines = io::BufReader::new(File::open(&input)?).lines();
        lines.next(); // first line ignored

        let line_busses = lines
            .next()
            .with_context(|| "No line with bus information provided.")??;
        Self::from(&line_busses)
    }

    pub fn from(input: &str) -> Result<Self> {
        let mut bus_to_offset = HashMap::new();

        for (offset, potential_bus) in input.split(",").enumerate() {
            match potential_bus {
                "x" => {}
                id => {
                    bus_to_offset.insert(
                        id.parse()
                            .with_context(|| format!("Invalid bus line: {}", id))?,
                        offset,
                    );
                }
            }
        }

        Ok(Self { bus_to_offset })
    }

    pub fn find_first_matching(&self) -> usize {
        let busses = self.busses_sorted_reversed();
        let largest = busses[0];
        let offset_largest = self.bus_to_offset[&largest] as i64;

        let bus_to_relat_offset: HashMap<usize, i64> = self
            .bus_to_offset
            .iter()
            .map(|(k, v)| (*k, *v as i64 - offset_largest as i64))
            .collect();

        let mut timestep: i64 = 0;
        'a: loop {
            timestep += largest as i64;
            eprint!("\rChecking: {}", timestep);
            for bus in &busses[1..] {
                if (timestep + bus_to_relat_offset[bus]) % *bus as i64 != 0 {
                    continue 'a;
                }
            }

            eprint!("\r");
            return (timestep - offset_largest) as usize;
        }
    }

    fn busses_sorted_reversed(&self) -> Vec<usize> {
        let mut rv: Vec<_> = self.bus_to_offset.keys().cloned().collect();
        rv.sort();
        rv.reverse();
        rv
    }
}
