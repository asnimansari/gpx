use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    De(quick_xml::DeError),
    Io(std::io::Error),
    MissingAttribute { element: &'static str, attribute: &'static str },
    InvalidAttribute { element: &'static str, attribute: &'static str, value: String },
    InvalidTime(String),
    InvalidValue { element: &'static str, value: String },
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::De(err) => write!(f, "GPX deserialization error: {err}"),
            Self::Io(err) => write!(f, "GPX I/O error: {err}"),
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
            Self::InvalidValue { element, value } => {
                write!(f, "invalid value for `<{element}>`: `{value}`")
            }
            Self::UnexpectedEof => write!(f, "unexpected end of GPX document"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::De(err) => Some(err),
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<quick_xml::DeError> for ParseError {
    fn from(err: quick_xml::DeError) -> Self {
        Self::De(err)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
