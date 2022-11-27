use crate::parser::{Modifier, Quantifier, ShortcutDay, TimeClue, AMPM, HMS};
use crate::unified;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
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

const fn check_hms(hms: HMS, am_or_pm_maybe: Option<AMPM>) -> Result<HMS, EvaluationError> {
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

/// Same as `evaluate(time_clue, now)`
///
/// # Errors
/// See [`EvaluationError`]
#[cfg(any(feature = "chrono", feature = "time"))]
pub fn evaluate(
    time_clue: TimeClue,
    now: unified::DateTime,
) -> Result<unified::DateTime, EvaluationError> {
    evaluate_time_clue(time_clue, now, false)
}

/// Evaluate `time_clue` given reference time `now`.
///
/// `assume_next_day`:
/// * if true: times without a day will be interpreted as times during the following the day.
/// e.g. 19:43 will be interpreted as tomorrow at 19:43 if current time is > 19:43.
/// * if false: times without a day will be interpreted as times during current day.
///
/// # Errors
/// See [`EvaluationError`]
#[cfg(any(feature = "chrono", feature = "time"))]
#[allow(clippy::cast_possible_truncation)] // QUERY: Would it make more sense to use `u8` instead of `u32` for `HMS`, and month/day on `YMD`?
pub fn evaluate_time_clue(
    time_clue: TimeClue,
    now: unified::DateTime,
    assume_next_day: bool, // assume next day if only time is supplied and time < now
) -> Result<unified::DateTime, EvaluationError> {
    match time_clue {
        TimeClue::Now => Ok(now),
        TimeClue::Time((h, m, s), am_or_pm_maybe) => {
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let d = now.and_hms(h as u8, m as u8, s as u8);
            if assume_next_day && d < now {
                Ok(d + Duration::from_secs(24 * 60 * 60))
            } else {
                Ok(d)
            }
        }
        TimeClue::Relative(n, quantifier) => match quantifier {
            Quantifier::Min => Ok(now - Duration::from_secs(n as u64 * 60)),
            Quantifier::Hours => Ok(now - Duration::from_secs(n as u64 * 60 * 60)),
            Quantifier::Days => Ok(now - Duration::from_secs(n as u64 * 24 * 60 * 60)),
            Quantifier::Weeks => Ok(now - Duration::from_secs(n as u64 * 7 * 24 * 60 * 60)),
            Quantifier::Months => Ok(now - Duration::from_secs(30 * (n as u64 * 7 * 24 * 60 * 60))), // assume 1 month = 30 days
        },
        TimeClue::RelativeFuture(n, quantifier) => match quantifier {
            Quantifier::Min => Ok(now + Duration::from_secs(n as u64 * 60)),
            Quantifier::Hours => Ok(now + Duration::from_secs(n as u64 * 60 * 60)),
            Quantifier::Days => Ok(now + Duration::from_secs(n as u64 * 24 * 60 * 60)),
            Quantifier::Weeks => Ok(now + Duration::from_secs(n as u64 * 7 * 24 * 60 * 60)),
            Quantifier::Months => Ok(now + Duration::from_secs(30 * (n as u64 * 7 * 24 * 60 * 60))), // assume 1 month = 30 days
        },
        TimeClue::RelativeDayAt(modifier, weekday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let monday = now
                - Duration::from_secs(
                    u64::from(now.weekday().num_days_from_monday()) * 24 * 60 * 60,
                );
            match modifier {
                Modifier::Last => {
                    let same_week_day = monday
                        + (Duration::from_secs(
                            u64::from(weekday.num_days_from_monday()) * 24 * 60 * 60,
                        ));
                    if weekday.num_days_from_monday() < now.weekday().num_days_from_monday() {
                        Ok(same_week_day.and_hms(h as u8, m as u8, s as u8)) // same week
                    } else {
                        Ok(same_week_day.and_hms(h as u8, m as u8, s as u8)
                            - Duration::from_secs(7 * 24 * 60 * 60))
                        // last week
                    }
                }
                Modifier::Next => {
                    let same_week_day = monday
                        + (Duration::from_secs(
                            u64::from(weekday.num_days_from_monday()) * 24 * 60 * 60,
                        ));
                    if weekday.num_days_from_monday() > now.weekday().num_days_from_monday() {
                        Ok(same_week_day.and_hms(h as u8, m as u8, s as u8)) // same week
                    } else {
                        Ok(same_week_day.and_hms(h as u8, m as u8, s as u8)
                            + Duration::from_secs(7 * 24 * 60 * 60))
                        // next week
                    }
                }
            }
        }
        TimeClue::SameWeekDayAt(weekday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let monday = now
                - Duration::from_secs(
                    u64::from(now.weekday().num_days_from_monday()) * 24 * 60 * 60,
                );
            Ok((monday
                + Duration::from_secs(u64::from(weekday.num_days_from_monday()) * 24 * 60 * 60))
            .and_hms(h as u8, m as u8, s as u8))
        }
        TimeClue::ShortcutDayAt(rday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            match rday {
                ShortcutDay::Today => Ok(now.and_hms(h as u8, m as u8, s as u8)),
                ShortcutDay::Yesterday => Ok(
                    (now - Duration::from_secs(24 * 60 * 60)).and_hms(h as u8, m as u8, s as u8)
                ),
                ShortcutDay::Tomorrow => Ok(
                    (now + Duration::from_secs(24 * 60 * 60)).and_hms(h as u8, m as u8, s as u8)
                ),
            }
        }
        TimeClue::ISO((year, month, day), (h, m, s)) => {
            let utc = now
                .and_ymd(year, month as u8, day as u8)
                .and_hms(h as u8, m as u8, s as u8);

            Ok(utc)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::{check_hms, evaluate, evaluate_time_clue};
    use crate::parser::AMPM::{AM, PM};
    use crate::parser::{Modifier, TimeClue};
    #[cfg(feature = "chrono")]
    use chrono::{offset::TimeZone, Utc, Weekday as ChronoWeekday};
    #[cfg(feature = "time")]
    use time::{macros::format_description, PrimitiveDateTime, Weekday as TimeWeekday};

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
    #[cfg(feature = "chrono")]
    fn chrono_test_next_weekday() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday
        let expected = Utc
            .datetime_from_str("2020-07-17T00:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .into();

        assert_eq!(
            evaluate(
                TimeClue::RelativeDayAt(Modifier::Next, ChronoWeekday::Fri.into(), None, None),
                now.into()
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn time_test_next_weekday() {
        let now = PrimitiveDateTime::parse(
            "2020-07-12T12:45:00Z",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z"),
        )
        .expect("failed to parse origin date")
        .assume_utc(); // sunday
        let expected = PrimitiveDateTime::parse(
            "2020-07-17T00:00:00Z",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z"),
        )
        .expect("failed to parse expected date")
        .assume_utc()
        .into();

        assert_eq!(
            evaluate(
                TimeClue::RelativeDayAt(Modifier::Next, TimeWeekday::Friday.into(), None, None),
                now.into()
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_test_assume_next_day() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday

        let expected = Utc
            .datetime_from_str("2020-07-12T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .into();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now.into(), false).unwrap(),
            expected
        );

        let expected = Utc
            .datetime_from_str("2020-07-13T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .into();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now.into(), true).unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn time_test_assume_next_day() {
        let now = PrimitiveDateTime::parse(
            "2020-07-12T12:45:00",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .expect("failed to parse test date")
        .assume_utc(); // sunday

        let expected = PrimitiveDateTime::parse(
            "2020-07-12T08:00:00",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .expect("failed to parse expected date")
        .assume_utc()
        .into();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now.into(), false).unwrap(),
            expected
        );

        let expected = PrimitiveDateTime::parse(
            "2020-07-13T08:00:00",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .expect("failed to parse expected date")
        .assume_utc()
        .into();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now.into(), true).unwrap(),
            expected
        );
    }
}
