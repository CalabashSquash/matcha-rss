use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::fs;
use matcha_rss::parse_feed; // QUESTION: How does it get matcha_rss? This doesn't appear anywhere else in the repo

const MAX_COUNT: i32 = 2;

#[derive(Debug)]
struct FeedInput {
    url: String,
    list_length: i32,
}

#[derive(Debug)]
struct FeedInputs {
    feeds: Vec<FeedInput>,
}

impl FeedInput {
    fn from(url: String, list_length: i32) -> Self {
        FeedInput {
            url: url,
            list_length: list_length,
        }
    }
}

impl From<&Mapping> for FeedInputs {
    fn from(value: &Mapping) -> FeedInputs {
        // let ding = value["feeds"]; <--- QUESTION: why can't you move?
        let ding = &value["feeds"];

        let urls = match ding {
            serde_yaml::Value::Sequence(s) => s,
            _ => {
                println!("feed list invalid");
                &vec![]
            }
        };

        let mut feeds: Vec<FeedInput> = vec![];
        for url in urls {
            let url_and_length = match url {
                serde_yaml::Value::String(s) => s.split(" ").collect::<Vec<&str>>(),
                _ => {
                    continue;
                }
            };

            if url_and_length.len() != 2 {
                println!("feed {:?} invalid", url);
                continue;
            }
            let feed_length: i32;
            match str::parse(url_and_length[1]) {
                Ok(l) => {
                    feed_length = l;
                }
                Err(e) => {
                    print!("Invalid feed length: {}. Error: {}", url_and_length[1], e);
                    continue;
                }
            }

            feeds.push(FeedInput {
                url: url_and_length[0].to_string(),
                list_length: feed_length,
            });
        }

        // println!("{:?}", ding);
        println!("{:?}", feeds);
        FeedInputs { feeds: feeds }
    }
}

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
    for feed in feedy_boy.feeds {
        let output = parse_feed(feed.url)?;
        // println!("{:#?}", parse_feed(feed.url)?);
        output.write_to_md()?;
        
    }


    Ok(())
}
