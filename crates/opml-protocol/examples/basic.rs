use std::str::FromStr;

use opml_protocol::{Opml, OutlineBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parsing OPML from string
    let xml = r#"
        <?xml version="1.0"?>
        <opml version="2.0">
            <head>
                <title>My Feeds</title>
            </head>
            <body>
                <outline text="Tech" type="folder">
                    <outline text="Rust Blog" type="rss" xmlUrl="https://blog.rust-lang.org/feed.xml" />
                </outline>
            </body>
        </opml>
    "#;

    let opml = Opml::from_str(xml)?;
    println!("Parsed OPML version: {}", opml.version);
    if let Some(title) = &opml.head.title {
        println!("Title: {}", title);
    }

    // 2. Building OPML programmatically
    let outline = OutlineBuilder::default()
        .text("Hacker News".to_string())
        .type_(Some("rss".to_string()))
        .xml_url(Some("https://news.ycombinator.com/rss".to_string()))
        .build()?;

    let opml_new = Opml {
        version: "2.0".to_string(),
        head: Default::default(),
        body: opml_protocol::Body { outlines: vec![outline] },
    };

    let xml_out = opml_new.to_string()?;
    println!("\nGenerated XML:\n{}", xml_out);

    Ok(())
}
