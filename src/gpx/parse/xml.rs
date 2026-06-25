use chrono::{DateTime, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;

use crate::gpx::error::ParseError;

pub fn local_name(name: QName<'_>) -> &[u8] {
    name.local_name().into_inner()
}

pub fn is_local_name(name: QName<'_>, expected: &[u8]) -> bool {
    local_name(name) == expected
}

pub fn read_element_text(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String, ParseError> {
    let mut text = String::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => text.push_str(&e.unescape()?),
            Ok(Event::CData(e)) => text.push_str(&String::from_utf8_lossy(&e)),
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(text)
}

pub fn skip_element(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<(), ParseError> {
    let name = start.name().into_inner().to_vec();
    let mut depth = 1;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == name.as_slice() => depth += 1,
            Ok(Event::End(e)) if e.name().as_ref() == name.as_slice() => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(())
}

pub fn capture_inner_xml(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<String, ParseError> {
    let mut out = String::new();
    let name = start.name().into_inner().to_vec();
    let mut depth = 1;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == name.as_slice() {
                    depth += 1;
                }
                out.push('<');
                write_element_start(&mut out, &e)?;
                out.push('>');
            }
            Ok(Event::Empty(e)) => {
                out.push('<');
                write_element_start(&mut out, &e)?;
                out.push_str("/>");
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == name.as_slice() {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                out.push_str("</");
                out.push_str(&String::from_utf8_lossy(e.name().as_ref()));
                out.push('>');
            }
            Ok(Event::Text(e)) => out.push_str(&e.unescape()?),
            Ok(Event::CData(e)) => {
                out.push_str("<![CDATA[");
                out.push_str(&String::from_utf8_lossy(&e));
                out.push_str("]]>");
            }
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(out)
}

fn write_element_start(out: &mut String, e: &BytesStart<'_>) -> Result<(), ParseError> {
    out.push_str(&String::from_utf8_lossy(e.name().as_ref()));
    for attr in e.attributes().with_checks(false) {
        let attr = attr?;
        out.push(' ');
        out.push_str(&String::from_utf8_lossy(attr.key.as_ref()));
        out.push('=');
        out.push('"');
        out.push_str(&String::from_utf8_lossy(&attr.value));
        out.push('"');
    }
    Ok(())
}

pub fn attr_value(start: &BytesStart<'_>, attribute: &str) -> Result<Option<String>, ParseError> {
    for attr in start.attributes().with_checks(false) {
        let attr = attr?;
        if local_name(attr.key) == attribute.as_bytes() {
            return Ok(Some(String::from_utf8_lossy(&attr.value).into_owned()));
        }
    }

    Ok(None)
}

pub fn attr_string(
    start: &BytesStart<'_>,
    element: &'static str,
    attribute: &'static str,
) -> Result<String, ParseError> {
    attr_value(start, attribute)?
        .ok_or(ParseError::MissingAttribute { element, attribute })
}

pub fn attr_f64(
    start: &BytesStart<'_>,
    element: &'static str,
    attribute: &'static str,
) -> Result<f64, ParseError> {
    let value = attr_string(start, element, attribute)?;
    value.parse().map_err(|_| ParseError::InvalidAttribute {
        element,
        attribute,
        value,
    })
}

pub fn parse_decimal(element: &'static str, field: &'static str, value: &str) -> Result<f64, ParseError> {
    value.parse().map_err(|_| ParseError::InvalidAttribute {
        element,
        attribute: field,
        value: value.to_owned(),
    })
}

pub fn parse_u64(element: &'static str, field: &'static str, value: &str) -> Result<u64, ParseError> {
    value.parse().map_err(|_| ParseError::InvalidAttribute {
        element,
        attribute: field,
        value: value.to_owned(),
    })
}

pub fn parse_u16(element: &'static str, field: &'static str, value: &str) -> Result<u16, ParseError> {
    value.parse().map_err(|_| ParseError::InvalidAttribute {
        element,
        attribute: field,
        value: value.to_owned(),
    })
}

pub fn parse_time(value: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| ParseError::InvalidTime(value.to_owned()))
}
