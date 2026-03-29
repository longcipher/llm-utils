use quick_xml::{DeError, SeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpmlError {
    #[error("Failed to parse OPML: {0}")]
    BadFeed(String),
    #[error("Failed to parse XML: {0}")]
    XmlDeError(#[from] DeError),
    #[error("Failed to serialize XML: {0}")]
    XmlSerError(#[from] SeError),
}
