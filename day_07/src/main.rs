use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, space1},
    combinator::opt,
    multi::many1,
    sequence::terminated,
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::str;

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );
    let rules = RuleSet::read_from(&input)?;
    // rules.print();
    part1(&rules);
    part2(&rules);

    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Color {
    name: String,
}

impl From<&str> for Color {
    fn from(i: &str) -> Self {
        Self { name: i.into() }
    }
}

impl Color {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, first) = alpha1(i)?;
        let (i, _) = space1(i)?;
        let (i, second) = alpha1(i)?;
        Ok((
            i,
            Self {
                name: format!("{} {}", first, second),
            },
        ))
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Bags {
    color: Color,
    count: usize,
}

impl Bags {
    pub fn parse(i: &str) -> IResult<&str, Vec<Self>> {
        alt((
            Self::parse_none,
            many1(terminated(Self::parse_some, alt((tag(", "), tag("."))))),
        ))(i)
    }

    pub fn parse_some(i: &str) -> IResult<&str, Self> {
        let (i, num) = digit1(i)?;
        let count: usize = num.parse().unwrap();
        let (i, _) = space1(i)?;
        let (i, color) = Color::parse(i)?;
        let (i, _) = space1(i)?;
        let (i, _) = tag("bag")(i)?;
        let (i, _) = opt(tag("s"))(i)?;
        Ok((i, Self { color, count }))
    }

    pub fn parse_none(i: &str) -> IResult<&str, Vec<Self>> {
        let (i, _) = tag("no other bags.")(i)?;
        Ok((i, Vec::new()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BagRule {
    container: Color,
    contents: HashMap<Color, Bags>,
}

impl BagRule {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, container) = Color::parse(i)?;
        let (i, _) = tag(" bags contain ")(i)?;
        let (i, contents) = Bags::parse(i)?;

        let contents: HashMap<_, _> = contents.into_iter().map(|b| (b.color.clone(), b)).collect();

        Ok((
            i,
            Self {
                container,
                contents,
            },
        ))
    }

    pub fn contains(&self, color: &Color) -> bool {
        self.contents.contains_key(color)
    }
}

struct RuleSet {
    rules: HashMap<Color, BagRule>,
}

impl RuleSet {
    pub fn read_from(path: &Path) -> Result<Self> {
        let mut rules = HashMap::new();
        for line in io::BufReader::new(File::open(&path)?).lines() {
            let line = line?;
            let parsed = BagRule::parse(&line).finish();
            match parsed {
                Ok((_, br)) => {
                    rules.insert(br.container.clone(), br);
                }
                Err(e) => bail!("Error parsing {}: {}", line, e),
            }
        }
        Ok(Self { rules })
    }

    /// Return a HashSet of bag colors the given color is in (recursively).
    pub fn contain(&self, color: &Color) -> HashSet<Color> {
        let mut found = HashSet::new();
        let mut to_check = vec![color.clone()];

        while to_check.len() > 0 {
            let current = to_check.pop().unwrap();
            let containers = self.is_in(&current);
            for color in containers {
                if !found.contains(&color) {
                    found.insert(color.clone());
                    to_check.push(color);
                }
            }
        }
        found
    }

    pub fn count(&self, color: &Color) -> usize {
        let contents = &self.rules.get(color).unwrap().contents;
        contents
            .values()
            .map(|bs| bs.count * (1 + self.count(&bs.color)))
            .sum()
    }

    /// Return a set of bag colors the given color is in (non-recursively).
    fn is_in(&self, color: &Color) -> HashSet<Color> {
        self.rules
            .values()
            .filter(|r| r.contains(color))
            .map(|r| r.container.clone())
            .collect()
    }

    pub fn print(&self) {
        for r in self.rules.iter() {
            println!("{:#?}", r);
        }
    }
}

fn part1(rules: &RuleSet) {
    let golden = Color::from("shiny gold");
    let containers = rules.contain(&golden);
    println!("{} is in {} bags.", golden, containers.len());
}

fn part2(rules: &RuleSet) {
    let golden = Color::from("shiny gold");
    let count = rules.count(&golden);
    println!("{} contains {} bags.", golden, count);
}
