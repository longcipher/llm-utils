use std::{collections::HashSet, str::FromStr};

use linkify::{LinkFinder, LinkKind};
use url::Url;

pub fn extract_urls<T: AsRef<str>>(input: T) -> Vec<Url> {
    let mut unique_strs = HashSet::new();

    LinkFinder::new()
        .kinds(&[LinkKind::Url])
        .links(input.as_ref())
        // Deduplicate on the cheap string representation first
        .filter(|link| unique_strs.insert(link.as_str().to_string()))
        // Only parse the expensive Url for unique entries
        .filter_map(|link| Url::from_str(link.as_str()).ok())
        .collect()
}
