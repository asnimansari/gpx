mod components;
mod xml;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use components::{
    parse_extensions, parse_metadata, parse_route_track_child, parse_waypoint_child,
};
use xml::{attr_f64, attr_value, is_local_name, local_name, skip_element};
use crate::gpx::error::ParseError;
use crate::gpx::types::{Gpx, Route, Track, TrackSegment, Waypoint};

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
                gpx.creator = attr_value(&start, "creator")?;
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
                    b"metadata" => gpx.metadata = Some(parse_metadata(reader, buf, &start)?),
                    b"wpt" => gpx.waypoints.push(parse_waypoint(reader, buf, &start, "wpt")?),
                    b"rte" => gpx.routes.push(parse_route(reader, buf, &start)?),
                    b"trk" => gpx.tracks.push(parse_track(reader, buf, &start)?),
                    b"extensions" => gpx.extensions = Some(parse_extensions(reader, buf, &start)?),
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
    _start: &BytesStart<'_>,
) -> Result<Route, ParseError> {
    let mut route = Route::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                if local_name(child.name()) == b"rtept" {
                    route.points.push(parse_waypoint(reader, buf, &child, "rtept")?);
                } else if !parse_route_track_child(
                    reader,
                    buf,
                    &child,
                    "rte",
                    &mut route.name,
                    &mut route.cmt,
                    &mut route.desc,
                    &mut route.src,
                    &mut route.links,
                    &mut route.number,
                    &mut route.route_type,
                    &mut route.extensions,
                )? {
                    skip_element(reader, buf, &child)?;
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

    Ok(route)
}

fn parse_track(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    _start: &BytesStart<'_>,
) -> Result<Track, ParseError> {
    let mut track = Track::default();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                if local_name(child.name()) == b"trkseg" {
                    track.segments.push(parse_track_segment(reader, buf)?);
                } else if !parse_route_track_child(
                    reader,
                    buf,
                    &child,
                    "trk",
                    &mut track.name,
                    &mut track.cmt,
                    &mut track.desc,
                    &mut track.src,
                    &mut track.links,
                    &mut track.number,
                    &mut track.track_type,
                    &mut track.extensions,
                )? {
                    skip_element(reader, buf, &child)?;
                }
            }
            Ok(Event::End(e)) if is_local_name(e.name(), b"trk") => break,
            Ok(Event::Eof) => return Err(ParseError::UnexpectedEof),
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
        buf.clear();
    }

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
                match local_name(child.name()) {
                    b"trkpt" => {
                        segment
                            .points
                            .push(parse_waypoint(reader, buf, &child, "trkpt")?);
                    }
                    b"extensions" => {
                        segment.extensions = Some(parse_extensions(reader, buf, &child)?);
                    }
                    _ => skip_element(reader, buf, &child)?,
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
    let mut waypoint = Waypoint::new(
        attr_f64(start, element, "lat")?,
        attr_f64(start, element, "lon")?,
    );

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let child = e.into_owned();
                parse_waypoint_child(reader, buf, &child, &mut waypoint, element)?;
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

fn parse_empty_waypoint(
    start: &BytesStart<'_>,
    element: &'static str,
) -> Result<Waypoint, ParseError> {
    Ok(Waypoint::new(
        attr_f64(start, element, "lat")?,
        attr_f64(start, element, "lon")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::gpx::types::Fix;

    const SAMPLE_GPX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="gpx-rs" xmlns="http://www.topografix.com/GPX/1/1">
  <metadata>
    <name>Sample GPX</name>
    <desc>A test file</desc>
    <author>
      <name>gpx-rs</name>
      <email id="dev" domain="example.com"/>
    </author>
    <copyright author="gpx-rs">
      <year>2026</year>
      <license>https://opensource.org/licenses/MIT</license>
    </copyright>
    <link href="https://example.com">
      <text>Example</text>
      <type>web</type>
    </link>
    <time>2002-05-30T09:00:10Z</time>
    <keywords>test,sample</keywords>
    <bounds minlat="47.0" minlon="-123.0" maxlat="48.0" maxlon="-122.0"/>
  </metadata>
  <wpt lat="47.608013" lon="-122.335167">
    <ele>4.46</ele>
    <time>2002-05-30T09:00:10Z</time>
    <magvar>12.5</magvar>
    <geoidheight>2.0</geoidheight>
    <name>Waypoint 1</name>
    <cmt>comment</cmt>
    <desc>description</desc>
    <src>source</src>
    <link href="https://example.com/wpt">
      <text>Waypoint link</text>
      <type>web</type>
    </link>
    <sym>Flag</sym>
    <type>summit</type>
    <fix>3d</fix>
    <sat>8</sat>
    <hdop>1.2</hdop>
    <vdop>1.5</vdop>
    <pdop>2.0</pdop>
    <ageofdgpsdata>1.1</ageofdgpsdata>
    <dgpsid>101</dgpsid>
  </wpt>
  <rte>
    <name>Example Route</name>
    <cmt>route comment</cmt>
    <desc>route description</desc>
    <src>route source</src>
    <number>1</number>
    <type>road</type>
    <rtept lat="47.608013" lon="-122.335167"/>
    <rtept lat="47.609123" lon="-122.336277">
      <ele>5.0</ele>
    </rtept>
  </rte>
  <trk>
    <name>Example Track</name>
    <cmt>track comment</cmt>
    <desc>track description</desc>
    <src>track source</src>
    <number>2</number>
    <type>hike</type>
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
    fn parses_full_gpx_document() {
        let gpx = parse_gpx(SAMPLE_GPX).expect("sample GPX should parse");

        assert_eq!(gpx.version.as_deref(), Some("1.1"));
        assert_eq!(gpx.creator.as_deref(), Some("gpx-rs"));

        let metadata = gpx.metadata.as_ref().expect("metadata should parse");
        assert_eq!(metadata.name.as_deref(), Some("Sample GPX"));
        assert_eq!(metadata.desc.as_deref(), Some("A test file"));
        assert_eq!(
            metadata.author.as_ref().and_then(|a| a.name.as_deref()),
            Some("gpx-rs")
        );
        let email = metadata.author.as_ref().and_then(|a| a.email.as_ref());
        assert_eq!(email.map(|e| e.id.as_str()), Some("dev"));
        assert_eq!(email.map(|e| e.domain.as_str()), Some("example.com"));
        assert_eq!(
            metadata.copyright.as_ref().map(|c| c.author.as_str()),
            Some("gpx-rs")
        );
        assert_eq!(
            metadata.copyright.as_ref().and_then(|c| c.year.as_deref()),
            Some("2026")
        );
        assert_eq!(metadata.links.len(), 1);
        assert_eq!(metadata.links[0].href, "https://example.com");
        assert_eq!(metadata.keywords.as_deref(), Some("test,sample"));
        assert_eq!(metadata.bounds.as_ref().map(|b| b.minlat), Some(47.0));

        assert_eq!(gpx.waypoints.len(), 1);
        let waypoint = &gpx.waypoints[0];
        assert_eq!(waypoint.lat, 47.608013);
        assert_eq!(waypoint.lon, -122.335167);
        assert_eq!(waypoint.ele, Some(4.46));
        assert_eq!(
            waypoint.time,
            Some(Utc.with_ymd_and_hms(2002, 5, 30, 9, 0, 10).unwrap())
        );
        assert_eq!(waypoint.magvar, Some(12.5));
        assert_eq!(waypoint.geoidheight, Some(2.0));
        assert_eq!(waypoint.name.as_deref(), Some("Waypoint 1"));
        assert_eq!(waypoint.cmt.as_deref(), Some("comment"));
        assert_eq!(waypoint.desc.as_deref(), Some("description"));
        assert_eq!(waypoint.src.as_deref(), Some("source"));
        assert_eq!(waypoint.links.len(), 1);
        assert_eq!(waypoint.sym.as_deref(), Some("Flag"));
        assert_eq!(waypoint.waypoint_type.as_deref(), Some("summit"));
        assert_eq!(waypoint.fix, Some(Fix::ThreeD));
        assert_eq!(waypoint.sat, Some(8));
        assert_eq!(waypoint.hdop, Some(1.2));
        assert_eq!(waypoint.vdop, Some(1.5));
        assert_eq!(waypoint.pdop, Some(2.0));
        assert_eq!(waypoint.ageofdgpsdata, Some(1.1));
        assert_eq!(waypoint.dgpsid, Some(101));

        let route = &gpx.routes[0];
        assert_eq!(route.name.as_deref(), Some("Example Route"));
        assert_eq!(route.cmt.as_deref(), Some("route comment"));
        assert_eq!(route.desc.as_deref(), Some("route description"));
        assert_eq!(route.src.as_deref(), Some("route source"));
        assert_eq!(route.number, Some(1));
        assert_eq!(route.route_type.as_deref(), Some("road"));
        assert_eq!(route.points.len(), 2);

        let track = &gpx.tracks[0];
        assert_eq!(track.name.as_deref(), Some("Example Track"));
        assert_eq!(track.cmt.as_deref(), Some("track comment"));
        assert_eq!(track.number, Some(2));
        assert_eq!(track.track_type.as_deref(), Some("hike"));
        assert_eq!(track.segments[0].points.len(), 2);
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
