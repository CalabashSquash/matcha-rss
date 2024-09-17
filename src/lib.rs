pub use rss::parse_feed;

pub mod rss {
    use core::str;
    use std::fs;
    use reqwest;
    use quick_xml::{events::Event, Reader};
    use std::io::Read;

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
        pub fn write_to_md(self: Self) -> Result<(), Box<dyn std::error::Error>> {
            let mut contents = format!("# {}\n", self.feed_name);
            for item in self.items {
                contents.push_str(&format!("- [{}]({})\n", item.title, item.url));
            }

            let contents = contents.as_bytes();
            fs::write("test.md", contents)?;
            Ok(())
        }

    }

    pub fn parse_title(mut reader: Reader<&[u8]>) -> (Reader<&[u8]>, String) {
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
                        println!("attributes values: {:?}",
                                    e.attributes().map(|a| a.unwrap().value)
                                    .collect::<Vec<_>>());
                        let res = parse_url(reader);
                        reader = res.0;
                        url = res.1;
                    }
                    other => {
                        println!("start: {}", str::from_utf8(other).unwrap());
                    },
                },
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"link" => {
                        match e.attributes().find(|x| x.as_ref().unwrap().key.eq(&quick_xml::name::QName(b"href"))) {
                            Some(r) => match r {
                                Ok(a) => {
                                    url = String::from_utf8(a.value.into_owned()).unwrap();
                                    println!("URL SET====");
                                },
                                Err(e) => panic!("Error finding href")
                            }
                            None => ()
                        }
                    }
                    _ => ()
                }
                Ok(Event::Text(text)) => {
                    println!("text: {:?}", text);
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"item" | b"entry" => break,
                    _ => (),
                },
                Ok(e) => {
                    println!("other {:?}", e);
                },
            }
        }

        (reader, Item { url, title })
    }

    // TODO state machine in ascii

    pub fn parse_feed(url: String) -> Result<FeedOutput, Box<dyn std::error::Error>> {
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
                            println!("PARSING ENTRY: {:?}", e.name().as_ref());
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
}