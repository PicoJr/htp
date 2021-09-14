//! # HTP
//! (H)uman (T)ime (P)arser.
//!
//! ## Example
//!
//! ```
//! use chrono::{Utc, TimeZone};
//! use htp::parse;
//! let now = Utc.datetime_from_str("2020-12-24T23:45:00", "%Y-%m-%dT%H:%M:%S").unwrap();
//! let expected = Utc.datetime_from_str("2020-12-18T19:43:00", "%Y-%m-%dT%H:%M:%S").unwrap();
//! let datetime = parse("last friday at 19:43", now).unwrap();
//! assert_eq!(datetime, expected);
//! ```
//!
extern crate pest;
#[macro_use]
extern crate pest_derive;

use chrono::DateTime;
use interpreter::Cfg;
use thiserror::Error;

pub mod interpreter;
pub mod parser;

#[derive(Error, Debug)]
pub enum HTPError {
    #[error(transparent)]
    ParseError(#[from] parser::ParseError),
    #[error(transparent)]
    EvaluationError(#[from] interpreter::EvaluationError),
}

pub fn parse<Tz: chrono::TimeZone>(s: &str, now: DateTime<Tz>) -> Result<DateTime<Tz>, HTPError> {
    let time_clue = parser::parse_time_clue_from_str(s)?;
    let cfg = Cfg {
        now,
        future_if_past: false,
    };
    let datetime = interpreter::evaluate(time_clue, &cfg)?;
    Ok(datetime)
}

pub fn parse_with_cfg<Tz: chrono::TimeZone>(
    s: &str,
    cfg: &Cfg<Tz>,
) -> Result<DateTime<Tz>, HTPError> {
    let time_clue = parser::parse_time_clue_from_str(s)?;
    let datetime = interpreter::evaluate(time_clue, cfg)?;
    Ok(datetime)
}
