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

/// Same as `parse_time_clue(s, now, false)`
///
/// Parse time clue from `s` given reference time `now` in timezone `Tz`.
pub fn parse<Tz: chrono::TimeZone>(s: &str, now: DateTime<Tz>) -> Result<DateTime<Tz>, HTPError> {
    parse_time_clue(s, now, false)
}

/// Parse time clue from `s` given reference time `now` in timezone `Tz`.
///
/// `assume_next_day`:
/// * if true: times without a day will be interpreted as times during the following the day.
/// e.g. 19:43 will be interpreted as tomorrow at 19:43 if current time is > 19:43.
/// * if false: times without a day will be interpreted as times during current day.
pub fn parse_time_clue<Tz: chrono::TimeZone>(
    s: &str,
    now: DateTime<Tz>,
    assume_next_day: bool,
) -> Result<DateTime<Tz>, HTPError> {
    let time_clue = parser::parse_time_clue_from_str(s)?;
    let datetime = interpreter::evaluate_time_clue(time_clue, now, assume_next_day)?;
    Ok(datetime)
}
