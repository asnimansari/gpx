use super::{Email, Link};

/// A person or organization (`personType` in GPX 1.1).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Person {
    /// Name of the person or organization.
    pub name: Option<String>,
    /// Contact email address.
    pub email: Option<Email>,
    /// Link to a web page with more information.
    pub link: Option<Link>,
}
