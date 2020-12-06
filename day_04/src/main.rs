use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
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
    let passports = Passport::read_from(&input)?;

    println!("# of passports: {}", passports.len());

    let num_valid = passports.iter().filter(|p| p.valid()).count();
    println!("[Part01] # of valid passports: {}", num_valid);

    for f in VALIDATORS.keys() {
        for p in passports.iter() {
            if p.validate() {
                println!("Valid [{}]: {}", f, p.entries.get(f).unwrap());
            }
        }
    }

    let num_validated = passports.iter().filter(|p| p.validate()).count();
    println!("[Part02] # of validated passports: {}", num_validated);

    Ok(())
}

#[derive(Debug)]
pub struct Passport {
    entries: HashMap<String, String>,
}

lazy_static! {
    pub static ref VALIDATORS: HashMap<String, Regex> = {
        let mut rv = HashMap::new();

        rv.insert("byr".to_string(), Regex::new(r"^\d{4}$").unwrap());
        rv.insert("iyr".to_string(), Regex::new(r"^\d{4}$").unwrap());
        rv.insert("eyr".to_string(), Regex::new(r"^\d{4}$").unwrap());
        rv.insert(
            "hgt".to_string(),
            Regex::new(r"^(?P<value>\d{2,3})(?P<unit>(in|cm))$").unwrap(),
        );
        rv.insert("hcl".to_string(), Regex::new(r"^#[0-9a-f]{6}$").unwrap());
        rv.insert(
            "ecl".to_string(),
            Regex::new(r"^(amb|blu|brn|gry|grn|hzl|oth)$").unwrap(),
        );
        rv.insert("pid".to_string(), Regex::new(r"^\d{9}$").unwrap());

        rv
    };
}

impl Passport {
    const FIELDS_REQUIRED: &'static [&'static str] =
        &["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"];
    const _FIELDS_OPTIONAL: &'static [&'static str] = &["cid"];

    pub fn read_from(input: &Path) -> Result<Vec<Self>> {
        let mut entries = HashMap::new();
        let mut passports = Vec::new();

        for line in io::BufReader::new(File::open(&input)?).lines() {
            let line = line?;
            if line.len() == 0 {
                passports.push(Passport { entries });
                entries = HashMap::new();
            } else {
                for entry in line.split_whitespace() {
                    let key_value: Vec<_> = entry.split(':').collect();
                    if key_value.len() != 2 {
                        bail!("Malformed entry: {}", entry);
                    } else {
                        entries.insert(key_value[0].to_string(), key_value[1].to_string());
                    }
                }
            }
        }
        passports.push(Passport { entries });

        Ok(passports)
    }

    pub fn valid(&self) -> bool {
        Self::FIELDS_REQUIRED
            .iter()
            .all(|f| self.entries.contains_key(*f))
    }

    pub fn validate(&self) -> bool {
        if !self.valid() {
            return false;
        }

        Self::FIELDS_REQUIRED
            .iter()
            .map(|f| -> Option<bool> {
                let entry = self.entries.get(*f)?;
                if !VALIDATORS.get(*f)?.is_match(entry) {
                    return Some(false);
                }
                match *f {
                    "byr" => {
                        let year = entry.parse::<u64>().ok()?;
                        Some(1920 <= year && year <= 2002)
                    }
                    "iyr" => {
                        let year = entry.parse::<u64>().ok()?;
                        Some(2010 <= year && year <= 2020)
                    }
                    "eyr" => {
                        let year = entry.parse::<u64>().ok()?;
                        Some(2020 <= year && year <= 2030)
                    }
                    "hgt" => {
                        let height = VALIDATORS.get(*f)?.captures(entry)?;
                        let value = height["value"].parse::<u64>().ok()?;
                        match &height["unit"] {
                            "cm" => Some(150 <= value && value <= 193),
                            "in" => Some(59 <= value && value <= 76),
                            _ => None, // should not happen
                        }
                    }
                    _ => Some(true),
                }
            })
            .all(|o| match o {
                Some(b) => b,
                None => false,
            })
    }
}
