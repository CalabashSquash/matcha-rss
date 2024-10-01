use crate::rss::FeedOutput;
use std::fs;

// Mutates the current digest by appending a new digest
// QUESTION: is returning the string gucci here or nah? Is there a better way to do it?
pub fn build_rss_digest(digest: &mut String, feed_output: FeedOutput) {// -> String {
    let feed_digest = feed_output.feed_digest();
    digest.push_str(&feed_digest);
    // digest
}

pub fn write_digest(digest: String, filename: String) -> Result<(), Box<dyn std::error::Error>> {
    Ok(fs::write(filename, digest)?)
}
