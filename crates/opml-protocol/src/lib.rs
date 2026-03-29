use std::collections::HashMap;

use derive_builder::Builder;
use errors::OpmlError;
use serde::{Deserialize, Serialize, Serializer};

pub mod errors;

/// OPML outline types
const OUTLINE_TYPE_FOLDER: &str = "folder";
const OUTLINE_TYPE_RSS: &str = "rss";

/// OPML head element containing metadata
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Builder, Default)]
#[builder(default)]
#[serde(rename_all = "camelCase")]
pub struct Head {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expansion_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vert_scroll_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_right: Option<String>,
}

/// OPML body element containing outlines
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Builder, Default)]
#[builder(default)]
pub struct Body {
    #[serde(rename = "outline", default)]
    pub outlines: Vec<Outline>,
}

/// OPML outline element - the core hierarchical structure
/// Can represent folders, RSS feeds, or other content types
///
/// Note: `#[serde(flatten)]` on `extra` is retained for deserialization of
/// unknown XML attributes. For serialization, a custom `Serialize` impl
/// ensures `extra` entries are serialized as XML attributes (with `@` prefix)
/// rather than child elements, which is what `quick-xml` produces by default
/// for non-prefixed HashMap keys.
#[derive(Clone, Debug, Deserialize, PartialEq, Builder, Default)]
#[builder(default)]
pub struct Outline {
    #[serde(rename = "@text", default)]
    pub text: String,
    #[serde(rename = "outline", default, skip_serializing_if = "Option::is_none")]
    pub outlines: Option<Vec<Outline>>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(rename = "@xmlUrl", skip_serializing_if = "Option::is_none")]
    pub xml_url: Option<String>,
    #[serde(rename = "@description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "@htmlUrl", skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(rename = "@title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Whether the outline is commented out ("true"/"false"). By convention,
    /// if an outline is commented, all subordinate outlines are also commented.
    #[serde(rename = "@isComment", skip_serializing_if = "Option::is_none")]
    pub is_comment: Option<String>,
    /// Whether a breakpoint is set on this outline ("true"/"false").
    /// Mainly used for outlines that edit scripts.
    #[serde(rename = "@isBreakpoint", skip_serializing_if = "Option::is_none")]
    pub is_breakpoint: Option<String>,
    /// Date-time when the outline node was created.
    #[serde(rename = "@created", skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Comma-separated slash-delimited category strings (RSS 2.0 category format).
    /// Example: "/Boston/Weather" or "/Harvard/Berkman,/Politics"
    #[serde(rename = "@category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Extra unknown attributes captured during deserialization.
    /// Skipped during serialization to avoid `quick-xml` flattening them as
    /// child elements instead of XML attributes.
    #[serde(flatten, skip_serializing)]
    pub extra: HashMap<String, String>,
}

impl Serialize for Outline {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        // Count fields that will actually be serialized
        let mut field_count = 1; // @text is always present
        if self.outlines.is_some() {
            field_count += 1;
        }
        if self.type_.is_some() {
            field_count += 1;
        }
        if self.xml_url.is_some() {
            field_count += 1;
        }
        if self.description.is_some() {
            field_count += 1;
        }
        if self.html_url.is_some() {
            field_count += 1;
        }
        if self.title.is_some() {
            field_count += 1;
        }
        if self.version.is_some() {
            field_count += 1;
        }
        if self.language.is_some() {
            field_count += 1;
        }
        if self.is_comment.is_some() {
            field_count += 1;
        }
        if self.is_breakpoint.is_some() {
            field_count += 1;
        }
        if self.created.is_some() {
            field_count += 1;
        }
        if self.category.is_some() {
            field_count += 1;
        }
        // extra is intentionally skipped in serialization

        let mut state = serializer.serialize_struct("Outline", field_count)?;

        state.serialize_field("@text", &self.text)?;

