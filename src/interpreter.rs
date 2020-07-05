use crate::parser::{Modifier, Quantifier, ShortcutDay, TimeClue, AMPM, HMS};
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("invalid time: {hour}:{minute}:{second}")]
    InvalidTime {
        hour: u32,
        minute: u32,
        second: u32,
        am_or_pm_maybe: Option<AMPM>,
    },
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

fn check_hms(hms: HMS, am_or_pm_maybe: Option<AMPM>) -> Result<HMS, EvaluationError> {
    let (h, m, s) = hms;
    let h = match am_or_pm_maybe {
        None | Some(AMPM::AM) => h,
        Some(AMPM::PM) => h + 12,
    };
    if h < 24 && m < 60 && s < 60 {
        Ok((h, m, s))
    } else {
        Err(EvaluationError::InvalidTime {
            hour: h,
            minute: m,
            second: s,
            am_or_pm_maybe,
        })
    }
}

pub fn evaluate<Tz: chrono::TimeZone>(
    time_clue: TimeClue,
    now: DateTime<Tz>,
) -> Result<DateTime<Tz>, EvaluationError> {
    match time_clue {
        TimeClue::Now => Ok(now),
        TimeClue::Time((h, m, s), am_or_pm_maybe) => {
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            Ok(now.date().and_hms(h, m, s))
        }
        TimeClue::Relative(n, quantifier) => match quantifier {
            Quantifier::Min => Ok(now - Duration::minutes(n as i64)),
            Quantifier::Hours => Ok(now - Duration::hours(n as i64)),
            Quantifier::Days => Ok(now - Duration::days(n as i64)),
            Quantifier::Weeks => Ok(now - Duration::weeks(n as i64)),
            Quantifier::Months => Ok(now - Duration::days(30 * n as i64)), // assume 1 month = 30 days
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
                    if weekday.num_days_from_monday() < now.weekday().num_days_from_monday() {
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
