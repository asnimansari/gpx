use serde::Deserialize;

use super::{Email, Link};

/// A person or organization (`personType` in GPX 1.1).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct Person {
    /// Name of the person or organization.
    #[serde(default)]
    pub name: Option<String>,
    /// Contact email address.
    #[serde(default)]
    pub email: Option<Email>,
    /// Link to a web page with more information.
    #[serde(default)]
    pub link: Option<Link>,
}
