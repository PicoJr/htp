use crate::parser::{Modifier, Quantifier, ShortcutDay, TimeClue, AMPM, HMS};
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone, Utc};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EvaluationError {
    #[error("invalid time: {hour}:{minute}:{second} {am_or_pm}")]
    InvalidTimeAMPM {
        hour: u32,
        minute: u32,
        second: u32,
        am_or_pm: AMPM,
    },
    #[error("invalid time: {hour}:{minute}:{second}")]
    InvalidTime { hour: u32, minute: u32, second: u32 },
    #[error("invalid ISO date: {year}-{month}-{day}T{hour}:{minute}:{second}")]
    ChronoISOError {
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    },
}

/// Configure the interpreter
pub struct Cfg<Tz: TimeZone> {
    /// Set the reference time
    pub now: DateTime<Tz>,
    /// If a parse gives a date before `now`, go to next day (used for time format like "8:00")
    pub future_if_past: bool,
}

impl<Tz: TimeZone> Cfg<Tz> {
    /// Create a configuration object
    pub fn new(now: DateTime<Tz>) -> Self {
        Cfg {
            now,
            future_if_past: false,
        }
    }

    /// Set if the interpreter jump to the next day if ever the parsed date is before `now`
    pub fn set_future_if_past(self, future_if_past: bool) -> Self {
        Cfg {
            now: self.now,
            future_if_past,
        }
    }
}

fn check_hms(hms: HMS, am_or_pm_maybe: Option<AMPM>) -> Result<HMS, EvaluationError> {
    let (h, m, s) = hms;
    let h_am_pm = match am_or_pm_maybe {
        None | Some(AMPM::AM) => h,
        Some(AMPM::PM) => h + 12,
    };
    if h_am_pm < 24 && m < 60 && s < 60 {
        Ok((h_am_pm, m, s))
    } else {
        match am_or_pm_maybe {
            Some(am_or_pm) => Err(EvaluationError::InvalidTimeAMPM {
                hour: h,
                minute: m,
                second: s,
                am_or_pm,
            }),
            None => Err(EvaluationError::InvalidTime {
                hour: h,
                minute: m,
                second: s,
            }),
        }
    }
}

pub fn evaluate<Tz: chrono::TimeZone>(
    time_clue: TimeClue,
    cfg: &Cfg<Tz>,
) -> Result<DateTime<Tz>, EvaluationError> {
    let now = cfg.now.clone();
    match time_clue {
        TimeClue::Now => Ok(now),
        TimeClue::Time((h, m, s), am_or_pm_maybe) => {
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let d = now.date().and_hms(h, m, s);
            if cfg.future_if_past && d < now {
                Ok(d + Duration::days(1))
            } else {
                Ok(d)
            }
        }
        TimeClue::Relative(n, quantifier) => match quantifier {
            Quantifier::Min => Ok(now - Duration::minutes(n as i64)),
            Quantifier::Hours => Ok(now - Duration::hours(n as i64)),
            Quantifier::Days => Ok(now - Duration::days(n as i64)),
            Quantifier::Weeks => Ok(now - Duration::weeks(n as i64)),
            Quantifier::Months => Ok(now - Duration::days(30 * n as i64)), // assume 1 month = 30 days
        },
        TimeClue::RelativeFuture(n, quantifier) => match quantifier {
            Quantifier::Min => Ok(now + Duration::minutes(n as i64)),
            Quantifier::Hours => Ok(now + Duration::hours(n as i64)),
            Quantifier::Days => Ok(now + Duration::days(n as i64)),
            Quantifier::Weeks => Ok(now + Duration::weeks(n as i64)),
            Quantifier::Months => Ok(now + Duration::days(30 * n as i64)), // assume 1 month = 30 days
        },
        TimeClue::RelativeDayAt(modifier, weekday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let monday = now.date() - Duration::days(now.weekday().num_days_from_monday() as i64);
            match modifier {
                Modifier::Last => {
                    let same_week_day =
                        monday + (Duration::days(weekday.num_days_from_monday() as i64));
                    if weekday.num_days_from_monday() < now.weekday().num_days_from_monday() {
                        Ok(same_week_day.and_hms(h, m, s)) // same week
                    } else {
                        Ok(same_week_day.and_hms(h, m, s) - Duration::days(7)) // last week
                    }
                }
                Modifier::Next => {
                    let same_week_day =
                        monday + (Duration::days(weekday.num_days_from_monday() as i64));
                    if weekday.num_days_from_monday() > now.weekday().num_days_from_monday() {
                        Ok(same_week_day.and_hms(h, m, s)) // same week
                    } else {
                        Ok(same_week_day.and_hms(h, m, s) + Duration::days(7)) // next week
                    }
                }
            }
        }
        TimeClue::SameWeekDayAt(weekday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let monday = now.date() - Duration::days(now.weekday().num_days_from_monday() as i64);
            Ok((monday + Duration::days(weekday.num_days_from_monday() as i64)).and_hms(h, m, s))
        }
        TimeClue::ShortcutDayAt(rday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            match rday {
                ShortcutDay::Today => Ok(now.date().and_hms(h, m, s)),
                ShortcutDay::Yesterday => Ok((now.date() - Duration::days(1)).and_hms(h, m, s)),
            }
        }
        TimeClue::ISO((year, month, day), (h, m, s)) => {
            let utc = Utc.ymd_opt(year, month, day).and_hms_opt(h, m, s);
            match utc {
                LocalResult::Single(utc) => Ok(utc.with_timezone(&now.timezone())),
                _ => Err(EvaluationError::ChronoISOError {
                    year,
                    month,
                    day,
                    hour: h,
                    minute: m,
                    second: s,
                }),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::{check_hms, evaluate, Cfg};
    use crate::parser::AMPM::{AM, PM};
    use crate::parser::{Modifier, TimeClue};
    use chrono::offset::TimeZone;
    use chrono::Utc;
    use chrono::Weekday;

    #[test]
    fn test_check_hms() {
        assert_eq!(check_hms((19, 43, 42), None), Ok((19, 43, 42)));
        assert_eq!(check_hms((19, 43, 42), Some(AM)), Ok((19, 43, 42)));
        assert!(check_hms((19, 43, 42), Some(PM)).is_err());
        assert!(check_hms((24, 43, 42), None).is_err());
        assert!(check_hms((19, 63, 42), None).is_err());
        assert!(check_hms((19, 43, 62), None).is_err());
        assert_eq!(check_hms((6, 42, 43), Some(PM)), Ok((18, 42, 43)));
    }

    #[test]
    fn test_next_weekday() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday
        let expected = Utc
            .datetime_from_str("2020-07-17T00:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        let cfg = Cfg::new(now);
        assert_eq!(
            evaluate(
                TimeClue::RelativeDayAt(Modifier::Next, Weekday::Fri, None, None),
                &cfg
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn test_future_if_past() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday

        let cfg = Cfg::new(now);
        let expected = Utc
            .datetime_from_str("2020-07-12T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        assert_eq!(
            evaluate(TimeClue::Time((8, 0, 0), None), &cfg).unwrap(),
            expected
        );

        let expected = Utc
            .datetime_from_str("2020-07-13T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        let cfg = Cfg::new(now).set_future_if_past(true);
        assert_eq!(
            evaluate(TimeClue::Time((8, 0, 0), None), &cfg).unwrap(),
            expected
        );
    }
}
