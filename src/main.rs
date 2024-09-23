use matcha_rss::{
    digest::{build_digest, write_digest},
    rss::parse_feed,
    yaml::FeedInputs,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = fs::read_to_string("test/config.yaml")?;
    let feedy_boy: FeedInputs = serde_yaml::from_str(&input)?;
    println!("{:?}", feedy_boy);

    let mut digest = String::new();

    for feed in feedy_boy.feeds {
        let feed = parse_feed(feed)?;
        // println!("{:#?}", parse_feed(feed.url)?);
        digest = build_digest(digest, feed);
    }
    write_digest(digest, String::from("test2.md"))?;
    println!("Done");
    Ok(())
}
