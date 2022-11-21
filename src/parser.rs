#[cfg(feature = "chrono")]
use chrono::Weekday;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;
#[cfg(feature = "time")]
use time::Weekday;

#[derive(Parser)]
#[grammar = "time.pest"]
pub struct TimeParser;

pub type YMD = (i32, u32, u32);
pub type HMS = (u32, u32, u32);

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid integer")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("unexpected non matching pattern")]
    UnexpectedNonMatchingPattern,
    #[error("unknown weekday: `{0}`")]
    UnknownWeekday(String),
    #[error("unknown shortcut day: `{0}`")]
    UnknownShortcutDay(String),
    #[error("unknown modifier: `{0}`")]
    UnknownModifier(String),
    #[error("unknown quantifier `{0}`")]
    UnknownQuantifier(String),
    #[error("unknown am or pm `{0}`")]
    UnknownAMPM(String),
}

#[cfg(feature = "chrono")]
fn weekday_from(s: &str) -> Result<Weekday, ParseError> {
    match s {
        "monday" | "mon" => Ok(Weekday::Mon),
        "tuesday" | "tue" => Ok(Weekday::Tue),
        "wednesday" | "wed" => Ok(Weekday::Wed),
        "thursday" | "thu" => Ok(Weekday::Thu),
        "friday" | "fri" => Ok(Weekday::Fri),
        "saturday" | "sat" => Ok(Weekday::Sat),
        "sunday" | "sun" => Ok(Weekday::Sun),
        _ => Err(ParseError::UnknownWeekday(s.to_string())),
    }
}

#[cfg(feature = "time")]
fn weekday_from(s: &str) -> Result<Weekday, ParseError> {
    match s {
        "monday" | "mon" => Ok(Weekday::Monday),
        "tuesday" | "tue" => Ok(Weekday::Tuesday),
        "wednesday" | "wed" => Ok(Weekday::Wednesday),
        "thursday" | "thu" => Ok(Weekday::Thursday),
        "friday" | "fri" => Ok(Weekday::Friday),
        "saturday" | "sat" => Ok(Weekday::Saturday),
        "sunday" | "sun" => Ok(Weekday::Sunday),
        _ => Err(ParseError::UnknownWeekday(s.to_string())),
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AMPM {
    AM,
    PM,
}

impl fmt::Display for AMPM {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AMPM::AM => write!(f, "am"),
            AMPM::PM => write!(f, "pm"),
        }
    }
}

fn am_or_pm_from(s: &str) -> Result<AMPM, ParseError> {
    match s {
        "am" => Ok(AMPM::AM),
        "pm" => Ok(AMPM::PM),
        _ => Err(ParseError::UnknownAMPM(s.to_string())),
    }
}

#[derive(Debug, PartialEq)]
pub enum ShortcutDay {
    Today,
    Yesterday,
    Tomorrow,
}

fn shortcut_day_from(s: &str) -> Result<ShortcutDay, ParseError> {
    match s {
        "today" => Ok(ShortcutDay::Today),
        "yesterday" => Ok(ShortcutDay::Yesterday),
        "tomorrow" => Ok(ShortcutDay::Tomorrow),
        _ => Err(ParseError::UnknownShortcutDay(s.to_string())),
    }
}

#[derive(Debug, PartialEq)]
pub enum Modifier {
    Last,
    Next,
}

fn modifier_from(s: &str) -> Result<Modifier, ParseError> {
    match s {
        "last" => Ok(Modifier::Last),
        "next" => Ok(Modifier::Next),
        _ => Err(ParseError::UnknownModifier(s.to_string())),
    }
}

#[derive(Debug, PartialEq)]
pub enum Quantifier {
    Min,
    Hours,
    Days,
    Weeks,
    Months,
}

fn quantifier_from(s: &str) -> Result<Quantifier, ParseError> {
    match s {
        "min" => Ok(Quantifier::Min),
        "hours" | "hour" | "h" => Ok(Quantifier::Hours),
        "days" | "day" | "d" => Ok(Quantifier::Days),
        "weeks" | "week" | "w" => Ok(Quantifier::Weeks),
        "months" | "month" => Ok(Quantifier::Months),
        _ => Err(ParseError::UnknownQuantifier(s.to_string())),
    }
}