        if let Some(ref outlines) = self.outlines {
            state.serialize_field("outline", outlines)?;
        }
        if let Some(ref type_) = self.type_ {
            state.serialize_field("@type", type_)?;
        }
        if let Some(ref xml_url) = self.xml_url {
            state.serialize_field("@xmlUrl", xml_url)?;
        }
        if let Some(ref description) = self.description {
            state.serialize_field("@description", description)?;
        }
        if let Some(ref html_url) = self.html_url {
            state.serialize_field("@htmlUrl", html_url)?;
        }
        if let Some(ref title) = self.title {
            state.serialize_field("@title", title)?;
        }
        if let Some(ref version) = self.version {
            state.serialize_field("@version", version)?;
        }
        if let Some(ref language) = self.language {
            state.serialize_field("@language", language)?;
        }
        if let Some(ref is_comment) = self.is_comment {
            state.serialize_field("@isComment", is_comment)?;
        }
        if let Some(ref is_breakpoint) = self.is_breakpoint {
            state.serialize_field("@isBreakpoint", is_breakpoint)?;
        }
        if let Some(ref created) = self.created {
            state.serialize_field("@created", created)?;
        }
        if let Some(ref category) = self.category {
            state.serialize_field("@category", category)?;
        }

        state.end()
    }
}

/// Root OPML document structure
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Builder)]
#[builder(default)]
#[serde(rename = "opml")]
pub struct Opml {
    #[serde(rename = "@version")]
    pub version: String,
    pub head: Head,
    pub body: Body,
}

impl Default for Opml {
    fn default() -> Self {
        Opml { version: "2.0".into(), head: Head::default(), body: Body::default() }
    }
}

/// Group of RSS feeds organized by folder
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GroupFeeds {
    /// Group name (empty string for top-level feeds)
    pub group: String,
    /// RSS feeds in this group
    pub feeds: Vec<OpmlFeed>,
}

/// RSS feed extracted from OPML outline
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OpmlFeed {
    pub text: Option<String>,
    pub title: Option<String>,
    pub xml_url: String,
    pub html_url: Option<String>,
}

impl std::str::FromStr for Opml {
    type Err = OpmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        quick_xml::de::from_str(s).map_err(OpmlError::from)
    }
}

impl Opml {
    /// Convert OPML to XML string
    pub fn to_string(&self) -> Result<String, OpmlError> {
        quick_xml::se::to_string(self).map_err(OpmlError::from)
    }

    /// Extract RSS feeds grouped by folders
    /// Supports unlimited nesting depth using recursive traversal
    pub fn get_group_feeds(&self) -> Result<Vec<GroupFeeds>, OpmlError> {
        let mut group_feeds = Vec::new();
        self.collect_feeds_recursive(&self.body.outlines, "", &mut group_feeds)?;
        Ok(group_feeds)
    }

