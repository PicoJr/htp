use crate::parser::{Modifier, Quantifier, ShortcutDay, TimeClue, AMPM, HMS};
#[cfg(feature = "chrono")]
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone, Utc};
use thiserror::Error;
#[cfg(feature = "time")]
use time::{macros::format_description, Duration, OffsetDateTime};

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

/// Same as `evaluate(time_clue, now)`
#[cfg(feature = "chrono")]
pub fn evaluate<Tz: chrono::TimeZone>(
    time_clue: TimeClue,
    now: DateTime<Tz>,
) -> Result<DateTime<Tz>, EvaluationError> {
    evaluate_time_clue(time_clue, now, false)
}

#[cfg(feature = "time")]
pub fn evaluate(
    time_clue: TimeClue,
    now: OffsetDateTime,
) -> Result<OffsetDateTime, EvaluationError> {
    evaluate_time_clue(time_clue, now, false)
}

/// Evaluate `time_clue` given reference time `now`.
///
/// `assume_next_day`:
/// * if true: times without a day will be interpreted as times during the following the day.
/// e.g. 19:43 will be interpreted as tomorrow at 19:43 if current time is > 19:43.
/// * if false: times without a day will be interpreted as times during current day.
#[cfg(feature = "chrono")]
pub fn evaluate_time_clue<Tz: chrono::TimeZone>(
    time_clue: TimeClue,
    now: DateTime<Tz>,
    assume_next_day: bool, // assume next day if only time is supplied and time < now
) -> Result<DateTime<Tz>, EvaluationError> {
    match time_clue {
        TimeClue::Now => Ok(now),
        TimeClue::Time((h, m, s), am_or_pm_maybe) => {
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let d = now.date().and_hms(h, m, s);
            if assume_next_day && d < now {
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
                ShortcutDay::Tomorrow => Ok((now.date() + Duration::days(1)).and_hms(h, m, s)),
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

#[cfg(feature = "time")]
pub fn evaluate_time_clue(
    time_clue: TimeClue,
    now: OffsetDateTime,
    assume_next_day: bool, // assume next day if only time is supplied and time < now
) -> Result<OffsetDateTime, EvaluationError> {
    match time_clue {
        TimeClue::Now => Ok(now),
        TimeClue::Time((h, m, s), am_or_pm_maybe) => {
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let d = now
                .date()
                .with_hms(h as u8, m as u8, s as u8)
                .map_err(|_| EvaluationError::InvalidTime {
                    hour: h,
                    minute: m,
                    second: s,
                })?
                .assume_offset(now.offset());
            if assume_next_day && d < now {
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
            let monday =
                now.date() - Duration::days(now.weekday().number_days_from_monday() as i64);
            match modifier {
                Modifier::Last => {
                    let same_week_day =
                        monday + (Duration::days(weekday.number_days_from_monday() as i64));
                    if weekday.number_days_from_monday() < now.weekday().number_days_from_monday() {
                        Ok(same_week_day
                            .with_hms(h as u8, m as u8, s as u8)
                            .map_err(|_| EvaluationError::InvalidTime {
                                hour: h,
                                minute: m,
                                second: s,
                            })?
                            .assume_offset(now.offset())) // same week
                    } else {
                        Ok(same_week_day
                            .with_hms(h as u8, m as u8, s as u8)
                            .map_err(|_| EvaluationError::InvalidTime {
                                hour: h,
                                minute: m,
                                second: s,
                            })?
                            .assume_offset(now.offset())
                            - Duration::days(7)) // last week
                    }
                }
                Modifier::Next => {
                    let same_week_day =
                        monday + (Duration::days(weekday.number_days_from_monday() as i64));
                    if weekday.number_days_from_monday() > now.weekday().number_days_from_monday() {
                        Ok(same_week_day
                            .with_hms(h as u8, m as u8, s as u8)
                            .map_err(|_| EvaluationError::InvalidTime {
                                hour: h,
                                minute: m,
                                second: s,
                            })?
                            .assume_offset(now.offset())) // same week
                    } else {
                        Ok(same_week_day
                            .with_hms(h as u8, m as u8, s as u8)
                            .map_err(|_| EvaluationError::InvalidTime {
                                hour: h,
                                minute: m,
                                second: s,
                            })?
                            .assume_offset(now.offset())
                            + Duration::days(7)) // next week
                    }
                }
            }
        }
        TimeClue::SameWeekDayAt(weekday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            let monday =
                now.date() - Duration::days(now.weekday().number_days_from_monday() as i64);
            Ok(
                (monday + Duration::days(weekday.number_days_from_monday() as i64))
                    .with_hms(h as u8, m as u8, s as u8)
                    .map_err(|_| EvaluationError::InvalidTime {
                        hour: h,
                        minute: m,
                        second: s,
                    })?
                    .assume_offset(now.offset()),
            )
        }
        TimeClue::ShortcutDayAt(rday, hms_maybe, am_or_pm_maybe) => {
            let (h, m, s) = hms_maybe.unwrap_or((0, 0, 0));
            let (h, m, s) = check_hms((h, m, s), am_or_pm_maybe)?;
            match rday {
                ShortcutDay::Today => Ok(now
                    .date()
                    .with_hms(h as u8, m as u8, s as u8)
                    .map_err(|_| EvaluationError::InvalidTime {
                        hour: h,
                        minute: m,
                        second: s,
                    })?
                    .assume_offset(now.offset())),
                ShortcutDay::Yesterday => Ok((now.date() - Duration::days(1))
                    .with_hms(h as u8, m as u8, s as u8)
                    .map_err(|_| EvaluationError::InvalidTime {
                        hour: h,
                        minute: m,
                        second: s,
                    })?
                    .assume_offset(now.offset())),
                ShortcutDay::Tomorrow => Ok((now.date() + Duration::days(1))
                    .with_hms(h as u8, m as u8, s as u8)
                    .map_err(|_| EvaluationError::InvalidTime {
                        hour: h,
                        minute: m,
                        second: s,
                    })?
                    .assume_offset(now.offset())),
            }
        }
        TimeClue::ISO((year, month, day), (h, m, s)) => {
            let fmt = format_description!("YYYY-mm-dd HH:MM:SS");
            let utc = OffsetDateTime::parse(
                &format!("{year:>04}-{month:>02}-{day:>02} {h:>02}:{m:>02}:{s:>02}"),
                fmt,
            )
            .map_err(|_| EvaluationError::ChronoISOError {
                year,
                month,
                day,
                hour: h,
                minute: m,
                second: s,
            })?;

            Ok(utc.replace_offset(now.offset()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interpreter::{check_hms, evaluate, evaluate_time_clue};
    use crate::parser::AMPM::{AM, PM};
    use crate::parser::{Modifier, TimeClue};
    #[cfg(feature = "chrono")]
    use chrono::{offset::TimeZone, Utc, Weekday};
    #[cfg(feature = "time")]
    use time::{macros::format_description, PrimitiveDateTime, Weekday};

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
    fn test_next_weekday() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday
        let expected = Utc
            .datetime_from_str("2020-07-17T00:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        assert_eq!(
            evaluate(
                TimeClue::RelativeDayAt(Modifier::Next, Weekday::Fri, None, None),
                now
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn test_next_weekday() {
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
        .assume_utc();

        assert_eq!(
            evaluate(
                TimeClue::RelativeDayAt(Modifier::Next, Weekday::Friday, None, None),
                now
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_assume_next_day() {
        let now = Utc
            .datetime_from_str("2020-07-12T12:45:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap(); // sunday

        let expected = Utc
            .datetime_from_str("2020-07-12T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now.clone(), false).unwrap(),
            expected
        );

        let expected = Utc
            .datetime_from_str("2020-07-13T08:00:00", "%Y-%m-%dT%H:%M:%S")
            .unwrap();
        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now, true).unwrap(),
            expected
        );
    }

    #[test]
    #[cfg(feature = "time")]
    fn test_assume_next_day() {
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
        .assume_utc();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now, false).unwrap(),
            expected
        );

        let expected = PrimitiveDateTime::parse(
            "2020-07-13T08:00:00",
            format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        )
        .expect("failed to parse expected date")
        .assume_utc();

        assert_eq!(
            evaluate_time_clue(TimeClue::Time((8, 0, 0), None), now, true).unwrap(),
            expected
        );
    }
}
