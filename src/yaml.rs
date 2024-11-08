use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedInput {
    pub url: String,
    pub list_length: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedInputs {
    pub feeds: Vec<FeedInput>,
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

        FeedInputs { feeds: feeds }
    }
}
