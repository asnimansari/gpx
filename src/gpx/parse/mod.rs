use crate::gpx::error::ParseError;
use crate::gpx::types::Gpx;

const GPX_NS: &str = "http://www.topografix.com/GPX/1/1";

/// Parse a GPX document from XML text using Serde deserialization.
pub fn parse_gpx(data: &str) -> Result<Gpx, ParseError> {
    quick_xml::de::from_str(data)
        .or_else(|_| quick_xml::de::from_str(&strip_default_namespace(data)))
        .map_err(ParseError::from)
}

fn strip_default_namespace(xml: &str) -> String {
    xml.replace(&format!(r#" xmlns="{GPX_NS}""#), "")
        .replace(&format!(" xmlns='{GPX_NS}'"), "")
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
        assert!(matches!(err, ParseError::De(_)));
    }
}
