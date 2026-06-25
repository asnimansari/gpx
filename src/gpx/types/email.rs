/// An email address split into id and domain (`emailType` in GPX 1.1).
///
/// The full address is `id@domain`. Splitting the parts helps reduce email harvesting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
    /// Local part of the email address (before `@`).
    pub id: String,
    /// Domain part of the email address (after `@`).
    pub domain: String,
}
