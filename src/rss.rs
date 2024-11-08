use core::str;
use quick_xml::{events::Event, Reader};
use reqwest;
use std::io::Read;

use crate::yaml::FeedInput;

const MAX_COUNT: i32 = 2;

#[derive(Debug)]
pub struct Item {
    url: String,
    title: String,
}
#[derive(Debug)]
pub struct FeedOutput {
    items: Vec<Item>,
    feed_name: String,
}

impl FeedOutput {
    pub fn feed_digest(self) -> String {
        let mut contents = format!("# {}\n", self.feed_name);
        for item in self.items {
            contents.push_str(&format!("- [{}]({})\n", item.title, item.url));
        }

        contents
    }
}

pub fn parse_title(reader: &mut Reader<&[u8]>) -> String {
    let mut buf = Vec::new();
    match reader.read_event_into(&mut buf) {
        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
        Ok(Event::Text(t)) => {
            return t.unescape().unwrap().to_string(); // TODO This can panic
        }
        Ok(Event::CData(cdata)) => {
            let text = cdata.escape().unwrap();
            return text.unescape().unwrap().to_string();
        }
        _ => panic!("No text in title"),
    }
}

// doesn't work if the link is of format <link href="..." />.
// Only works if it's like <link>...</link>
// The former case is instead caught by the `Event::Empty` variant (<foo/> is called an "empty" tag)
fn parse_url(reader: &mut Reader<&[u8]>) -> String {
    let mut buf = Vec::new();
    match reader.read_event_into(&mut buf) {
        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e), // TODO don't panic but instead fail on just this feed.
        Ok(Event::Text(t)) => {
            return t.unescape().unwrap().to_string(); // TODO This can panic
        }
        _ => {
            panic!("No text in URL")
        }
    }
}

fn parse_item<'a>(reader: &mut Reader<&'a [u8]>) -> Item {
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
                    // reader = res.0;
                    title = res;
                }
                b"link" => {
                    url = parse_url(reader);
                }
                _ => (),
            },
            // <link href=... />
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"link" => {
                    match e
                        .attributes()
                        .find(|x| x.as_ref().unwrap().key.eq(&quick_xml::name::QName(b"href")))
                    {
                        Some(r) => match r {
                            Ok(a) => {
                                url = String::from_utf8(a.value.into_owned()).unwrap();
                            }
                            Err(_) => panic!("Error finding href"),
                        },
                        None => (),
                    }
                }
                _ => (),
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"item" | b"entry" => break,
                _ => (),
            },
            Ok(_) => (),
        }
    }

    Item { url, title }
}

pub fn parse_feed(feed: FeedInput) -> Result<FeedOutput, Box<dyn std::error::Error>> {
    let buf = &mut Default::default();
    reqwest::blocking::get(feed.url)?.read_to_string(buf)?;
    let mut reader = Reader::from_str(buf);
    reader.trim_text(true);

    let mut item_count = 0;
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

            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"item" | b"entry" => {
                    let res = parse_item(&mut reader);
                    items.push(res);
                    item_count += 1;
                    if item_count >= feed.list_length {
                        break;
                    }
                }
                b"title" => {
                    let res = parse_title(&mut reader);
                    // reader = res.0;
                    feed_name = res;
                }
                _ => (),
            },
            // There are several other `Event`s we do not consider here
            _ => (),
        }
    }
    Ok(FeedOutput { items, feed_name })
}