#[derive(Debug, PartialEq)]
pub enum TimeClue {
    /// Now.
    Now,
    /// Time without date: "19:43:42", "18", "8", "7pm", "3am"
    Time(HMS, Option<AMPM>),
    /// Relative (past) time clue: "4 minutes ago"
    Relative(usize, Quantifier),
    /// last/next `<day>` at `<time>`: "last friday at 12"
    RelativeDayAt(Modifier, Weekday, Option<HMS>, Option<AMPM>),
    /// Relative (future) time clue: "in 4 minutes"
    RelativeFuture(usize, Quantifier),
    /// Same week day at `<time>`: "monday at 4"
    SameWeekDayAt(Weekday, Option<HMS>, Option<AMPM>),
    /// `<shortcut_day>` at `<time>`: "yesterday at 4", "tomorrow"
    ShortcutDayAt(ShortcutDay, Option<HMS>, Option<AMPM>),
    /// YYYY-MM-DDThh:mm:ss or YYYY/MM/DDThh:mm:ss: "2020-12-25T19:43:00"
    ISO(YMD, HMS),
}

fn parse_time_hms(rules_and_str: &[(Rule, &str)]) -> Result<TimeClue, ParseError> {
    match rules_and_str {
        [(Rule::hms, h)] => {
            let h: u32 = h.parse()?;
            Ok(TimeClue::Time((h, 0, 0), None))
        }
        [(Rule::hms, h), (Rule::hms, m)] => {
            let h: u32 = h.parse()?;
            let m: u32 = m.parse()?;
            Ok(TimeClue::Time((h, m, 0), None))
        }
        [(Rule::hms, h), (Rule::hms, m), (Rule::hms, s)] => {
            let h: u32 = h.parse()?;
            let m: u32 = m.parse()?;
            let s: u32 = s.parse()?;
            Ok(TimeClue::Time((h, m, s), None))
        }
        [(Rule::hms, h), (Rule::am_or_pm, am_or_pm)] => {
            let h: u32 = h.parse()?;
            let am_or_pm = am_or_pm_from(am_or_pm)?;
            Ok(TimeClue::Time((h, 0, 0), Some(am_or_pm)))
        }
        [(Rule::hms, h), (Rule::hms, m), (Rule::am_or_pm, am_or_pm)] => {
            let h: u32 = h.parse()?;
            let m: u32 = m.parse()?;
            let am_or_pm = am_or_pm_from(am_or_pm)?;
            Ok(TimeClue::Time((h, m, 0), Some(am_or_pm)))
        }
        [(Rule::hms, h), (Rule::hms, m), (Rule::hms, s), (Rule::am_or_pm, am_or_pm)] => {
            let h: u32 = h.parse()?;
            let m: u32 = m.parse()?;
            let s: u32 = s.parse()?;
            let am_or_pm = am_or_pm_from(am_or_pm)?;
            Ok(TimeClue::Time((h, m, s), Some(am_or_pm)))
        }
        _ => Err(ParseError::UnexpectedNonMatchingPattern),
    }
}

