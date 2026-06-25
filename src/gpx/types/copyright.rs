/// Copyright holder information (`copyrightType` in GPX 1.1).
///
/// Describes who owns the data and under what license it may be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copyright {
    /// Name of the copyright holder.
    pub author: String,
    /// Year of copyright (ISO 8601 `gYear`, e.g. `"2026"`).
    pub year: Option<String>,
    /// URI of the license governing use of the file.
    pub license: Option<String>,
}
