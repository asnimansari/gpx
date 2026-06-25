use std::fmt::{self, Display};

use serde::Deserialize;

/// An email address split into id and domain (`emailType` in GPX 1.1).
///
/// The full address is `id@domain`. Splitting the parts helps reduce email harvesting.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Email {
    /// Local part of the email address (before `@`).
    #[serde(rename = "@id")]
    pub id: String,
    /// Domain part of the email address (after `@`).
    #[serde(rename = "@domain")]
    pub domain: String,
}

impl Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.id, self.domain)
    }
}