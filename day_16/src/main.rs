use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, none_of, one_of},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let notes = Notes::read_from(&input)?;
    // println!("Notes: {:#?}", notes);
    part1(&notes);
    part2(&notes)?;

    Ok(())
}

fn part1(notes: &Notes) {
    let rate = notes.ticket_scanning_error_rate();
    println!("(part1) Ticket scanning error rate: {}", rate);
}

fn part2(notes: &Notes) -> Result<()> {
    let fields = notes.infer_fields()?;

    // println!("Inferred fields: {:#?}", fields);

    let mut result: usize = 1;

    for (constr, value) in fields.iter().zip(notes.my_ticket.fields.iter()) {
        if constr.name.starts_with("departure") {
            result *= value;
        }
    }

    println!("(part2) Result: {}", result);

    Ok(())
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Range {
    from: usize,
    to: usize,
}

impl Range {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (from, to)) = separated_pair(digit1, char('-'), digit1)(i)?;
        Ok((
            i,
            Self {
                from: from.parse().unwrap(),
                to: to.parse().unwrap(),
            },
        ))
    }

    fn fits(&self, value: usize) -> bool {
        self.from <= value && value <= self.to
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FieldConstraint {
    name: String,
    ranges: Vec<Range>,
}

impl FieldConstraint {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, name) = terminated(is_not(":"), tuple((is_a(":"), multispace0)))(i)?;
        let (i, ranges) = separated_list1(tag(" or "), Range::parse)(i)?;
        Ok((
            i,
            Self {
                name: name.into(),
                ranges,
            },
        ))
    }

    fn fits(&self, value: usize) -> bool {
        for range in self.ranges.iter() {
            if range.fits(value) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
struct Ticket {
    fields: Vec<usize>,
}

impl Ticket {
    fn empty() -> Self {
        Self { fields: Vec::new() }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, values) = separated_list1(char(','), digit1)(i)?;
        let fields = values.iter().map(|v| v.parse().unwrap()).collect();
        Ok((i, Self { fields }))
    }

    fn len(&self) -> usize {
        self.fields.len()
    }
}

#[derive(Debug, Clone)]
struct Notes {
    constraints: Vec<FieldConstraint>,
    my_ticket: Ticket,
    tickets: Vec<Ticket>,
}

impl Notes {
    fn read_from(input: &Path) -> Result<Self> {
        let input = read_to_string(&input)?;

        match Self::parse(&input).finish() {
            Ok((i, notes)) => {
                if i.len() > 0 {
                    bail!("Could not parse all of the input, {} bytes left", i.len());
                }
                Ok(notes)
            }
            Err(e) => {
                bail!("Could not parse notes: {}", e);
            }
        }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, constraints) = separated_list1(line_ending, FieldConstraint::parse)(i)?;
        let (i, _) = many1(line_ending)(i)?;
        let (i, my_ticket) = preceded(tuple((tag("your ticket:"), line_ending)), Ticket::parse)(i)?;
        let (i, _) = tuple((
            line_ending,
            line_ending,
            tag("nearby tickets:"),
            line_ending,
        ))(i)?;
        let (i, tickets) = separated_list1(line_ending, Ticket::parse)(i)?;
        let (i, _) = many1(line_ending)(i)?;

        Ok((
            i,
            Self {
                constraints,
                my_ticket,
                tickets,
            },
        ))
    }

    fn fits(&self, value: usize) -> bool {
        for constr in self.constraints.iter() {
            if constr.fits(value) {
                return true;
            }
        }
        false
    }

    fn infer_fields(&self) -> Result<Vec<FieldConstraint>> {
        let mut possible_fields: Vec<HashSet<FieldConstraint>> =
            std::iter::repeat(self.constraints.clone().into_iter().collect())
                .take(self.my_ticket.len())
                .collect();

        for ticket in self.valid_tickets().iter() {
            for (idx, value) in ticket.fields.iter().enumerate() {
                possible_fields[idx] = possible_fields[idx]
                    .clone()
                    .into_iter()
                    .filter(|c| c.fits(*value))
                    .collect();
            }
        }

        for (idx, left) in possible_fields.iter().enumerate() {
            eprintln!("Num possible fields left at {}: {}", idx, left.len());
        }

        let mut done: HashMap<usize, FieldConstraint> = HashMap::new();

        while done.len() < self.my_ticket.len() {
            let (idx, constraint) = possible_fields
                .iter()
                .enumerate()
                .filter(|(_, c)| c.len() == 1)
                .next()
                .map(|(idx, v)| (idx, v.iter().next().unwrap().clone()))
                .unwrap();
            for pf in possible_fields.iter_mut() {
                pf.remove(&constraint);
            }
            done.insert(idx, constraint);
        }

        for (idx, left) in possible_fields.iter().enumerate() {
            if left.len() != 0 {
                bail!("Still {} choices for field at pos {}", left.len(), idx);
            }
        }

        Ok((0..self.my_ticket.len())
            .map(|idx| done.get(&idx).unwrap().clone())
            .collect())
    }

    fn ticket_scanning_error_rate(&self) -> usize {
        let mut rate = 0;
        for ticket in self.tickets.iter() {
            for value in ticket.fields.iter() {
                if !self.fits(*value) {
                    rate += value;
                }
            }
        }
        rate
    }

    fn valid_tickets(&self) -> Vec<Ticket> {
        self.tickets
            .iter()
            .filter(|t| -> bool {
                for value in t.fields.iter() {
                    if !self.fits(*value) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }
}
