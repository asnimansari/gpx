use chrono::{DateTime, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;

use super::error::ParseError;
use super::types::{Route, Track, TrackSegment, Waypoint};
use super::Gpx;

pub fn parse_gpx(data: &str) -> Result<Gpx, ParseError> {
    let mut reader = Reader::from_str(data);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut gpx = Gpx::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if is_local_name(e.name(), b"gpx") => {
                let start = e.into_owned();
                gpx.version = attr_value(&start, "version")?;
                parse_gpx_content(&mut reader, &mut buf, &mut gpx)?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(gpx)
}

fn parse_gpx_content(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    gpx: &mut Gpx,
) -> Result<(), ParseError> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let start = e.into_owned();
                match local_name(start.name()) {
                    b"wpt" => gpx.waypoints.push(parse_waypoint(reader, buf, &start, "wpt")?),
                    b"rte" => gpx.routes.push(parse_route(reader, buf, &start)?),
                    b"trk" => gpx.tracks.push(parse_track(reader, buf, &start)?),
                    _ => skip_element(reader, buf, &start)?,
                }
            }
            Ok(Event::Empty(e)) if local_name(e.name()) == b"wpt" => {
                gpx.waypoints.push(parse_empty_waypoint(&e.into_owned(), "wpt")?);
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"gpx") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(())
}

fn parse_route(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Route, ParseError> {
    let mut route = Route::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"name" => route.name = Some(read_element_text(reader, buf)?),
                    b"rtept" => route.points.push(parse_waypoint(reader, buf, &child, "rtept")?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::Empty(e)) if local_name(e.name()) == b"rtept" => {
                route
                    .points
                    .push(parse_empty_waypoint(&e.into_owned(), "rtept")?);
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"rte") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    let _ = start;
    Ok(route)
}

fn parse_track(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
) -> Result<Track, ParseError> {
    let mut track = Track::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"name" => track.name = Some(read_element_text(reader, buf)?),
                    b"trkseg" => track.segments.push(parse_track_segment(reader, buf)?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"trk") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    let _ = start;
    Ok(track)
}

fn parse_track_segment(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<TrackSegment, ParseError> {
    let mut segment = TrackSegment::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                if local_name(child.name()) == b"trkpt" {
                    segment
                        .points
                        .push(parse_waypoint(reader, buf, &child, "trkpt")?);
                } else {
                    skip_element(reader, buf, &child)?;
                }
            }
            Ok(Event::Empty(e)) if local_name(e.name()) == b"trkpt" => {
                segment
                    .points
                    .push(parse_empty_waypoint(&e.into_owned(), "trkpt")?);
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"trkseg") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(segment)
}

fn parse_waypoint(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    start: &BytesStart<'_>,
    element: &'static str,
) -> Result<Waypoint, ParseError> {
    let lat = attr_f64(start, element, "lat")?;
    let lon = attr_f64(start, element, "lon")?;

    let mut waypoint = Waypoint {
        lat,
        lon,
        ele: None,
        time: None,
        name: None,
    };

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                match local_name(child.name()) {
                    b"ele" => {
                        waypoint.ele = Some(read_element_text(reader, buf)?.parse().map_err(
                            |_| ParseError::InvalidAttribute {
                                element,
                                attribute: "ele",
                                value: String::new(),
                            },
                        )?);
                    }
                    b"time" => {
                        let value = read_element_text(reader, buf)?;
                        waypoint.time = Some(parse_time(&value)?);
                    }
                    b"name" => waypoint.name = Some(read_element_text(reader, buf)?),
                    _ => skip_element(reader, buf, &child)?,
                }
            }
            Ok(Event::End(e)) if is_local_name(e.name(), element.as_bytes()) => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

    Ok(waypoint)
}

fn parse_empty_waypoint(start: &BytesStart<'_>, element: &'static str) -> Result<Waypoint, ParseError> {
    Ok(Waypoint {
        lat: attr_f64(start, element, "lat")?,
        lon: attr_f64(start, element, "lon")?,
        ele: None,
        time: None,
        name: None,
    })
}