fn parse_time_clue(pairs: &[Pair<Rule>]) -> Result<TimeClue, ParseError> {
    let rules_and_str: Vec<(Rule, &str)> = pairs
        .iter()
        .map(|pair| (pair.as_rule(), pair.as_str()))
        .collect();
    match rules_and_str.as_slice() {
        [(Rule::time_clue, _), (Rule::now, _), (Rule::EOI, _)] => Ok(TimeClue::Now),
        [(Rule::time_clue, _), (Rule::time, _), time_hms @ .., (Rule::EOI, _)] => {
            parse_time_hms(time_hms)
        }
        [(Rule::time_clue, _), (Rule::relative, _), (Rule::int, s), (Rule::quantifier, q), (Rule::EOI, _)] =>
        {
            let n: usize = s.parse()?;
            let q = quantifier_from(q)?;
            Ok(TimeClue::Relative(n, q))
        }
        [(Rule::time_clue, _), (Rule::relative_future, _), (Rule::int, s), (Rule::quantifier, q), (Rule::EOI, _)] =>
        {
            let n: usize = s.parse()?;
            let q = quantifier_from(q)?;
            Ok(TimeClue::RelativeFuture(n, q))
        }
        [(Rule::time_clue, _), (Rule::day_at, _), (Rule::mday, _), mday @ .., (Rule::EOI, _)] => {
            match mday {
                [(Rule::modifier, m), (Rule::weekday, w), (Rule::time, _), time_hms @ ..] => {
                    let (time_maybe, am_or_pm_maybe) = match parse_time_hms(time_hms)? {
                        TimeClue::Time(hms, am_or_pm) => (Some(hms), am_or_pm),
                        _ => (None, None),
                    };
                    let m = modifier_from(m)?;
                    let w = weekday_from(w)?;
                    Ok(TimeClue::RelativeDayAt(m, w, time_maybe, am_or_pm_maybe))
                }
                [(Rule::modifier, m), (Rule::weekday, w)] => {
                    let m = modifier_from(m)?;
                    let w = weekday_from(w)?;
                    Ok(TimeClue::RelativeDayAt(m, w, None, None))
                }
                [(Rule::weekday, w)] => {
                    let w = weekday_from(w)?;
                    Ok(TimeClue::SameWeekDayAt(w, None, None))
                }
                [(Rule::weekday, w), (Rule::time, _), time_hms @ ..] => {
                    let (time_maybe, am_or_pm_maybe) = match parse_time_hms(time_hms)? {
                        TimeClue::Time(hms, am_or_pm) => (Some(hms), am_or_pm),
                        _ => (None, None),
                    };
                    let w = weekday_from(w)?;
                    Ok(TimeClue::SameWeekDayAt(w, time_maybe, am_or_pm_maybe))
                }
                [(Rule::shortcut_day, r), (Rule::time, _), time_hms @ ..] => {
                    let (time_maybe, am_or_pm_maybe) = match parse_time_hms(time_hms)? {
                        TimeClue::Time(hms, am_or_pm) => (Some(hms), am_or_pm),
                        _ => (None, None),
                    };
                    let r = shortcut_day_from(r)?;
                    Ok(TimeClue::ShortcutDayAt(r, time_maybe, am_or_pm_maybe))
                }
                [(Rule::shortcut_day, r)] => {
                    let r = shortcut_day_from(r)?;
                    Ok(TimeClue::ShortcutDayAt(r, None, None))
                }
                _ => Err(ParseError::UnexpectedNonMatchingPattern),
            }
        }
        [(Rule::time_clue, _), (Rule::iso, _), (Rule::year, y), (Rule::month, m), (Rule::day, d), time_hms @ .., (Rule::EOI, _)] => {
            match parse_time_hms(time_hms)? {
                TimeClue::Time(hms, _) => {
                    let y: i32 = y.parse()?;
                    let m: u32 = m.parse()?;
                    let d: u32 = d.parse()?;
                    Ok(TimeClue::ISO((y, m, d), hms))
                }
                _ => Err(ParseError::UnexpectedNonMatchingPattern),
            }
        }
        [(Rule::time_clue, _), (Rule::date, _), (Rule::day, d), (Rule::month, m), (Rule::year, y), (Rule::EOI, _)] =>
        {
            let y: i32 = y.parse()?;
            let m: u32 = m.parse()?;
            let d: u32 = d.parse()?;
            Ok(TimeClue::ISO((y, m, d), (0, 0, 0)))
        }
        _ => Err(ParseError::UnexpectedNonMatchingPattern),
    }
}

/// Parse time clue from `s`. Prefer `htp::parse`.
///
/// This function is provided in case you wish to interpret time clues
/// yourself. Prefer `htp::parse`.
pub fn parse_time_clue_from_str(s: &str) -> Result<TimeClue, ParseError> {
    let pairs: Pairs<Rule> = TimeParser::parse(Rule::time_clue, s)?;
    let pairs: Vec<Pair<Rule>> = pairs.flatten().collect();
    parse_time_clue(pairs.as_slice())
}

#[cfg(test)]
mod test {
    use crate::parser::{
        parse_time_clue_from_str, Modifier, Quantifier, ShortcutDay, TimeClue, AMPM,
    };
    #[cfg(feature = "chrono")]
    use chrono::Weekday;
    #[cfg(feature = "time")]
    use time::Weekday;

