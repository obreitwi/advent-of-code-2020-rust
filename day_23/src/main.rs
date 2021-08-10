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
use std::cmp::{max, min};
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

    for _ in 0..100 {
        cups.make_move();
    }
    println!(
        "Labels after 1: {}",
        cups.labels_from(1)?
            .into_iter()
            .skip(1)
            .map(|i| format!("{}", i))
            .collect::<String>()
    );

    Ok(())
}

type Label = usize;

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
    min_label: Label,
    max_label: Label,
    current: WeakCup,
    cups: HashMap<Label, RcCup>,
}

impl CrabCups {
    pub fn new(label: Label) -> Self {
        let mut cups = HashMap::new();
        let init = {
            let init = Rc::new(RefCell::new(Cup {
                label,
                left: Weak::new(),
                right: Weak::new(),
            }));
            init.borrow_mut().left = Rc::downgrade(&init);
            init.borrow_mut().right = Rc::downgrade(&init);
            init
        };
        let current = Rc::downgrade(&init);
        cups.insert(label, init);
        Self {
            cups,
            min_label: Label::MAX,
            max_label: Label::MIN,
            current,
        }
    }

    fn add_left_from_current(&mut self, label: Label) {
        let current = self.current.upgrade().unwrap();
        let last = Self::left(&current);

        self.min_label = min(label, self.min_label);
        self.max_label = max(label, self.max_label);

        let new_cup = Rc::new(RefCell::new(Cup {
            label,
            left: Weak::new(),
            right: Weak::new(),
        }));
        self.cups.insert(label, new_cup.clone());

        Self::close_gap(&last, &new_cup);
        Self::close_gap(&new_cup, &current);
    }

    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, labels) = digit1(i)?;

        let mut retval: Option<Self> = None;

        for l in labels.chars() {
            let l = String::from(l).parse::<Label>().unwrap();

            retval = match retval {
                None => Some(Self::new(l)),
                Some(mut cc) => {
                    cc.add_left_from_current(l);
                    Some(cc)
                }
            }
        }
        Ok((i, retval.unwrap()))
    }

    pub fn make_move(&mut self) {
        let taken = self.pick_up_cups();

        let destination = self.select_destination(taken.clone());
        let (taken_left, taken_right) = taken;

        let destination_right = Self::right(&destination);
        Self::close_gap(&destination, &taken_left);
        Self::close_gap(&taken_right, &destination_right);
        self.current = Rc::downgrade(&self.select_current());
    }

    fn select_destination(&self, taken: (RcCup, RcCup)) -> RcCup {
        let taken = Self::labels(taken);
        self.cups
            .get(&self.label_destination(&taken[..]))
            .cloned()
            .unwrap()
    }

    fn label_destination(&self, taken: &[Label]) -> Label {
        let next_label = |label| {
            if label == self.min_label {
                self.max_label
            } else {
                label - 1
            }
        };
        let mut proposed = next_label(self.current.upgrade().unwrap().borrow().label);
        while taken.contains(&proposed) {
            proposed = next_label(proposed);
        }
        proposed
    }

    fn pick_up_cups(&mut self) -> (RcCup, RcCup) {
        self.remove_cups(3)
    }

    fn remove_cups(&mut self, num: usize) -> (RcCup, RcCup) {
        assert!(num > 0);
        let removed_first = Self::nth_right(self.current.clone(), 1).upgrade().unwrap();
        let removed_last = Self::nth_right(self.current.clone(), num)
            .upgrade()
            .unwrap();

        let remaining_gap_left = removed_first.borrow().left.upgrade().unwrap();
        let remaining_gap_right = removed_last.borrow().right.upgrade().unwrap();

        Self::close_gap(&removed_last, &removed_first);
        Self::close_gap(&remaining_gap_left, &remaining_gap_right);

        (removed_first, removed_last)
    }

    fn labels((left, right): (RcCup, RcCup)) -> Vec<Label> {
        let label_right = &right.borrow().label;
        let mut current = left;
        let mut labels = Vec::new();
        loop {
            let current_label = current.borrow().label;
            labels.push(current_label);
            if current_label == *label_right {
                break;
            } else {
                let right = current.borrow().right.clone();
                current = right.upgrade().unwrap();
            }
        }
        labels
    }

    fn close_gap(left: &RcCup, right: &RcCup) {
        right.borrow_mut().left = Rc::downgrade(left);
        left.borrow_mut().right = Rc::downgrade(right);
    }

    fn nth_right(mut current: WeakCup, num: usize) -> WeakCup {
        assert!(num > 0);
        for _ in 0..num {
            let cup = current.upgrade().unwrap();
            current = cup.borrow().right.clone();
        }
        current
    }

    fn nth_left(mut current: WeakCup, num: usize) -> WeakCup {
        assert!(num > 0);
        for _ in 0..num {
            let cup = current.upgrade().unwrap();
            current = cup.borrow().left.clone();
        }
        current
    }

    fn right(current: &RcCup) -> RcCup {
        current.borrow().right.clone().upgrade().unwrap()
    }

    fn left(current: &RcCup) -> RcCup {
        current.borrow().left.clone().upgrade().unwrap()
    }

    fn select_current(&mut self) -> RcCup {
        Self::right(&self.current.upgrade().unwrap())
    }

    pub fn labels_from(&self, label: Label) -> Result<Vec<Label>> {
        let mut current = self
            .cups
            .get(&label)
            .with_context(|| "Invalid label specified.j")?
            .clone();
        let mut labels = Vec::with_capacity(self.cups.len());
        for _ in 0..self.cups.len() {
            labels.push(current.borrow().label);
            current = Self::right(&current);
        }
        Ok(labels)
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
    fn test_parsing() -> Result<()> {
        let raw = "123456789".to_string();
        let cups = CrabCups::from(raw.as_str());

        for label in raw.chars() {
            let label = String::from(label).parse::<Label>().unwrap();
            let current = cups.cups.get(&label).unwrap();
            eprintln!(
                "current: {} left: {:#?} right: {:#?}",
                current.borrow().label,
                current
                    .borrow()
                    .left
                    .upgrade()
                    .map(|c| c.borrow().label)
                    .unwrap_or(Label::MAX),
                current
                    .borrow()
                    .right
                    .upgrade()
                    .map(|c| c.borrow().label)
                    .unwrap_or(Label::MAX),
            );
        }

        assert_eq!(cups.labels_from(1)?, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        eprintln!("{:#?}", cups);
        for (label, cup) in cups.cups.iter() {
            assert_eq!(*label, cup.borrow().label, "Labels don't match.");
        }
        assert_eq!(format!("{}", cups), "(1) 2 3 4 5 6 7 8 9".to_string());
        Ok(())
    }

    #[test]
    fn debug_moves() -> Result<()> {
        let mut cups = CrabCups::from("389125467");
        for _ in 0..100 {
            cups.make_move();
        }
        assert_eq!(cups.labels_from(1)?, vec![1, 6, 7, 3, 8, 4, 5, 2, 9]);
        Ok(())
    }
}