fn read_element_text(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String, ParseError> {
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

fn skip_element(
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

fn attr_value(start: &BytesStart<'_>, attribute: &str) -> Result<Option<String>, ParseError> {
    for attr in start.attributes().with_checks(false) {
        let attr = attr?;
        if attr.key.as_ref() == attribute.as_bytes() {
            return Ok(Some(String::from_utf8_lossy(&attr.value).into_owned()));
        }
    }

    Ok(None)
}

fn attr_f64(
    start: &BytesStart<'_>,
    element: &'static str,
    attribute: &'static str,
) -> Result<f64, ParseError> {
    for attr in start.attributes().with_checks(false) {
        let attr = attr?;
        if local_name(attr.key) == attribute.as_bytes() {
            let value = String::from_utf8_lossy(&attr.value).into_owned();
            return value.parse().map_err(|_| ParseError::InvalidAttribute {
                element,
                attribute,
                value,
            });
        }
    }

    Err(ParseError::MissingAttribute { element, attribute })
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| ParseError::InvalidTime(value.to_owned()))
}

fn local_name(name: QName<'_>) -> &[u8] {
    name.local_name().into_inner()
}

fn is_local_name(name: QName<'_>, expected: &[u8]) -> bool {
    local_name(name) == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const SAMPLE_GPX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="gpx-rs" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lat="47.608013" lon="-122.335167">
    <ele>4.46</ele>
    <time>2002-05-30T09:00:10Z</time>
    <name>Waypoint 1</name>
  </wpt>
  <rte>
    <name>Example Route</name>
    <rtept lat="47.608013" lon="-122.335167"/>
    <rtept lat="47.609123" lon="-122.336277">
      <ele>5.0</ele>
    </rtept>
  </rte>
  <trk>
    <name>Example Track</name>
    <trkseg>
      <trkpt lat="47.608013" lon="-122.335167">
        <ele>4.46</ele>
      </trkpt>
      <trkpt lat="47.609123" lon="-122.336277">
        <ele>5.0</ele>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;

    #[test]
    fn parses_sample_gpx_document() {
        let gpx = parse_gpx(SAMPLE_GPX).expect("sample GPX should parse");

        assert_eq!(gpx.version.as_deref(), Some("1.1"));

        assert_eq!(gpx.waypoints.len(), 1);
        let waypoint = &gpx.waypoints[0];
        assert_eq!(waypoint.lat, 47.608013);
        assert_eq!(waypoint.lon, -122.335167);
        assert_eq!(waypoint.ele, Some(4.46));
        assert_eq!(
            waypoint.time,
            Some(Utc.with_ymd_and_hms(2002, 5, 30, 9, 0, 10).unwrap())
        );
        assert_eq!(waypoint.name.as_deref(), Some("Waypoint 1"));

        assert_eq!(gpx.routes.len(), 1);
        let route = &gpx.routes[0];
        assert_eq!(route.name.as_deref(), Some("Example Route"));
        assert_eq!(route.points.len(), 2);
        assert_eq!(route.points[0].lat, 47.608013);
        assert_eq!(route.points[0].lon, -122.335167);
        assert_eq!(route.points[0].ele, None);
        assert_eq!(route.points[1].ele, Some(5.0));

        assert_eq!(gpx.tracks.len(), 1);
        let track = &gpx.tracks[0];
        assert_eq!(track.name.as_deref(), Some("Example Track"));
        assert_eq!(track.segments.len(), 1);
        assert_eq!(track.segments[0].points.len(), 2);
        assert_eq!(track.segments[0].points[0].ele, Some(4.46));
        assert_eq!(track.segments[0].points[1].ele, Some(5.0));
    }

    #[test]
    fn rejects_waypoint_without_latitude() {
        let xml = r#"<gpx version="1.1" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lon="-122.335167"/>
</gpx>"#;

        let err = parse_gpx(xml).unwrap_err();
        assert!(matches!(
            err,
            ParseError::MissingAttribute {
                element: "wpt",
                attribute: "lat"
            }
        ));
    }
}
