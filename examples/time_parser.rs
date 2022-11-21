use std::env;
use std::error::Error;

#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "time")]
use time::OffsetDateTime;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let parameters = &args[1..];
    #[cfg(feature = "time")]
    let datetime = htp::parse(&parameters.join(" "), OffsetDateTime::now_utc());
    #[cfg(feature = "chrono")]
    let datetime = htp::parse(&parameters.join(" "), Utc::now());

    match datetime {
        Ok(datetime) => println!("{:?}", datetime),
        Err(e) => println!("{}", e),
    }
    Ok(())
}