    #[test]
    fn test_parse_time_ok() {
        assert_eq!(
            TimeClue::Time((9, 0, 0), None),
            parse_time_clue_from_str("9").unwrap()
        );
        assert_eq!(
            TimeClue::Time((9, 0, 0), Some(AMPM::AM)),
            parse_time_clue_from_str("9 am").unwrap()
        );
        assert_eq!(
            TimeClue::Time((9, 0, 0), Some(AMPM::PM)),
            parse_time_clue_from_str("9 pm").unwrap()
        );
        assert_eq!(
            TimeClue::Time((9, 30, 0), None),
            parse_time_clue_from_str("9:30").unwrap()
        );
        assert_eq!(
            TimeClue::Time((9, 30, 56), None),
            parse_time_clue_from_str("9:30:56").unwrap()
        );
    }

    #[test]
    fn test_parse_relative_ok() {
        for s in vec!["2 min ago", "2min ago", "2minago", "2   min  ago"].iter() {
            assert_eq!(
                TimeClue::Relative(2, Quantifier::Min),
                parse_time_clue_from_str(s).unwrap()
            );
        }
        for s in vec!["2 h ago", "2 hour ago", "2 hours ago"].iter() {
            assert_eq!(
                TimeClue::Relative(2, Quantifier::Hours),
                parse_time_clue_from_str(s).unwrap()
            );
        }
        for s in vec!["2 d ago", "2 day ago", "2 days ago"].iter() {
            assert_eq!(
                TimeClue::Relative(2, Quantifier::Days),
                parse_time_clue_from_str(s).unwrap()
            );
        }
    }

    #[test]
    fn test_parse_relative_future_ok() {
        for s in vec!["in 2 min", "in 2min", "in2min", "in  2   min"].iter() {
            assert_eq!(
                TimeClue::RelativeFuture(2, Quantifier::Min),
                parse_time_clue_from_str(s).unwrap()
            );
        }
        for s in vec!["in 2 h", "in 2 hour", "in 2 hours"].iter() {
            assert_eq!(
                TimeClue::RelativeFuture(2, Quantifier::Hours),
                parse_time_clue_from_str(s).unwrap()
            );
        }
        for s in vec!["in 2 d", "in 2 day", "in 2 days"].iter() {
            assert_eq!(
                TimeClue::RelativeFuture(2, Quantifier::Days),
                parse_time_clue_from_str(s).unwrap()
            );
        }
    }

