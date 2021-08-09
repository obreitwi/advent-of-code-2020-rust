#![allow(unused_imports)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, digit1, line_ending, none_of, one_of, space0},
    combinator::{map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

fn main() -> Result<()> {
    // let input = env::args().nth(1).with_context(|| "No input provided!")?;
    let input = "583976241".to_string();
    part1(&input)?;
    // part2(&input)?;
    Ok(())
}

fn part1(i: &str) -> Result<()> {
    let mut cups = CrabCups::from(i);

    println!("{}", cups);

    Ok(())
}

type Label = char;

#[derive(Debug)]
struct Cup {
    label: Label,
    left: WeakCup,
    right: WeakCup,
}

type RcCup = Rc<RefCell<Cup>>;
type WeakCup = Weak<RefCell<Cup>>;

#[derive(Debug)]
struct CrabCups {
    current: WeakCup,
    cups: HashMap<Label, RcCup>,
}

impl CrabCups {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, labels) = digit1(i)?;

        let mut cups = HashMap::new();

        let mut first = Weak::new();
        let mut prev = Weak::new();

        for l in labels.chars() {
            let current = Rc::new(RefCell::new(Cup {
                label: l,
                left: prev.clone(),
                right: Weak::new(),
            }));
            if let Some(prev) = prev.upgrade() {
                prev.borrow_mut().right = Rc::downgrade(&current);
            };
            if first.upgrade().is_none() {
                first = Rc::downgrade(&current);
            }
            prev = Rc::downgrade(&current);
            cups.insert(l, current);
        }
        // close the loop
        let last = prev;
        if let Some(c) = first.upgrade() {
            c.borrow_mut().left = last.clone();
        };
        if let Some(c) = last.upgrade() {
            c.borrow_mut().right = first.clone();
        };

        Ok((
            i,
            Self {
                cups,
                current: first,
            },
        ))
    }

    pub fn make_move(&mut self) {
        todo!();
    }

    fn pick_up_cups(&mut self) -> RcCup
    {
        todo!();
    }

    fn select_destination(&mut self) -> RcCup {
        todo!();
    }

    fn select_current(&mut self) -> RcCup {
        todo!();
    }
}

impl fmt::Display for CrabCups {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = self.current.upgrade().unwrap();
        let label_done = current.borrow().label;
        write!(f, "({})", label_done)?;
        loop {
            current = {
                let next = current.borrow().right.clone();
                next.upgrade().unwrap()
            };
            let cup = current.borrow();
            if cup.label == label_done {
                break;
            }
            write!(f, " {}", cup.label)?;
        }
        Ok(())
    }
}

impl From<&str> for CrabCups {
    fn from(i: &str) -> Self {
        match CrabCups::parse(i).finish() {
            Ok((i, cc)) => {
                assert!(i.is_empty(), "Did not consume full string.");
                cc
            }
            Err(e) => {
                panic!("Error parsing CrabCups: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let raw = "123456789".to_string();
        let cups = CrabCups::from(raw.as_str());

        for label in raw.chars() {
            let current = cups.cups.get(&label).unwrap();
            eprintln!(
                "current: {} left: {:#?} right: {:#?}",
                current.borrow().label,
                current
                    .borrow()
                    .left
                    .upgrade()
                    .map(|c| c.borrow().label)
                    .unwrap_or('X'),
                current
                    .borrow()
                    .right
                    .upgrade()
                    .map(|c| c.borrow().label)
                    .unwrap_or('X'),
            );
        }

        for (label, cup) in cups.cups.iter() {
            assert_eq!(*label, cup.borrow().label, "Labels don't match.");
        }
        assert_eq!(format!("{}", cups), "(1) 2 3 4 5 6 7 8 9".to_string());
    }
}
