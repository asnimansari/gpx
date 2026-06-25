use serde::Deserialize;

/// Extension elements from namespaces other than the GPX schema (`extensionsType`).
///
/// GPX allows arbitrary elements from other XML namespaces to extend the format.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct Extensions {
    /// Raw inner XML contained within the `<extensions>` element.
    #[serde(default, rename = "$value")]
    pub inner_xml: String,
}
