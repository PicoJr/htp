use std::env;
use std::error::Error;

use chrono::Utc;
use time::OffsetDateTime;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let parameters = &args[1..];

    let time_result = htp::parse(&parameters.join(" "), OffsetDateTime::now_utc());

    match time_result {
        Ok(datetime) => println!("time: {}", datetime),
        Err(e) => println!("time: {}", e),
    }

    let chrono_result = htp::parse(&parameters.join(" "), Utc::now());

    match chrono_result {
        Ok(datetime) => println!("chrono: {}", datetime),
        Err(e) => println!("chrono: {}", e),
    }

    Ok(())
}