    /// Recursively collect feeds from outline tree
    fn collect_feeds_recursive(
        &self,
        outlines: &[Outline],
        parent_path: &str,
        group_feeds: &mut Vec<GroupFeeds>,
    ) -> Result<(), OpmlError> {
        let mut current_feeds = Vec::new();
        let mut child_groups = Vec::new();

        // Process outlines in order, separating feeds from folders
        for outline in outlines {
            match outline.type_.as_deref() {
                Some(OUTLINE_TYPE_FOLDER) => {
                    if outline.text.is_empty() {
                        // Empty-text folder: process children at the same path level
                        // to avoid producing double slashes like "Parent//Child"
                        if let Some(child_outlines) = &outline.outlines {
                            self.collect_feeds_recursive(child_outlines, parent_path, group_feeds)?;
                        }
                    } else {
                        let current_path = if parent_path.is_empty() {
                            outline.text.clone()
                        } else {
                            format!("{}/{}", parent_path, outline.text)
                        };

                        if let Some(child_outlines) = &outline.outlines {
                            child_groups.push((current_path, child_outlines));
                        }
                    }
                }
                Some(OUTLINE_TYPE_RSS) => {
                    let xml_url = outline.xml_url.as_ref().ok_or_else(|| {
                        OpmlError::BadFeed("RSS outline missing xmlUrl".to_string())
                    })?;
                    current_feeds.push(OpmlFeed {
                        text: Some(outline.text.clone()),
                        title: outline.title.clone(),
                        xml_url: xml_url.clone(),
                        html_url: outline.html_url.clone(),
                    });
                }
                _ => {
                    // Handle outlines without explicit type that have xmlUrl (implicit RSS)
                    if let Some(xml_url) = &outline.xml_url {
                        current_feeds.push(OpmlFeed {
                            text: Some(outline.text.clone()),
                            title: outline.title.clone(),
                            xml_url: xml_url.clone(),
                            html_url: outline.html_url.clone(),
                        });
                    }
                }
            }
        }

        // Add feeds at this level (top-level feeds) BEFORE processing folders
        if !current_feeds.is_empty() {
            group_feeds.push(GroupFeeds { group: parent_path.to_string(), feeds: current_feeds });
        }

        // Second pass: process folders in order AFTER adding current level feeds
        for (path, children) in child_groups {
            self.collect_feeds_recursive(children, &path, group_feeds)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[test]
    fn basic() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head>
                    <title>My Subscription List</title>
                </head>
                <body>
                    <!-- top level group -->
                    <outline text="Technology" type="folder">
                        <outline text="TechCrunch" type="rss" xmlUrl="https://techcrunch.com/feed/" />
                        <outline text="The Verge" type="rss" xmlUrl="https://www.theverge.com/rss/index.xml" />
                    </outline>

                    <!-- top level feed -->
                    <outline text="Hacker News" type="rss" xmlUrl="https://hnrss.org/frontpage" />

                    <outline text="My Bookmarks" type="folder">
                        <outline text="Google" type="rss" xmlUrl="https://techcrunch.com/feed/" />
                        <outline text="Stack Overflow" type="rss" xmlUrl="https://techcrunch.com/feed/" />
                    </outline>
                </body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        assert_eq!(opml.version, "2.0");
        assert_eq!(opml.head.title, Some("My Subscription List".to_string()));
        assert_eq!(opml.body.outlines.len(), 3);
        let group_feeds = opml.get_group_feeds().unwrap();
        assert_eq!(group_feeds.len(), 3);
        // Root-level feeds come first (no folder), then folder groups in document order
        assert_eq!(group_feeds[0].group, "");
        assert_eq!(group_feeds[0].feeds.len(), 1);
        assert_eq!(group_feeds[0].feeds[0].text, Some("Hacker News".to_string()));
        assert_eq!(group_feeds[1].group, "Technology");
        assert_eq!(group_feeds[1].feeds.len(), 2);
        assert_eq!(group_feeds[2].group, "My Bookmarks");
        assert_eq!(group_feeds[2].feeds.len(), 2);
    }

    #[test]
    fn test_missing_xml_url() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <outline text="Invalid RSS" type="rss" />
                </body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        let result = opml.get_group_feeds();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_opml() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head></head>
                <body></body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        let group_feeds = opml.get_group_feeds().unwrap();
        assert_eq!(group_feeds.len(), 0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let opml = super::Opml {
            version: "2.0".to_string(),
            head: super::Head { title: Some("Test Title".to_string()), ..Default::default() },
            body: super::Body {
                outlines: vec![super::Outline {
                    text: "Test Feed".to_string(),
                    type_: Some("rss".to_string()),
                    xml_url: Some("https://example.com/feed.xml".to_string()),
                    ..Default::default()
                }],
            },
        };

        let xml_string = opml.to_string().unwrap();
        let parsed_opml = super::Opml::from_str(&xml_string).unwrap();
        assert_eq!(opml.version, parsed_opml.version);
        assert_eq!(opml.head.title, parsed_opml.head.title);
        assert_eq!(opml.body.outlines.len(), parsed_opml.body.outlines.len());
        assert_eq!(opml.body.outlines[0].text, parsed_opml.body.outlines[0].text);
        assert_eq!(opml.body.outlines[0].type_, parsed_opml.body.outlines[0].type_);
        assert_eq!(opml.body.outlines[0].xml_url, parsed_opml.body.outlines[0].xml_url);
    }

    #[test]
    fn test_serialization_outputs_attributes() {
        let opml = super::Opml {
            version: "2.0".to_string(),
            head: super::Head::default(),
            body: super::Body {
                outlines: vec![super::Outline {
                    text: "Feed".to_string(),
                    type_: Some("rss".to_string()),
                    xml_url: Some("https://example.com/feed.xml".to_string()),
                    html_url: Some("https://example.com".to_string()),
                    ..Default::default()
                }],
            },
        };

        let xml = opml.to_string().unwrap();
        // Verify attributes appear as XML attributes, not child elements
        assert!(xml.contains("text=\"Feed\""), "text should be an attribute");
        assert!(xml.contains("type=\"rss\""), "type should be an attribute");
        assert!(
            xml.contains("xmlUrl=\"https://example.com/feed.xml\""),
            "xmlUrl should be an attribute"
        );
        assert!(xml.contains("htmlUrl=\"https://example.com\""), "htmlUrl should be an attribute");
        // Verify no child elements for attribute fields
        assert!(!xml.contains("<text>"), "text should not be a child element");
        assert!(!xml.contains("<type>"), "type should not be a child element");
    }

    #[test]
    fn test_nested_folders() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head>
                    <title>Nested Folders Test</title>
                </head>
                <body>
                    <outline text="Tech" type="folder">
                        <outline text="News" type="folder">
                            <outline text="TechCrunch" type="rss" xmlUrl="https://techcrunch.com/feed/" />
                            <outline text="The Verge" type="rss" xmlUrl="https://www.theverge.com/rss/index.xml" />
                        </outline>
                        <outline text="Blogs" type="folder">
                            <outline text="Personal Tech" type="folder">
                                <outline text="John's Blog" type="rss" xmlUrl="https://example.com/john.xml" />
                            </outline>
                        </outline>
                    </outline>
                    <outline text="Direct Feed" type="rss" xmlUrl="https://example.com/direct.xml" />
                </body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        let group_feeds = opml.get_group_feeds().unwrap();

        // Should have 3 groups: root level (empty), Tech/News, Tech/Blogs/Personal Tech
        assert_eq!(group_feeds.len(), 3);

        // Root-level feeds first
        let root_feeds = group_feeds.iter().find(|g| g.group.is_empty()).unwrap();
        assert_eq!(root_feeds.feeds.len(), 1);
        assert_eq!(root_feeds.feeds[0].text, Some("Direct Feed".to_string()));

        // Nested folder groups with full path
        let tech_news = group_feeds.iter().find(|g| g.group == "Tech/News").unwrap();
        assert_eq!(tech_news.feeds.len(), 2);
        assert_eq!(tech_news.feeds[0].text, Some("TechCrunch".to_string()));

        let tech_blogs_personal =
            group_feeds.iter().find(|g| g.group == "Tech/Blogs/Personal Tech").unwrap();
        assert_eq!(tech_blogs_personal.feeds.len(), 1);
        assert_eq!(tech_blogs_personal.feeds[0].text, Some("John's Blog".to_string()));
    }

