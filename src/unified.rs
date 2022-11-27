use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Sub},
    time::Duration,
};

#[cfg(feature = "chrono")]
use chrono::{Datelike, TimeZone, Timelike};
#[cfg(feature = "time")]
use time::UtcOffset;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum DateTime {
    #[cfg(feature = "time")]
    Time(time::OffsetDateTime),
    #[cfg(feature = "chrono")]
    Chrono(chrono::DateTime<chrono::Utc>),
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => write!(f, "{}", t),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => write!(f, "{}", t),
        }
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => Self::Time(t + rhs),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => Self::Chrono(t + chrono::Duration::from_std(rhs).unwrap()), // chrono wants to be a unique one
        }
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => Self::Time(t - rhs),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => Self::Chrono(t - chrono::Duration::from_std(rhs).unwrap()), // chrono wants to be a unique one
        }
    }
}

impl PartialOrd for DateTime {
    #[allow(unreachable_patterns)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            #[cfg(feature = "time")]
            (Self::Time(a), Self::Time(b)) => a.partial_cmp(b),
            #[cfg(feature = "chrono")]
            (Self::Chrono(a), Self::Chrono(b)) => a.partial_cmp(b),
            _ => unimplemented!("Cannot compare time::OffsetDateTime with chrono::DateTime"),
        }
    }
}

impl PartialEq for DateTime {
    #[allow(unreachable_patterns)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            #[cfg(feature = "time")]
            (Self::Time(self_time), Self::Time(other_time)) => self_time == other_time,
            #[cfg(feature = "chrono")]
            (Self::Chrono(self_chrono), Self::Chrono(other_chrono)) => self_chrono == other_chrono,
            _ => unimplemented!("Cannot compare time::OffsetDateTime with chrono::DateTime"),
        }
    }
}

impl DateTime {
    #[cfg(feature = "time")]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn as_time(self) -> Option<time::OffsetDateTime> {
        match self {
            Self::Time(t) => Some(t),
            #[cfg(feature = "chrono")]
            Self::Chrono(_) => None,
        }
    }

    #[cfg(feature = "chrono")]
    #[allow(clippy::unnecessary_wraps)]
    pub const fn as_chrono(self) -> Option<chrono::DateTime<chrono::Utc>> {
        match self {
            #[cfg(feature = "time")]
            Self::Time(_) => None,
            Self::Chrono(t) => Some(t),
        }
    }

    pub fn weekday(&self) -> Weekday {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => t.weekday().into(),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => t.weekday().into(),
        }
    }

    pub fn and_hms(&self, hour: u8, min: u8, sec: u8) -> Self {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => Self::Time(
                t.replace_time(time::Time::from_hms(hour, min, sec).expect("invalid time")),
            ),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => Self::Chrono(
                t.with_hour(u32::from(hour))
                    .expect("invalid hour")
                    .with_minute(u32::from(min))
                    .expect("invalid minute")
                    .with_second(u32::from(sec))
                    .expect("invalid second"),
            ),
        }
    }

    pub fn and_ymd(&self, year: i32, month: u8, day: u8) -> Self {
        match self {
            #[cfg(feature = "time")]
            Self::Time(t) => Self::Time(
                t.replace_date(
                    time::Date::from_calendar_date(
                        year,
                        time::Month::try_from(month).expect("invalid month"),
                        day,
                    )
                    .expect("invalid date"),
                ),
            ),
            #[cfg(feature = "chrono")]
            Self::Chrono(t) => Self::Chrono(
                t.with_year(year)
                    .expect("invalid year")
                    .with_month(u32::from(month))
                    .expect("invalid month")
                    .with_day(u32::from(day))
                    .expect("invalid day"),
            ),
        }
    }
}

#[cfg(feature = "time")]
impl From<time::OffsetDateTime> for DateTime {
    fn from(t: time::OffsetDateTime) -> Self {
        Self::Time(t.replace_offset(UtcOffset::UTC))
    }
}

#[cfg(feature = "chrono")]
impl<T: TimeZone> From<chrono::DateTime<T>> for DateTime {
    fn from(t: chrono::DateTime<T>) -> Self {
        Self::Chrono(t.with_timezone(&chrono::Utc))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub const fn num_days_from_monday(self) -> u8 {
        self as _
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::Weekday> for Weekday {
    fn from(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday,
        }
    }
}

#[cfg(feature = "time")]
impl From<time::Weekday> for Weekday {
    fn from(weekday: time::Weekday) -> Self {
        match weekday {
            time::Weekday::Monday => Self::Monday,
            time::Weekday::Tuesday => Self::Tuesday,
            time::Weekday::Wednesday => Self::Wednesday,
            time::Weekday::Thursday => Self::Thursday,
            time::Weekday::Friday => Self::Friday,
            time::Weekday::Saturday => Self::Saturday,
            time::Weekday::Sunday => Self::Sunday,
        }
    }
}
