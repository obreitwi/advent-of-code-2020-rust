#![allow(unused_imports)]
#![feature(iter_intersperse)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space0},
    combinator::{map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    part1(&input)?;
    Ok(())
}

fn part1(input: &Path) -> Result<usize> {
    let mut game = Game::read_from(input)?;

    let score = game.play();
    println!("Final score: {}", score);

    Ok(score)
}

type Card = usize;
type Score = usize;

#[derive(Debug)]
struct Deck {
    player_id: usize,
    stack: VecDeque<Card>,
}

impl Deck {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (player_id, stack)) = pair(Self::parse_player_id, Self::parse_stack)(i)?;

        Ok((
            i,
            Self {
                player_id,
                stack: stack.into_iter().collect(),
            },
        ))
    }

    fn parse_player_id(i: &str) -> IResult<&str, usize> {
        delimited(
            tag("Player "),
            map_res(digit1, |s: &str| s.parse::<usize>()),
            pair(tag(":"), line_ending),
        )(i)
    }

    fn parse_stack(i: &str) -> IResult<&str, Vec<Card>> {
        let parse_digit = map_res(digit1, |s: &str| s.parse::<Card>());
        separated_list1(line_ending, parse_digit)(i)
    }

    pub fn out_of_cards(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.stack.pop_front()
    }

    pub fn add_card(&mut self, card: Card) {
        self.stack.push_back(card);
    }

    pub fn num_cards(&self) -> usize {
        self.stack.len()
    }

    pub fn calc_score(&self) -> Score {
        let num_cards = self.num_cards();
        let mut score = 0;

        for (i, card) in self.stack.iter().enumerate() {
            score += (num_cards - i) * card;
        }
        score
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Player {}'s deck: {}",
            self.player_id,
            self.stack
                .iter()
                .map(|d| format!("{}", d))
                .intersperse(", ".to_string())
                .collect::<String>()
        )
    }
}

#[derive(Debug)]
struct Game {
    player_1: Deck,
    player_2: Deck,
}

impl Game {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (player_1, player_2)) =
            separated_pair(Deck::parse, many1(line_ending), Deck::parse)(i)?;
        assert_eq!(player_1.player_id, 1, "Invalid player id");
        assert_eq!(player_2.player_id, 2, "Invalid player id");
        Ok((i, Self { player_1, player_2 }))
    }

    fn read_from(i: &Path) -> Result<Self> {
        let input = read_to_string(i)?;
        match Self::parse(&input) {
            Ok((i, game)) => {
                assert_eq!(i, "\n", "Did not parse all input.");
                Ok(game)
            }
            Err(e) => {
                bail!("Could not parse game: {}", e);
            }
        }
    }

    pub fn play(&mut self) -> Score {
        let mut round = 0;
        loop {
            round += 1;
            println!("Round #{}:", round);

            println!("{}", self.player_1);
            println!("{}", self.player_2);

            if let (Some(card1), Some(card2)) = (self.player_1.draw(), self.player_2.draw()) {
                println!("Player 1 plays: {}", card1);
                println!("Player 2 plays: {}", card2);
                if card1 > card2 {
                    println!("Player 1 wins round.");
                    self.player_1.add_card(card1);
                    self.player_1.add_card(card2);
                }
                else
                {
                    println!("Player 2 wins round.");
                    self.player_2.add_card(card2);
                    self.player_2.add_card(card1);
                }
            } else {
                panic!("Player unexpectetly ran out of cards.");
            }
            println!();

            if self.is_over() {
                break;
            }
        }
        self.player_1.calc_score() + self.player_2.calc_score()
    }

    fn is_over(&self) -> bool {
        self.player_1.out_of_cards() || self.player_2.out_of_cards()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_game() -> Result<()> {
        let game = Game::read_from(&PathBuf::from("debug.txt"))?;
        eprintln!("Game:\n{:#?}", game);
        Ok(())
    }

    #[test]
    fn play_game() -> Result<()> {
        let mut game = Game::read_from(&PathBuf::from("debug.txt"))?;
        assert_eq!(game.play(), 306, "Debug game does not have correct score.");
        Ok(())
    }
}
