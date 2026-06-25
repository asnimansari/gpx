use chrono::{DateTime, Utc};

use super::{Bounds, Copyright, Extensions, Link, Person};

/// GPX file metadata (`metadataType` in GPX 1.1).
///
/// Describes the file, its author, copyright, keywords, and geographic bounds.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Metadata {
    /// Name of the GPX file.
    pub name: Option<String>,
    /// Description of the GPX file contents.
    pub desc: Option<String>,
    /// Person or organization responsible for the file.
    pub author: Option<Person>,
    /// Copyright and licensing information.
    pub copyright: Option<Copyright>,
    /// Links to external resources related to the file.
    pub links: Vec<Link>,
    /// Creation or last-modification time of the file (ISO 8601).
    pub time: Option<DateTime<Utc>>,
    /// Keywords associated with the file for searching.
    pub keywords: Option<String>,
    /// Minimum and maximum latitude/longitude covered by the file.
    pub bounds: Option<Bounds>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
}
