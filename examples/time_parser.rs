use chrono::Local;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let parameters = &args[1..];
    let datetime = htp::parse(&parameters.join(" "), Local::now());
    match datetime {
        Ok(datetime) => println!("{:?}", datetime),
        Err(e) => println!("{}", e),
    }
    Ok(())
}
