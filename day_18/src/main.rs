use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    character::complete::{char, digit0, line_ending, space0},
    combinator::value,
    multi::separated_list1,
    sequence::{delimited, terminated, tuple},
    Finish, IResult,
};
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

    let exprs = Expression::read_from(&input)?;
    part1(&exprs[..]);

    Ok(())
}

fn part1(exprs: &[Expression]) {
    let result: i64 = exprs.iter().map(|e| e.eval()).sum();
    println!("(part1) Result: {}", result);
}

#[derive(Debug, Clone)]
struct Expression {
    initial: Operand,
    ops: Vec<(Operator, Operand)>,
}

impl Expression {
    fn read_from(input: &Path) -> Result<Vec<Self>> {
        let input = read_to_string(&input)?;
        let retval = match terminated(Self::parse_vec, line_ending)(&input) {
            Ok((rem, exprs)) => {
                if rem.len() > 0 {
                    bail!(
                        "Failed afer parsing {} entries.\nParsed: {:?}\nRemainder: {}",
                        exprs.len(),
                        exprs,
                        rem
                    );
                }
                Ok(exprs)
            }
            Err(e) => bail!("Error during parsing: {}", e),
        };
        retval
    }

    fn parse_vec(i: &str) -> IResult<&str, Vec<Expression>> {
        separated_list1(line_ending, Self::parse)(i)
    }

    fn parse(i: &str) -> IResult<&str, Expression> {
        let (i, initial) = Operand::parse(i)?;
        let (mut i, _) = space0(i)?;
        let mut ops = Vec::new();
        loop {
            match tuple((Operator::parse, space0, Operand::parse, space0))(i) {
                Ok((i_, (op, _, opnd, _))) => {
                    i = i_;
                    ops.push((op, opnd));
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok((i, Self { initial, ops }))
    }

    fn parse_full(i: &str) -> Result<Expression> {
        match Self::parse(i).finish() {
            Ok((i, expr)) => {
                if i.len() > 0 {
                    bail!("Failed to parse full expression, leftover: {}", i);
                }
                Ok(expr)
            }
            Err(e) => bail!("Failed to parse expression: {}", e),
        }
    }

    fn eval(&self) -> i64 {
        let mut result = self.initial.clone();

        for (op, operand) in self.ops.iter() {
            result = op.apply(&result, operand);
        }

        result.eval()
    }
}

#[derive(Debug, Clone)]
enum Operand {
    Value(i64),
    Expr(Box<Expression>),
}

impl Operand {
    fn eval(&self) -> i64 {
        match self {
            Operand::Value(value) => *value,
            Operand::Expr(expr) => expr.eval(),
        }
    }

    fn parse(i: &str) -> IResult<&str, Operand> {
        use Operand::*;
        let (i, num) = digit0(i)?;
        if num.len() > 0 {
            return Ok((i, Value(num.parse().unwrap())));
        }
        let (i, expr) = delimited(char('('), Expression::parse, char(')'))(i)?;
        Ok((i, Expr(Box::new(expr))))
    }
}

#[derive(Debug, Clone)]
enum Operator {
    Add,
    Mult,
}

impl Operator {
    fn apply(&self, left: &Operand, right: &Operand) -> Operand {
        let left_value = left.eval();
        let right_value = right.eval();
        match self {
            Operator::Add => Operand::Value(left_value + right_value),
            Operator::Mult => Operand::Value(left_value * right_value),
        }
    }

    fn parse(i: &str) -> IResult<&str, Operator> {
        use Operator::*;
        alt((value(Add, char('+')), value(Mult, char('*'))))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() -> Result<()> {
        eprintln!("{:?}", Expression::parse_full("2 * 3 + (4 * 5)")?);
        eprintln!(
            "{:?}",
            Expression::parse_full("5 + (8 * 3 + 9 + 3 * 4 * 3)")?
        );
        eprintln!(
            "{:?}",
            Expression::parse_full("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))")?
        );
        eprintln!(
            "{:?}",
            Expression::parse_full("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2")?
        );
        Ok(())
    }

    #[test]
    fn test_eval() -> Result<()> {
        assert_eq!(26, Expression::parse_full("2 * 3 + (4 * 5)")?.eval());
        assert_eq!(
            437,
            Expression::parse_full("5 + (8 * 3 + 9 + 3 * 4 * 3)")?.eval()
        );
        assert_eq!(
            12240,
            Expression::parse_full("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))")?.eval()
        );
        assert_eq!(
            13632,
            Expression::parse_full("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2")?.eval()
        );
        Ok(())
    }

    #[test]
    fn special_parsing() -> Result<()> {
        eprintln!(
            "Special Expression:\n{:#?}",
            Expression::parse_full(
                "2 + (3 + 3 + (9 + 3 * 4 * 9) + 2 + 5 * 7) * 7 * (3 * 6 * 5 * 9 + 6) + 6"
            )?
        );
        Ok(())
    }

    #[test]
    fn parse_input() -> Result<()> {
        let input = read_to_string(&PathBuf::from("input.txt"))?;

        match tuple((Expression::parse, line_ending, Expression::parse))(&input) {
            Ok((i, (first, _, second))) => {
                eprintln!("Parsed first: {:?}\nSecond: {:?}", first, second);
            }
            Err(e) => bail!("{}", e),
        }
        Ok(())
    }
}
