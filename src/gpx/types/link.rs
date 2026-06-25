use serde::Deserialize;

/// A link to an external resource (`linkType` in GPX 1.1).
///
/// Points to a web page, photo, video, or other related content.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Link {
    /// URL of the linked resource.
    #[serde(rename = "@href")]
    pub href: String,
    /// Human-readable text describing the link.
    #[serde(default)]
    pub text: Option<String>,
    /// MIME type or category of the linked resource (GPX `<type>` element).
    #[serde(rename = "type", default)]
    pub link_type: Option<String>,
}
