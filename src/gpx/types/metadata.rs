use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{Bounds, Copyright, Extensions, Link, Person};

/// GPX file metadata (`metadataType` in GPX 1.1).
///
/// Describes the file, its author, copyright, keywords, and geographic bounds.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Metadata {
    /// Name of the GPX file.
    #[serde(default)]
    pub name: Option<String>,
    /// Description of the GPX file contents.
    #[serde(default)]
    pub desc: Option<String>,
    /// Person or organization responsible for the file.
    #[serde(default)]
    pub author: Option<Person>,
    /// Copyright and licensing information.
    #[serde(default)]
    pub copyright: Option<Copyright>,
    /// Links to external resources related to the file.
    #[serde(rename = "link", default)]
    pub links: Vec<Link>,
    /// Creation or last-modification time of the file (ISO 8601).
    #[serde(default)]
    pub time: Option<DateTime<Utc>>,
    /// Keywords associated with the file for searching.
    #[serde(default)]
    pub keywords: Option<String>,
    /// Minimum and maximum latitude/longitude covered by the file.
    #[serde(default)]
    pub bounds: Option<Bounds>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
}
