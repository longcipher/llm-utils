# opml-protocol

A robust Rust library for parsing and generating OPML 2.0 (Outline Processor Markup Language) files.

## Features

- **OPML 2.0 Support**: Full implementation of the OPML 2.0 specification.
- **Serde Integration**: Seamless serialization and deserialization using `serde` and `quick-xml`.
- **Builder Pattern**: Ergonomic construction of OPML documents using `derive_builder`.
- **RSS Feed Extraction**: Helper method to extract RSS feeds grouped by folders.
- **Extensible**: Preserves unknown attributes via `HashMap` flattening.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
opml-protocol = "0.1.0"
```

## Usage

### Parsing OPML

```rust
use opml_protocol::Opml;
use std::str::FromStr;

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

let opml = Opml::from_str(xml).unwrap();
println!("Title: {:?}", opml.head.title);
```

### Creating OPML

```rust
use opml_protocol::{Opml, OutlineBuilder, Body, Head};

let outline = OutlineBuilder::default()
    .text("Hacker News".to_string())
    .type_(Some("rss".to_string()))
    .xml_url(Some("https://news.ycombinator.com/rss".to_string()))
    .build()
    .unwrap();

let opml = Opml {
    version: "2.0".to_string(),
    head: Head {
        title: Some("My Feeds".to_string()),
        ..Default::default()
    },
    body: Body {
        outlines: vec![outline],
    },
};

let xml = opml.to_string().unwrap();
```
