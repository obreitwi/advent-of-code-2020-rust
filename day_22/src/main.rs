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
use std::collections::{HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    part1(&input)?;
    part2(&input)?;
    Ok(())
}

fn part1(input: &Path) -> Result<()> {
    let mut game = Game::read_from(input)?;

    let score = game.play();
    println!("Final score: {}", score);

    Ok(())
}

fn part2(input: &Path) -> Result<()> {
    let mut game = Game::read_from(input)?;

    game.play_recursive();
    println!("Final score: {}", game.score());

    Ok(())
}

type Card = usize;
type Score = usize;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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

    pub fn enough_for_sub_game(&self, card: Card) -> bool {
        self.num_cards() >= card
    }

    pub fn get_sub_deck(&self, num_cards: usize) -> Self {
        Self {
            player_id: self.player_id,
            stack: self.stack.iter().take(num_cards).cloned().collect(),
        }
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

enum Winner {
    Player1,
    Player2,
}

#[derive(Debug)]
struct Game {
    num: usize,
    previous_rounds: HashSet<(Deck, Deck)>,
    player_1: Deck,
    player_2: Deck,
}

impl Game {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (player_1, player_2)) =
            separated_pair(Deck::parse, many1(line_ending), Deck::parse)(i)?;
        assert_eq!(player_1.player_id, 1, "Invalid player id");
        assert_eq!(player_2.player_id, 2, "Invalid player id");
        Ok((
            i,
            Self {
                num: 1,
                previous_rounds: HashSet::new(),
                player_1,
                player_2,
            },
        ))
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
                } else {
                    // println!("Player 2 wins round.");
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

    pub fn play_recursive(&mut self) -> Winner {
        let mut round = 0;
        loop {
            round += 1;
            // println!();
            // println!("Game #{} (Round #{}):", self.num, round);
            // println!("{}", self.player_1);
            // println!("{}", self.player_2);

            // eprintln!("Game #{} Round #{} Number of rounds recorded: {}", self.num, round, self.previous_rounds.len());
            if self.was_round_previously_played() {
                // println!("Round prevously played -> player 1 wins.");
                return Winner::Player1;
            } else {
                self.record_round();
            }

            if let (Some(card_1), Some(card_2)) = (self.player_1.draw(), self.player_2.draw()) {
                // println!("Player 1 plays: {}", card_1);
                // println!("Player 2 plays: {}", card_2);

                self.resolve_round(self.determine_winner(card_1, card_2), card_1, card_2);
            } else {
                panic!("Player unexpectetly ran out of cards.");
            }

            if self.is_over() {
                break;
            }
        }
        if self.player_1.out_of_cards() {
            Winner::Player2
        } else {
            Winner::Player1
        }
    }

    fn determine_winner(&self, card_1: Card, card_2: Card) -> Winner {
        if self.player_1.enough_for_sub_game(card_1) && self.player_2.enough_for_sub_game(card_2) {
            let mut sub_game = self.sub_game((card_1, card_2));
            sub_game.play_recursive()
        } else {
            self.determine_winner_regular(card_1, card_2)
        }
    }

    fn determine_winner_regular(&self, card_1: Card, card_2: Card) -> Winner {
        if card_1 > card_2 {
            Winner::Player1
        } else {
            Winner::Player2
        }
    }

    pub fn score(&self) -> Score {
        self.player_1.calc_score() + self.player_2.calc_score()
    }

    fn resolve_round(&mut self, winner: Winner, card_1: Card, card_2: Card) {
        match winner {
            Winner::Player1 => {
                // println!("Player 1 wins round.");
                self.player_1.add_card(card_1);
                self.player_1.add_card(card_2);
            }
            Winner::Player2 => {
                // println!("Player 2 wins round.");
                self.player_2.add_card(card_2);
                self.player_2.add_card(card_1);
            }
        };
    }

    fn record_round(&mut self) {
        self.previous_rounds
            .insert((self.player_1.clone(), self.player_2.clone()));
    }

    fn was_round_previously_played(&self) -> bool {
        self.previous_rounds
            .contains(&(self.player_1.clone(), self.player_2.clone()))
    }

    fn sub_game(&self, num_cards: (usize, usize)) -> Self {
        Self {
            previous_rounds: HashSet::new(),
            player_1: self.player_1.get_sub_deck(num_cards.0),
            player_2: self.player_2.get_sub_deck(num_cards.1),
            num: self.num + 1,
        }
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

    #[test]
    fn play_game_recursive() -> Result<()> {
        let mut game = Game::read_from(&PathBuf::from("debug.txt"))?;
        game.play_recursive();
        assert_eq!(game.score(), 291, "Debug game does not have correct score.");
        Ok(())
    }
}
