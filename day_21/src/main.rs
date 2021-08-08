#![allow(unused_imports)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space1},
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
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);

    part1(&input)?;

    Ok(())
}

fn part1(input: &Path) -> Result<usize> {
    let dishes = Dishes::read(input)?;
    eprintln!("{:#?}", dishes);

    let ingredient_to_allergens = IngredientToAllergens::new(&dishes);
    let ingredients_without_allergens = ingredient_to_allergens.ingredients_without_allergens();

    let mut count = 0;
    for dish in dishes.data.iter() {
        count += dish
            .ingredients
            .intersection(&ingredients_without_allergens)
            .count();
        println!("Count: {}", count);
    }
    println!(
        "Part1: Number of ingredients without allergens: {:#?}",
        ingredients_without_allergens
    );
    println!("Part1: Number of appearances: {}", count);

    Ok(count)
}

type Allergen = String;
type Ingredient = String;

#[derive(Debug, Clone)]
struct Dish {
    allergens: HashSet<Allergen>,
    ingredients: HashSet<Ingredient>,
}

impl Dish {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, ingredients) = separated_list1(space1, alpha1)(i)?;
        let (i, _) = tag(" (contains ")(i)?;
        let (i, allergens) = separated_list1(tag(", "), alpha1)(i)?;
        let (i, _) = char(')')(i)?;
        Ok((
            i,
            Self {
                ingredients: ingredients.iter().map(|s| s.to_string()).collect(),
                allergens: allergens.iter().map(|s| s.to_string()).collect(),
            },
        ))
    }
}

#[derive(Debug, Clone)]
struct Dishes {
    data: Vec<Dish>,
}

impl Dishes {
    fn read(input: &Path) -> Result<Dishes> {
        let input = read_to_string(&input)?;
        if let Ok((i, dishes)) = Self::parse(&input) {
            assert_eq!(i, "\n", "Could not parse all dishes.");
            Ok(dishes)
        } else {
            bail!("Could not read dishes.");
        }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, dishes) = separated_list1(line_ending, Dish::parse)(i)?;
        Ok((i, Self { data: dishes }))
    }

    fn allergens(&self) -> HashSet<Allergen> {
        self.data
            .iter()
            .flat_map(|d| d.allergens.iter().cloned())
            .collect()
    }

    fn ingredients(&self) -> HashSet<Allergen> {
        self.data
            .iter()
            .flat_map(|d| d.ingredients.iter().cloned())
            .collect()
    }
}

struct IngredientToAllergens {
    data: HashMap<Ingredient, HashSet<Allergen>>,
}

impl IngredientToAllergens {
    fn new(dishes: &Dishes) -> Self {
        let mut data = Self::raw(dishes);

        let ingredients = dishes.ingredients();

        for dish in dishes.data.iter() {
            for ingredient in ingredients.difference(&dish.ingredients) {
                // all ingredients not present in the current recipe cannot account for allergens
                // present in the recipe
                data.entry(ingredient.clone()).and_modify(|a| {
                    *a = a.difference(&dish.allergens).cloned().collect();
                });
            }
        }

        Self { data }
    }

    fn ingredients_without_allergens(&self) -> HashSet<Ingredient> {
        self.data
            .iter()
            .filter_map(|(k, v)| if v.is_empty() { Some(k) } else { None })
            .cloned()
            .collect()
    }

    fn raw(dishes: &Dishes) -> HashMap<Ingredient, HashSet<Allergen>> {
        let allergens = dishes.allergens();
        dishes
            .ingredients()
            .into_iter()
            .map(|i| (i, allergens.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug() -> Result<()> {
        assert_eq!(
            part1(&PathBuf::from("debug.txt"))?,
            5,
            "Invalid occurences in test data."
        );
        Ok(())
    }
}