    #[test]
    fn test_parse_shortcut_day_ok() {
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Today, None, None),
            parse_time_clue_from_str("today").unwrap()
        );
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Today, Some((7, 0, 0)), None),
            parse_time_clue_from_str("today at 7").unwrap()
        );
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Yesterday, None, None),
            parse_time_clue_from_str("yesterday").unwrap()
        );
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Yesterday, Some((19, 43, 0)), None),
            parse_time_clue_from_str("yesterday at 19:43").unwrap()
        );
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Yesterday, Some((19, 43, 0)), None),
            parse_time_clue_from_str("yesterday at 19:43:00").unwrap()
        );
        assert_eq!(
            TimeClue::ShortcutDayAt(ShortcutDay::Tomorrow, Some((19, 43, 0)), None),
            parse_time_clue_from_str("tomorrow at 19:43:00").unwrap()
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_parse_relative_day_ok() {
        assert_eq!(TimeClue::Now, parse_time_clue_from_str("now").unwrap());
        assert_eq!(
            TimeClue::SameWeekDayAt(Weekday::Fri, Some((19, 43, 0)), None),
            parse_time_clue_from_str("friday at 19:43").unwrap()
        );
        assert_eq!(
            TimeClue::RelativeDayAt(Modifier::Last, Weekday::Fri, None, None),
            parse_time_clue_from_str("last friday").unwrap()
        );
        assert_eq!(
            TimeClue::RelativeDayAt(Modifier::Last, Weekday::Fri, Some((9, 0, 0)), None),
            parse_time_clue_from_str("last friday at 9").unwrap()
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn test_parse_relative_day_ok() {
        assert_eq!(TimeClue::Now, parse_time_clue_from_str("now").unwrap());
        assert_eq!(
            TimeClue::SameWeekDayAt(Weekday::Friday, Some((19, 43, 0)), None),
            parse_time_clue_from_str("friday at 19:43").unwrap()
        );
        assert_eq!(
            TimeClue::RelativeDayAt(Modifier::Last, Weekday::Friday, None, None),
            parse_time_clue_from_str("last friday").unwrap()
        );
        assert_eq!(
            TimeClue::RelativeDayAt(Modifier::Last, Weekday::Friday, Some((9, 0, 0)), None),
            parse_time_clue_from_str("last friday at 9").unwrap()
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_parse_same_week_ok() {
        let weekdays = vec![
            (Weekday::Mon, "monday"),
            (Weekday::Tue, "tuesday"),
            (Weekday::Wed, "wednesday"),
            (Weekday::Thu, "thursday"),
            (Weekday::Fri, "friday"),
            (Weekday::Sat, "saturday"),
            (Weekday::Sun, "sunday"),
        ];
        for (weekday, weekday_str) in weekdays.iter() {
            assert_eq!(
                TimeClue::SameWeekDayAt(weekday.clone(), None, None),
                parse_time_clue_from_str(weekday_str).unwrap()
            )
        }
        let weekdays = vec![
            (Weekday::Mon, "mon"),
            (Weekday::Tue, "tue"),
            (Weekday::Wed, "wed"),
            (Weekday::Thu, "thu"),
            (Weekday::Fri, "fri"),
            (Weekday::Sat, "sat"),
            (Weekday::Sun, "sun"),
        ];
        for (weekday, weekday_str) in weekdays.iter() {
            assert_eq!(
                TimeClue::SameWeekDayAt(weekday.clone(), None, None),
                parse_time_clue_from_str(weekday_str).unwrap()
            )
        }
        assert_eq!(
            TimeClue::SameWeekDayAt(Weekday::Fri, Some((19, 43, 0)), None),
            parse_time_clue_from_str("friday at 19:43").unwrap()
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn test_parse_same_week_ok() {
        let weekdays = vec![
            (Weekday::Monday, "monday"),
            (Weekday::Tuesday, "tuesday"),
            (Weekday::Wednesday, "wednesday"),
            (Weekday::Thursday, "thursday"),
            (Weekday::Friday, "friday"),
            (Weekday::Saturday, "saturday"),
            (Weekday::Sunday, "sunday"),
        ];
        for (weekday, weekday_str) in weekdays.iter() {
            assert_eq!(
                TimeClue::SameWeekDayAt(*weekday, None, None),
                parse_time_clue_from_str(weekday_str).unwrap()
            )
        }
        let weekdays = vec![
            (Weekday::Monday, "mon"),
            (Weekday::Tuesday, "tue"),
            (Weekday::Wednesday, "wed"),
            (Weekday::Thursday, "thu"),
            (Weekday::Friday, "fri"),
            (Weekday::Saturday, "sat"),
            (Weekday::Sunday, "sun"),
        ];
        for (weekday, weekday_str) in weekdays.iter() {
            assert_eq!(
                TimeClue::SameWeekDayAt(*weekday, None, None),
                parse_time_clue_from_str(weekday_str).unwrap()
            )
        }
        assert_eq!(
            TimeClue::SameWeekDayAt(Weekday::Friday, Some((19, 43, 0)), None),
            parse_time_clue_from_str("friday at 19:43").unwrap()
        );
    }

    #[test]
    fn test_parse_now_ok() {
        assert_eq!(TimeClue::Now, parse_time_clue_from_str("now").unwrap());
    }

    #[test]
    fn test_parse_iso_ok() {
        assert_eq!(
            TimeClue::ISO((2020, 12, 25), (19, 43, 42)),
            parse_time_clue_from_str("2020-12-25T19:43:42").unwrap()
        );

        assert_eq!(
            TimeClue::ISO((2020, 12, 25), (0, 0, 0)),
            parse_time_clue_from_str("25/12/2020").unwrap()
        );

        assert_eq!(
            TimeClue::ISO((2020, 12, 25), (0, 0, 0)),
            parse_time_clue_from_str("25-12-2020").unwrap()
        );
    }
}
