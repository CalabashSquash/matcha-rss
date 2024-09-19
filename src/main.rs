use matcha_rss::{
    digest::{build_digest, write_digest},
    rss::parse_feed,
    yaml::FeedInputs,
};
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = fs::read_to_string("test/config.yaml")?;
    let de = serde_yaml::Deserializer::from_str(&input);
    let value = Value::deserialize(de)?;
    println!("{:?}", value);
    println!("{:?}", value["feeds"][0]);

    let mapping: Mapping;
    match value {
        serde_yaml::Value::Mapping(m) => {
            mapping = m;
        }
        _ => {
            return Err("Expected mapping YAML format".into());
        }
    }
    // let feeds: Vec<Feed> =
    let feedy_boy = FeedInputs::from(&mapping);
    let mut digest = String::new();

    for feed in feedy_boy.feeds {
        let feed = parse_feed(feed)?;
        // println!("{:#?}", parse_feed(feed.url)?);
        digest = build_digest(digest, feed);
    }
    write_digest(digest, String::from("test2.md"))?;
    Ok(())
}
