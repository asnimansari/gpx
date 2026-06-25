use gpx_rs::{Gpx, GPXTPTX_NS_V1, GPPXPX_NS_V1};

const STRAVA_TRACK_POINT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="StravaGPX" xmlns="http://www.topografix.com/GPX/1/1">
  <trk>
    <name>Morning Ride</name>
    <trkseg>
      <trkpt lat="45.0" lon="-122.0">
        <ele>100.0</ele>
        <time>2024-01-01T10:00:00Z</time>
        <extensions>
          <gpxtpx:TrackPointExtension xmlns:gpxtpx="http://www.garmin.com/xmlschemas/TrackPointExtension/v1">
            <gpxtpx:hr>107</gpxtpx:hr>
            <gpxtpx:cad>66</gpxtpx:cad>
          </gpxtpx:TrackPointExtension>
          <gpxpx:PowerExtension xmlns:gpxpx="http://www.garmin.com/xmlschemas/PowerExtension/v1">
            <gpxpx:PowerInWatts>250</gpxpx:PowerInWatts>
          </gpxpx:PowerExtension>
        </extensions>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;

#[test]
fn parse_strava_track_point_extensions() {
    let gpx = Gpx::parse(STRAVA_TRACK_POINT).unwrap();
    let point = &gpx.tracks[0].segments[0].points[0];

    assert_eq!(point.heart_rate(), Some(107));
    assert_eq!(point.cadence(), Some(66));
    assert_eq!(point.power_watts(), Some(250));

    let ext = point.extensions.as_ref().unwrap();
    assert!(ext.track_point.is_some());
    assert!(ext.power_extension.is_some());
    assert!(ext.inner_xml.is_empty());
}

#[test]
fn round_trip_strava_extensions() {
    let gpx = Gpx::parse(STRAVA_TRACK_POINT).unwrap();
    let xml = gpx_rs::to_string(&gpx, false);
    let reparsed = Gpx::parse(&xml).unwrap();

    let orig = &gpx.tracks[0].segments[0].points[0];
    let again = &reparsed.tracks[0].segments[0].points[0];

    assert_eq!(orig.heart_rate(), again.heart_rate());
    assert_eq!(orig.cadence(), again.cadence());
    assert_eq!(orig.power_watts(), again.power_watts());

    assert!(xml.contains(GPXTPTX_NS_V1));
    assert!(xml.contains(GPPXPX_NS_V1));
    assert!(xml.contains("<gpxtpx:hr>107</gpxtpx:hr>"));
    assert!(xml.contains("<gpxpx:PowerInWatts>250</gpxpx:PowerInWatts>"));
}

#[test]
fn parse_strava_simple_power_element() {
    const STRAVA_POWER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="StravaGPX" xmlns="http://www.topografix.com/GPX/1/1"
     xmlns:gpxtpx="http://www.garmin.com/xmlschemas/TrackPointExtension/v1">
  <trk>
    <trkseg>
      <trkpt lat="9.14" lon="76.53">
        <extensions>
          <power>141</power>
          <gpxtpx:TrackPointExtension>
            <gpxtpx:hr>120</gpxtpx:hr>
            <gpxtpx:cad>80</gpxtpx:cad>
          </gpxtpx:TrackPointExtension>
        </extensions>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;

    let gpx = Gpx::parse(STRAVA_POWER).unwrap();
    let point = &gpx.tracks[0].segments[0].points[0];

    assert_eq!(point.power_watts(), Some(141));
    assert_eq!(point.heart_rate(), Some(120));
    assert_eq!(point.cadence(), Some(80));
}

const GARMIN_TRACK_EXTENSION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Garmin" xmlns="http://www.topografix.com/GPX/1/1">
  <trk>
    <name>Trail Run</name>
    <extensions>
      <gpxx:TrackExtension xmlns:gpxx="http://www.garmin.com/xmlschemas/GpxExtensions/v3">
        <gpxx:DisplayColor>DarkGray</gpxx:DisplayColor>
      </gpxx:TrackExtension>
    </extensions>
    <trkseg>
      <trkpt lat="47.0" lon="-121.0"/>
    </trkseg>
  </trk>
</gpx>"#;

#[test]
fn parse_garmin_track_extension() {
    let gpx = Gpx::parse(GARMIN_TRACK_EXTENSION).unwrap();
    let track = &gpx.tracks[0];
    let ext = track.extensions.as_ref().unwrap();

    assert_eq!(
        ext.track.as_ref().unwrap().display_color.as_deref(),
        Some("DarkGray")
    );
}
