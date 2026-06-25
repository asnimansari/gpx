/// Extension elements from namespaces other than the GPX schema (`extensionsType`).
///
/// GPX allows arbitrary elements from other XML namespaces to extend the format.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extensions {
    /// Raw inner XML contained within the `<extensions>` element.
    pub inner_xml: String,
}
