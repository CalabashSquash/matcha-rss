# matcha-rss
RSS reader/daily digest generator written in Rust. Inspired by https://github.com/piqoni/matcha

# TODOs
- state machine explanation of RSS parsing in ascii
    - Potential inspiration: https://news.ycombinator.com/item?id=31891226
    - https://blog.regehr.org/archives/1653
- verbose option which prints description or something.
- image for feeds
- weather
- chatgpt summary
- Don't show previous items
- Specify filename and use date
- See if [feed-rs](https://github.com/feed-rs/feed-rs) can be used to parse RSS instead of [rss](https://crates.io/crates/rss) (which seems to require all feeds start with the `<rss>` feed.)
- [Podcasts](https://podcastindex.org/namespace/1.0)