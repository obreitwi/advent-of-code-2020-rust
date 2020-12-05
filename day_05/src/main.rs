use anyhow::{bail, Context, Result};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .with_context(|| "No input provided!")?,
    );

    let bps = BoardingPass::read_from(&input)?;

    part1(&bps[..])?;
    part2(&bps[..])?;

    Ok(())
}

struct BoardingPass {
    row: u64,
    col: u64,
}

impl TryFrom<&str> for BoardingPass {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        let mut row = 0;
        let mut col = 0;

        for c in s.chars() {
            match c {
                'F' => {
                    row = row << 1;
                }
                'B' => {
                    row = row << 1;
                    row += 1;
                }
                'L' => {
                    col = col << 1;
                }
                'R' => {
                    col = col << 1;
                    col += 1
                }
                _ => bail!(format!("Encountered invalid character: {}", c)),
            }
        }

        Ok(Self { row, col })
    }
}

impl BoardingPass {
    pub fn read_from(path: &Path) -> Result<Vec<Self>> {
        let mut bps = Vec::new();
        for line in io::BufReader::new(File::open(&path)?).lines() {
            bps.push(Self::try_from(line?.as_str())?);
        }
        Ok(bps)
    }

    pub fn seat_id(&self) -> u64 {
        (self.row << 3) + self.col
    }
}

fn part1(bps: &[BoardingPass]) -> Result<()> {
    let max = bps
        .iter()
        .map(|b| b.seat_id())
        .max()
        .with_context(|| "No boarding passes provided.")?;

    println!("(part1) Max seat id: {}", max);

    Ok(())
}

fn part2(bps: &[BoardingPass]) -> Result<()> {
    let mut seat_ids: Vec<_> = bps.iter().map(|b| b.seat_id()).collect();
    seat_ids.sort();

    let mut last_id = seat_ids[0];

    for sid in seat_ids.iter().skip(1) {
        if sid - last_id > 1
        {
            println!("(part2) Seat-ID is: {}", sid-1);
            break;
        }
        else {
            last_id = *sid;
        }
    }

    Ok(())
}
