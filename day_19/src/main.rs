use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space0},
    combinator::{map_res, value},
    multi::{many0, many1, separated_list1},
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

    let ruleset = RuleSet::read_from(&input)?;
    println!("ruleset: {:#?}", ruleset);

    part1(&ruleset)?;
    part2(ruleset)?;

    Ok(())
}

fn part1(ruleset: &RuleSet) -> Result<()> {
    let matching = ruleset.get_matching()?;
    println!("================================================================================");
    println!("(part1) num matching entries: {}", matching.len());
    println!("================================================================================");
    Ok(())
}

fn part2(mut ruleset: RuleSet) -> Result<()> {
    ruleset.rules.insert(8, Rule::Multi(42));
    ruleset.rules.insert(11, Rule::SameN(42, 31));
    println!("{:#?}", ruleset);
    let matching = ruleset.get_matching()?;
    println!("================================================================================");
    println!("(part2) num matching entries: {}", matching.len());
    println!("================================================================================");
    Ok(())
}

#[derive(Debug, Clone)]
enum Rule {
    Explicit(char),
    Alt(Vec<Vec<usize>>),
    Multi(usize),
    SameN(usize, usize),
}

impl Rule {
    fn parse(i: &str) -> IResult<&str, (usize, Rule)> {
        let (i, idx) = terminated(digit1, tuple((char(':'), space0)))(i)?;
        let idx = idx.parse().unwrap();

        let (i, rule) = alt((Rule::parse_alt, Rule::parse_explicit))(i)?;
        Ok((i, (idx, rule)))
    }

    fn parse_alt(i: &str) -> IResult<&str, Rule> {
        let (i, alt) = separated_list1(
            tag(" | "),
            separated_list1(tag(" "), map_res(digit1, |s: &str| s.parse::<usize>())),
        )(i)?;
        Ok((i, Rule::Alt(alt)))
    }

    fn parse_explicit(i: &str) -> IResult<&str, Rule> {
        let (i, s) = delimited(char('"'), anychar, char('"'))(i)?;
        Ok((i, Rule::Explicit(s.into())))
    }
}

#[derive(Debug, Clone)]
struct RuleSet {
    rules: HashMap<usize, Rule>,
    strings: Vec<String>,
}

impl RuleSet {
    fn parse(i: &str) -> IResult<&str, RuleSet> {
        let (i, vec) = separated_list1(line_ending, Rule::parse)(i)?;
        let rules = vec.into_iter().collect();

        let (i, _) = many1(line_ending)(i)?;

        let (i, strings) = separated_list1(
            line_ending,
            map_res::<_, _, _, _, nom::error::Error<&str>, _, _>(alpha1, |l: &str| Ok(l.into())),
        )(i)?;

        let (i, _) = many1(line_ending)(i)?;

        Ok((i, RuleSet { rules, strings }))
    }

    fn read_from(input: &Path) -> Result<Self> {
        let input = read_to_string(&input)?;
        let rules = {
            match Self::parse(&input).finish() {
                Ok((i, rules)) => {
                    if i.len() > 0 {
                        bail!("Num bytes not consumed: {}", i.len());
                    } else {
                        rules
                    }
                }
                Err(e) => bail!("Error: {}", e),
            }
        };
        Ok(rules)
    }

    fn match_rule<I: Iterator<Item = char> + Clone>(
        &self,
        idx: usize,
        mut i: I,
    ) -> Result<Option<I>> {
        let rule = self
            .rules
            .get(&idx)
            .with_context(|| format!("Invalid rule index: {}", idx))?;

        eprintln!(
            "Trying rule #{} while matching {}",
            idx,
            i.clone().collect::<String>()
        );

        match rule {
            Rule::Explicit(literal) => match i.next() {
                Some(c) => {
                    if *literal == c {
                        return Ok(Some(i));
                    } else {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            },
            Rule::Alt(vec) => {
                'a: for rls in vec.iter() {
                    let mut i = i.clone();
                    for idx in rls.iter() {
                        match self.match_rule(*idx, i)? {
                            Some(i_matched) => {
                                i = i_matched;
                            }
                            None => {
                                continue 'a;
                            }
                        }
                    }
                    return Ok(Some(i.clone()));
                }
                return Ok(None);
            }
            Rule::Multi(idx) => {
                // longest matching only
                let mut times_matched = 0;
                let mut i = i.clone();
                loop {
                    match self.match_rule(*idx, i.clone())? {
                        Some(i_new) => {
                            i = i_new;
                            times_matched += 1;
                        }
                        None => {
                            break;
                        }
                    }
                }
                if times_matched > 0 {
                    eprintln!("Matched multi-rule #{} {} times", idx, times_matched);
                    Ok(Some(i))
                } else {
                    Ok(None)
                }
            }
            Rule::SameN(idx_fst, idx_snd) => {
                // longest matching only for now
                let mut i = i.clone();
                let mut times_matched_fst = 0;
                loop {
                    match self.match_rule(*idx_fst, i.clone())? {
                        Some(i_new) => {
                            i = i_new;
                            times_matched_fst += 1;
                        }
                        None => {
                            break;
                        }
                    }
                }
                let times_matched_fst = times_matched_fst;

                if times_matched_fst == 0 {
                    Ok(None)
                } else {
                    let mut times_matched_snd = 0;
                    for _ in 0..times_matched_fst {
                        match self.match_rule(*idx_snd, i.clone())? {
                            Some(i_new) => {
                                i = i_new;
                                times_matched_snd += 1;
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    eprintln!("Matched first same-rule #{} {} times. Matched second same-rule #{} {} times.", idx_fst, times_matched_fst, idx_snd, times_matched_snd);

                    if times_matched_fst == times_matched_snd {
                        Ok(Some(i))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }

    fn get_matching(&self) -> Result<Vec<String>> {
        let mut rv = Vec::new();
        for e in self.strings.iter() {
            if let Some(mut i) = self.match_rule(0, e.chars())? {
                if let None = i.next() {
                    rv.push(e.to_string());
                }
            }
        }
        Ok(rv)
    }
}
