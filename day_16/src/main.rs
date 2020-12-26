use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, none_of, one_of},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::collections::HashMap;
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
    println!("Notes: {:#?}", notes);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
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
}
