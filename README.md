[![htp crate](https://img.shields.io/crates/v/htp.svg)](https://crates.io/crates/htp)
[![tbl documentation](https://docs.rs/htp/badge.svg)](https://docs.rs/htp)
[![GitHub license](https://img.shields.io/github/license/PicoJr/htp)](https://github.com/PicoJr/htp/blob/master/LICENSE)
# HTP

Work in progress **H**uman **T**ime **P**arser

This lib uses [pest](https://github.com/pest-parser/pest) for parsing.

## Example

```rust
use chrono::{Utc, TimeZone};
use htp::parse_and_interpret;
let now = Utc.datetime_from_str("2020-12-24T23:45:00", "%Y-%m-%dT%H:%M:%S").unwrap();
let expected = Utc.datetime_from_str("2020-12-18T19:43:00", "%Y-%m-%dT%H:%M:%S").unwrap();
let datetime = parse_and_interpret("last friday at 19:43", now).unwrap();
assert_eq!(datetime, expected);
```

## Similar Crate

* [chrono-english](https://github.com/stevedonovan/chrono-english).

## Why?

Tweak how time is parsed and interpreted inside my [rust time tracking tool](https://github.com/PicoJr/rtw).

It's fun to write parsers once in while, [pest](https://github.com/pest-parser/pest) is really nice.

## What date format can it parse?

see [time_clue grammar rule](src/time.pest).

some examples:

* `4 min ago`, `4 h ago`, `1 week ago`
* `last friday at 19`, `monday at 6 am`
* `7`, `7am`, `7pm`, `7:30`, `19:43:00`
* `now`, `yesterday`, `today`, `friday`
* `2020-12-25T19:43:00`

It also supports _interestingly-spaced_ inputs such as:
```
4           min      ago
```

It is possible to try `HTP` out using `cargo run --example time_parser`:

example
```
cargo run --example time_parser last friday at 6
```

output
```
2020-07-03T06:00:00+02:00
```

Thanks to pest it also provides meaningful errors:

example
```
cargo run --example time_parser last friday at
```

output
```
 --> 1:15
  |
1 | last friday at
  |               ^---
  |
  = expected hms
```

## Changelog

Please see the [CHANGELOG](CHANGELOG.md) for a release history.
