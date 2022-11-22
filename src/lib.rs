//! # HTP
//! (H)uman (T)ime (P)arser.
//!
//! ## Example
//!
//! ```rust
//! use htp::parse;
//!
//! // using `time`
//! #[cfg(feature = "time")]
//! {
//!     use time::{PrimitiveDateTime, macros::format_description};
//!     
//!     let now = PrimitiveDateTime::parse("2020-12-24T23:45:00", format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]")).unwrap().assume_utc();
//!     let expected = PrimitiveDateTime::parse("2020-12-18T19:43:00", format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]")).unwrap().assume_utc().into();
//!     let datetime = parse("last friday at 19:43", now).unwrap();
//!     assert_eq!(datetime, expected);
//! }
//!
//! // using `chrono`
//! #[cfg(feature = "chrono")]
//! {
//!     use chrono::{Utc, TimeZone};
//!
//!     let now = Utc.datetime_from_str("2020-12-24T23:45:00", "%Y-%m-%dT%H:%M:%S").unwrap();
//!     let expected = Utc.datetime_from_str("2020-12-18T19:43:00", "%Y-%m-%dT%H:%M:%S").unwrap().into();
//!     let datetime = parse("last friday at 19:43", now).unwrap();
//!     assert_eq!(datetime, expected);
//! }
//! ```
//!

#![warn(clippy::pedantic, clippy::nursery)]

#[cfg(not(any(feature = "time", feature = "chrono")))]
compile_error!("You must enable at least one of the following features: time, chrono");

use thiserror::Error;

pub mod interpreter;
pub mod parser;

mod unified;

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
///
/// # Errors
///
/// - If parsing fails (see [`parser::ParseError`])
/// - If evaluation fails (see [`interpreter::EvaluationError`])
pub fn parse(s: &str, now: impl Into<unified::DateTime>) -> Result<unified::DateTime, HTPError> {
    parse_time_clue(s, now.into(), false)
}

/// Parse time clue from `s` given reference time `now` in timezone `Tz`.
///
/// `assume_next_day`:
/// * if true: times without a day will be interpreted as times during the following the day.
/// e.g. 19:43 will be interpreted as tomorrow at 19:43 if current time is > 19:43.
/// * if false: times without a day will be interpreted as times during current day.
///
/// # Errors
///
/// - If parsing fails (see [`parser::ParseError`])
/// - If evaluation fails (see [`interpreter::EvaluationError`])
pub fn parse_time_clue(
    s: &str,
    now: impl Into<unified::DateTime>,
    assume_next_day: bool,
) -> Result<unified::DateTime, HTPError> {
    let time_clue = parser::parse_time_clue_from_str(s)?;
    let datetime = interpreter::evaluate_time_clue(time_clue, now.into(), assume_next_day)?;
    Ok(datetime)
}
