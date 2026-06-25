use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    Xml(quick_xml::Error),
    MissingAttribute { element: &'static str, attribute: &'static str },
    InvalidAttribute { element: &'static str, attribute: &'static str, value: String },
    InvalidTime(String),
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(err) => write!(f, "XML error: {err}"),
            Self::MissingAttribute { element, attribute } => {
                write!(f, "missing attribute `{attribute}` on `<{element}>`")
            }
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                f,
                "invalid attribute `{attribute}` on `<{element}>`: `{value}`"
            ),
            Self::InvalidTime(value) => write!(f, "invalid GPX time: `{value}`"),
            Self::UnexpectedEof => write!(f, "unexpected end of GPX document"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(err) => Some(err),
            _ => None,
        }
    }
}

impl From<quick_xml::Error> for ParseError {
    fn from(err: quick_xml::Error) -> Self {
        Self::Xml(err)
    }
}

impl From<quick_xml::events::attributes::AttrError> for ParseError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Self::Xml(err.into())
    }
}
