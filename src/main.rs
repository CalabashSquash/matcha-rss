use serde::{Deserialize};
use serde_yaml::{Error, Mapping, Sequence, Value};
use std::{fs};
use reqwest;
use rss::Channel;
use crate::rsss;

fn example_feed(url: String) -> Result<Channel, Box<dyn std::error::Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    println!("content: {:?}", content);
    let channel = Channel::read_from(&content[..])?;
    let channel2: Channel == rss::
    Ok(channel)
}

#[derive(Debug)]
struct Feed{
    url: String,
    list_length: i32
}

#[derive(Debug)]
struct Feeds {
    feeds: Vec<Feed>
}

impl Feed {
    fn from(url: String, list_length: i32) -> Self {
        Feed { url: url, list_length: list_length }
    }
}

impl From<&Mapping> for Feeds {
    fn from(value: &Mapping) -> Feeds {
        // let ding = value["feeds"]; <--- QUESTION: why can't you move?
        let ding = &value["feeds"];
        
        let urls = match ding {
            serde_yaml::Value::Sequence(s) => s,
            _ => {
                println!("feed list invalid");
                &vec![]
            }
        };

        let mut feeds: Vec<Feed> = vec![];
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
                },
                Err(e) => {
                    print!("Invalid feed length: {}. Error: {}", url_and_length[1], e);
                    continue;
                }
            }

            feeds.push(Feed {url: url_and_length[0].to_string(), list_length: feed_length});
        }

        // println!("{:?}", ding);
        println!("{:?}", feeds);
        Feeds {feeds: feeds }
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
        },
        _ => {
            return Err("Expected mapping YAML format".into());
        }
    }
    // let feeds: Vec<Feed> = 
    let feedyBoy = Feeds::from(&mapping);
    for feed in feedyBoy.feeds {
        println!("{:?}", example_feed(feed.url)?);
    }

    Ok(())
}