    #[test]
    fn test_deeply_nested_folders() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head><title>Deep Nesting</title></head>
                <body>
                    <outline text="Level1" type="folder">
                        <outline text="Level2" type="folder">
                            <outline text="Level3" type="folder">
                                <outline text="Level4" type="folder">
                                    <outline text="Deep Feed" type="rss" xmlUrl="https://example.com/deep.xml" />
                                </outline>
                            </outline>
                        </outline>
                    </outline>
                </body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        let group_feeds = opml.get_group_feeds().unwrap();

        assert_eq!(group_feeds.len(), 1);
        assert_eq!(group_feeds[0].group, "Level1/Level2/Level3/Level4");
        assert_eq!(group_feeds[0].feeds.len(), 1);
        assert_eq!(group_feeds[0].feeds[0].text, Some("Deep Feed".to_string()));
    }

    #[test]
    fn test_empty_text_no_double_slash() {
        let xml = r#"
            <?xml version="1.0"?>
            <opml version="2.0">
                <head><title>Test</title></head>
                <body>
                    <outline text="Parent" type="folder">
                        <outline text="" type="folder">
                            <outline text="Child" type="rss" xmlUrl="https://example.com/feed.xml" />
                        </outline>
                    </outline>
                </body>
            </opml>
        "#;

        let opml = super::Opml::from_str(xml).unwrap();
        let group_feeds = opml.get_group_feeds().unwrap();

        // Empty-text folder is transparent: children are processed at the
        // parent path level, producing "Parent" (NOT "Parent//Child")
        assert_eq!(group_feeds.len(), 1);
        assert_eq!(group_feeds[0].group, "Parent");
        assert_eq!(group_feeds[0].feeds[0].text, Some("Child".to_string()));
    }
}
