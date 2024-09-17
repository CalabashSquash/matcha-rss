use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::io::Read;
// use crate::rsss;

const MAX_COUNT: i32 = 2;

#[derive(Debug)]
struct Item {
    url: String,
    title: String,
}
#[derive(Debug)]
struct FeedOutput {
    items: Vec<Item>,
    feed_name: String,
}

impl FeedOutput {
    fn write_to_md(self: Self) -> Result<(), Box<dyn std::error::Error>> {
        let mut contents = format!("# {}\n", self.feed_name);
        for item in self.items {
            contents.push_str(&format!("- [{}]({})\n", item.title, item.url));
        }

        let contents = contents.as_bytes();
        fs::write("test.md", contents)?;
        Ok(())
    }

}

fn parse_title(mut reader: Reader<&[u8]>) -> (Reader<&[u8]>, String) {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
            Ok(Event::Text(t)) => {
                return (reader, t.unescape().unwrap().to_string()); // TODO This can panic
            }
            Ok(Event::CData(cdata)) => {
                let text = cdata.escape().unwrap();
                return (reader, text.unescape().unwrap().to_string());
            }
            _ => panic!("No text in title"),
        }
    }
}

// TODO doesn't work if the link is of format <link href="..." />.
// Only works if it's like <link>...</link>
fn parse_url(mut reader: Reader<&[u8]>) -> (Reader<&[u8]>, String) {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
            Ok(Event::Text(t)) => {
                return (reader, t.unescape().unwrap().to_string()); // TODO This can panic
            }
            _ => {
                panic!("No text in URL")
            }
        }
    }
}

fn parse_item<'a>(mut reader: Reader<&'a [u8]>) -> (Reader<&'a [u8]>, Item) {
    let mut buf = Vec::new();
    let mut title = String::default();
    let mut url = String::default();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
            // errors the loop when reaching end of file
            Ok(Event::Eof) => panic!("Error"), // TODO don't panic but instead fail on just this feed.
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"title" => {
                    let res = parse_title(reader);
                    reader = res.0;
                    title = res.1;
                }
                b"link" => {
                    let res = parse_url(reader);
                    reader = res.0;
                    url = res.1;
                }
                _ => (),
            },
            Ok(Event::Text(_)) => {}
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"item" => break,
                _ => (),
            },
            _ => (),
        }
    }

    (reader, Item { url, title })
}

// TODO state machine in ascii

fn parse_feed(url: String) -> Result<FeedOutput, Box<dyn std::error::Error>> {
    let buf = &mut Default::default();
    let content = reqwest::blocking::get(url)?.read_to_string(buf)?;
    let mut reader = Reader::from_str(buf);
    reader.trim_text(true);
    println!("content: {:?}", content);
    // println!("buf: {:?}", buf);

    let mut count = 0;
    let mut txt = Vec::new();
    let mut buf = Vec::new();
    let mut items: Vec<Item> = Vec::new();
    let mut feed_name: String = "".into();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"item" | b"entry" => {
                        let res = parse_item(reader);
                        reader = res.0;
                        items.push(res.1);
                    }
                    // b"item" => println!("attributes values: {:?}",
                    //                     e.attributes().map(|a| a.unwrap().value)
                    //                     .collect::<Vec<_>>()),
                    b"tag2" => count += 1,
                    b"title" => {
                        let res = parse_title(reader);
                        reader = res.0;
                        feed_name = res.1;
                    }
                    _ => println!("YEET: {:?}", e.name()),
                }
            }
            Ok(Event::Text(e)) => {
                println!("text! {:?}", e.unescape());
                txt.push(e.unescape().unwrap().into_owned());
                // println!("TXT: {:?}", txt);
            }

            // There are several other `Event`s we do not consider here
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        // buf.clear();
    }
    Ok(FeedOutput { items, feed_name })
}

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
