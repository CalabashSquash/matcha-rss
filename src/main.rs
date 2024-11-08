use clap::arg;
use clap::ArgAction;
use clap::Parser;
use std::{env, fs};
use chrono::Utc;

use matcha_rss::{
    digest::{build_rss_digest, write_digest},
    rss::parse_feed,
    weather::{build_weather_digest, get_weather_forecast},
    yaml::FeedInputs,
};

// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// struct Args {
//     output_file: String
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_yaml = fs::read_to_string("test/config.yaml")?;
    let inputs: FeedInputs = serde_yaml::from_str(&raw_yaml)?;

    let cmd = clap::Command::new("matcha").arg(
        arg!(-p --prefix <FILE> "The prefix for the output file path to write feed to")
            // .action(ArgAction::Set)
            .required(false)
            .default_value("")
            
    );

    let matches = cmd.get_matches();
    // Unwraps because default_value means it should always have a value.
    let output_prefix = matches.get_one::<String>("prefix").unwrap();

    let weather = get_weather_forecast();
    let mut digest = build_weather_digest(weather);

    for feed in inputs.feeds {
        let feed = parse_feed(feed)?;
        build_rss_digest(&mut digest, feed);
    }

    write_digest(digest, format!("{}{}.md", output_prefix, Utc::now().format("%d-%m-%y")))?;
    println!("Done");
    Ok(())
}
