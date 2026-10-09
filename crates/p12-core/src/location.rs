//! Program locations and selections.
//!
//! On the wire a location is a bank (0-7) and a program (0-98). Banks 0-3
//! are User banks U1-U4 and banks 4-7 are Factory banks F1-F4 (manual p.32,
//! Bank Select). For display and parsing, programs are numbered 1-99 as on
//! the panel, e.g. `U1:01`.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

pub const BANKS: u8 = 8;
pub const USER_BANKS: u8 = 4;
pub const PROGRAMS_PER_BANK: u8 = 99;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Location {
    bank: u8,
    program: u8,
}

impl Location {
    /// Builds a location from wire values (bank 0-7, program 0-98).
    pub fn new(bank: u8, program: u8) -> Result<Self> {
        if bank >= BANKS {
            return Err(Error::BankOutOfRange(bank));
        }
        if program >= PROGRAMS_PER_BANK {
            return Err(Error::ProgramOutOfRange(program as u16 + 1));
        }
        Ok(Self { bank, program })
    }

    pub fn bank(self) -> u8 {
        self.bank
    }

    pub fn program(self) -> u8 {
        self.program
    }

    /// User banks are writable; Factory banks are read-only on the synth.
    pub fn is_user(self) -> bool {
        self.bank < USER_BANKS
    }

    /// The location `offset` slots later in the same bank, if it exists.
    pub fn offset(self, offset: usize) -> Option<Self> {
        let program = self.program as usize + offset;
        (program < PROGRAMS_PER_BANK as usize).then_some(Self {
            bank: self.bank,
            program: program as u8,
        })
    }

    pub fn bank_locations(bank: u8) -> impl Iterator<Item = Self> {
        (0..PROGRAMS_PER_BANK).map(move |program| Self { bank, program })
    }

    pub fn user_locations() -> impl Iterator<Item = Self> {
        (0..USER_BANKS).flat_map(Self::bank_locations)
    }
}

pub fn bank_name(bank: u8) -> String {
    if bank < USER_BANKS {
        format!("U{}", bank + 1)
    } else {
        format!("F{}", bank - USER_BANKS + 1)
    }
}

fn parse_bank(s: &str) -> Option<u8> {
    let (kind, number) = s.split_at_checked(1)?;
    let n: u8 = number
        .parse()
        .ok()
        .filter(|n| (1..=USER_BANKS).contains(n))?;
    match kind {
        "U" | "u" => Some(n - 1),
        "F" | "f" => Some(n - 1 + USER_BANKS),
        _ => None,
    }
}

/// Parses a 1-based panel program number into a wire program number.
fn parse_program(s: &str) -> Option<u8> {
    let n: u8 = s.parse().ok()?;
    (1..=PROGRAMS_PER_BANK).contains(&n).then(|| n - 1)
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}", bank_name(self.bank), self.program + 1)
    }
}

impl FromStr for Location {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let invalid = || Error::InvalidLocation(s.to_string());
        let (bank, program) = s.trim().split_once(':').ok_or_else(invalid)?;
        Ok(Self {
            bank: parse_bank(bank).ok_or_else(invalid)?,
            program: parse_program(program).ok_or_else(invalid)?,
        })
    }
}

/// Parses a comma-separated selection into a sorted, de-duplicated set.
///
/// Accepted items: `user` (U1-U4), `factory` (F1-F4), `all`, a bank (`U2`),
/// a single program (`U2:15`) or a range within one bank (`U2:15-30`).
pub fn parse_selection(s: &str) -> Result<BTreeSet<Location>> {
    let mut set = BTreeSet::new();
    for item in s.split(',').map(str::trim).filter(|i| !i.is_empty()) {
        let invalid = || Error::InvalidSelection(item.to_string());
        match item.to_ascii_lowercase().as_str() {
            "user" => set.extend(Location::user_locations()),
            "factory" => set.extend((USER_BANKS..BANKS).flat_map(Location::bank_locations)),
            "all" => set.extend((0..BANKS).flat_map(Location::bank_locations)),
            _ => match item.split_once(':') {
                None => set.extend(Location::bank_locations(
                    parse_bank(item).ok_or_else(invalid)?,
                )),
                Some((bank, programs)) => {
                    let bank = parse_bank(bank).ok_or_else(invalid)?;
                    let (first, last) = match programs.split_once('-') {
                        Some((a, b)) => (parse_program(a), parse_program(b)),
                        None => (parse_program(programs), parse_program(programs)),
                    };
                    let (first, last) = first
                        .zip(last)
                        .filter(|(a, b)| a <= b)
                        .ok_or_else(invalid)?;
                    set.extend((first..=last).map(|program| Location { bank, program }));
                }
            },
        }
    }
    if set.is_empty() {
        return Err(Error::InvalidSelection(s.to_string()));
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_and_parse() {
        let loc = Location::new(1, 4).unwrap();
        assert_eq!(loc.to_string(), "U2:05");
        assert_eq!("u2:5".parse::<Location>().unwrap(), loc);
        assert_eq!(Location::new(7, 98).unwrap().to_string(), "F4:99");
        assert!(Location::new(7, 98).unwrap() > loc);
        assert!(!Location::new(4, 0).unwrap().is_user());
        for bad in ["U5:01", "U1:00", "U1:100", "X1:01", "U1", "U1:"] {
            assert!(bad.parse::<Location>().is_err(), "{bad}");
        }
    }

    #[test]
    fn selections() {
        assert_eq!(parse_selection("user").unwrap().len(), 396);
        assert_eq!(parse_selection("all").unwrap().len(), 792);
        let sel = parse_selection("U1:5-7, U1:6, F1:99").unwrap();
        let names: Vec<_> = sel.iter().map(ToString::to_string).collect();
        assert_eq!(names, ["U1:05", "U1:06", "U1:07", "F1:99"]);
        assert!(parse_selection("U1:7-5").is_err());
        assert!(parse_selection("").is_err());
    }

    #[test]
    fn offset_stays_in_bank() {
        let loc = Location::new(0, 97).unwrap();
        assert_eq!(loc.offset(1).unwrap().to_string(), "U1:99");
        assert_eq!(loc.offset(2), None);
    }
}
