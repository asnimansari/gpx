use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::xml::{
    attr_f64, attr_string, capture_inner_xml, is_local_name, local_name, parse_decimal, parse_time,
    read_element_text, skip_element,
};
use crate::gpx::error::ParseError;
use crate::gpx::types::{
    Bounds, Copyright, Email, Extensions, Fix, Link, Metadata, Person, Waypoint,
};

pub fn parse_extensions(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Extensions, ParseError> {
    Ok(Extensions {
        inner_xml: capture_inner_xml(reader, buf, start)?,
    })
}

pub fn parse_bounds(start: &BytesStart<'_>) -> Result<Bounds, ParseError> {
    Ok(Bounds {
        minlat: attr_f64(start, "bounds", "minlat")?,
        minlon: attr_f64(start, "bounds", "minlon")?,
        maxlat: attr_f64(start, "bounds", "maxlat")?,
        maxlon: attr_f64(start, "bounds", "maxlon")?,
    })
}

pub fn parse_email(start: &BytesStart<'_>) -> Result<Email, ParseError> {
    Ok(Email {
        id: attr_string(start, "email", "id")?,
        domain: attr_string(start, "email", "domain")?,
    })
}

pub fn parse_link(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Link, ParseError> {
    let mut link = Link {
        href: attr_string(start, "link", "href")?,
        text: None,
        link_type: None,
    };

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"text" => link.text = Some(read_element_text(reader, buf)?),
                    b"type" => link.link_type = Some(read_element_text(reader, buf)?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"link") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(link)
}

pub fn parse_copyright(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Copyright, ParseError> {
    let mut copyright = Copyright {
        author: attr_string(start, "copyright", "author")?,
        year: None,
        license: None,
    };

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"year" => copyright.year = Some(read_element_text(reader, buf)?),
                    b"license" => copyright.license = Some(read_element_text(reader, buf)?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"copyright") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(copyright)
}

pub fn parse_person(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    _start: &BytesStart<'_>,
) -> Result<Person, ParseError> {
    let mut person = Person::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"name" => person.name = Some(read_element_text(reader, buf)?),
                    b"email" => person.email = Some(parse_email(&child)?),
                    b"link" => person.link = Some(parse_link(reader, buf, &child)?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::Empty(e)) if local_name(e.name()) == b"email" => {
                person.email = Some(parse_email(&e.into_owned())?);
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"author") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(person)
}

pub fn parse_metadata(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    _start: &BytesStart<'_>,
) -> Result<Metadata, ParseError> {
    let mut metadata = Metadata::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"name" => metadata.name = Some(read_element_text(reader, buf)?),
                    b"desc" => metadata.desc = Some(read_element_text(reader, buf)?),
                    b"author" => metadata.author = Some(parse_person(reader, buf, &child)?),
                    b"copyright" => {
                        metadata.copyright = Some(parse_copyright(reader, buf, &child)?)
                    }
                    b"link" => metadata.links.push(parse_link(reader, buf, &child)?),
                    b"time" => {
                        let value = read_element_text(reader, buf)?;
                        metadata.time = Some(parse_time(&value)?);
                    }
                    b"keywords" => metadata.keywords = Some(read_element_text(reader, buf)?),
                    b"bounds" => metadata.bounds = Some(parse_bounds(&child)?),
                    b"extensions" => {
                        metadata.extensions = Some(parse_extensions(reader, buf, &child)?);
                    }
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::Empty(e)) if local_name(e.name()) == b"bounds" => {
                metadata.bounds = Some(parse_bounds(&e.into_owned())?);
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"metadata") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(metadata)
}

pub fn parse_fix(element: &'static str, value: &str) -> Result<Fix, ParseError> {
    match value {
        "none" => Ok(Fix::None),
        "2d" => Ok(Fix::TwoD),
        "3d" => Ok(Fix::ThreeD),
        "dgps" => Ok(Fix::Dgps),
        "pps" => Ok(Fix::Pps),
        _ => Err(ParseError::InvalidValue {
            element,
            value: value.to_owned(),
        }),
    }
}

pub fn parse_waypoint_child(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    child: &BytesStart<'_>,
    waypoint: &mut Waypoint,
    element: &'static str,
) -> Result<(), ParseError> {
    match local_name(child.name()) {
        b"ele" => {
            let value = read_element_text(reader, buf)?;
            waypoint.ele = Some(parse_decimal(element, "ele", &value)?);
        }
        b"time" => {
            let value = read_element_text(reader, buf)?;
            waypoint.time = Some(parse_time(&value)?);
        }
        b"magvar" => {
            let value = read_element_text(reader, buf)?;
            waypoint.magvar = Some(parse_decimal(element, "magvar", &value)?);
        }
        b"geoidheight" => {
            let value = read_element_text(reader, buf)?;
            waypoint.geoidheight = Some(parse_decimal(element, "geoidheight", &value)?);
        }
        b"name" => waypoint.name = Some(read_element_text(reader, buf)?),
        b"cmt" => waypoint.cmt = Some(read_element_text(reader, buf)?),
        b"desc" => waypoint.desc = Some(read_element_text(reader, buf)?),
        b"src" => waypoint.src = Some(read_element_text(reader, buf)?),
        b"link" => waypoint.links.push(parse_link(reader, buf, child)?),
        b"sym" => waypoint.sym = Some(read_element_text(reader, buf)?),
        b"type" => waypoint.waypoint_type = Some(read_element_text(reader, buf)?),
        b"fix" => {
            let value = read_element_text(reader, buf)?;
            waypoint.fix = Some(parse_fix(element, &value)?);
        }
        b"sat" => {
            let value = read_element_text(reader, buf)?;
            waypoint.sat = Some(super::xml::parse_u64(element, "sat", &value)?);
        }
        b"hdop" => {
            let value = read_element_text(reader, buf)?;
            waypoint.hdop = Some(parse_decimal(element, "hdop", &value)?);
        }
        b"vdop" => {
            let value = read_element_text(reader, buf)?;
            waypoint.vdop = Some(parse_decimal(element, "vdop", &value)?);
        }
        b"pdop" => {
            let value = read_element_text(reader, buf)?;
            waypoint.pdop = Some(parse_decimal(element, "pdop", &value)?);
        }
        b"ageofdgpsdata" => {
            let value = read_element_text(reader, buf)?;
            waypoint.ageofdgpsdata = Some(parse_decimal(element, "ageofdgpsdata", &value)?);
        }
        b"dgpsid" => {
            let value = read_element_text(reader, buf)?;
            waypoint.dgpsid = Some(super::xml::parse_u16(element, "dgpsid", &value)?);
        }
        b"extensions" => {
            waypoint.extensions = Some(parse_extensions(reader, buf, child)?);
        }
        _ => skip_element(reader, buf, child)?,
    }

    Ok(())
}

pub fn parse_route_track_child(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    child: &BytesStart<'_>,
    element: &'static str,
    name: &mut Option<String>,
    cmt: &mut Option<String>,
    desc: &mut Option<String>,
    src: &mut Option<String>,
    links: &mut Vec<Link>,
    number: &mut Option<u64>,
    item_type: &mut Option<String>,
    extensions: &mut Option<Extensions>,
) -> Result<bool, ParseError> {
    match local_name(child.name()) {
        b"name" => *name = Some(read_element_text(reader, buf)?),
        b"cmt" => *cmt = Some(read_element_text(reader, buf)?),
        b"desc" => *desc = Some(read_element_text(reader, buf)?),
        b"src" => *src = Some(read_element_text(reader, buf)?),
        b"link" => links.push(parse_link(reader, buf, child)?),
        b"number" => {
            let value = read_element_text(reader, buf)?;
            *number = Some(super::xml::parse_u64(element, "number", &value)?);
        }
        b"type" => *item_type = Some(read_element_text(reader, buf)?),
        b"extensions" => *extensions = Some(parse_extensions(reader, buf, child)?),
        _ => return Ok(false),
    }

    Ok(true)
}
