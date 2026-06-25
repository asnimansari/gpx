use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::{json, Value};

use crate::gpx::error::ParseError;
use crate::gpx::serialize;
use crate::gpx::types::{Gpx, Route, Track, TrackSegment, Waypoint};

const GPX_FORMAT: &str = "gpx";
const GEOJSON_FORMAT: &str = "geojson";
const JSON_FORMAT: &str = "json";
const KML_FORMAT: &str = "kml";

const KML_NS: &str = "http://www.opengis.net/kml/2.2";

/// Errors raised while converting between GPX, GeoJSON, and KML.
#[derive(Debug)]
pub enum ConvertError {
    Io(io::Error),
    Parse(ParseError),
    Json(serde_json::Error),
    Xml(String),
    UnsupportedFormat(String),
    UnsupportedGeometry(String),
    InvalidCoordinates(String),
    MissingFormat,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Parse(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::Xml(msg) => write!(f, "XML error: {msg}"),
            Self::UnsupportedFormat(format) => write!(f, "unsupported format: {format}"),
            Self::UnsupportedGeometry(kind) => write!(f, "unsupported geometry type: {kind}"),
            Self::InvalidCoordinates(msg) => write!(f, "invalid coordinates: {msg}"),
            Self::MissingFormat => write!(f, "could not detect file format from extension"),
        }
    }
}

impl std::error::Error for ConvertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Parse(err) => Some(err),
            Self::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ConvertError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ParseError> for ConvertError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<serde_json::Error> for ConvertError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Detect a supported format from a file extension.
pub fn detect_format(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "gpx" => Some(GPX_FORMAT),
        "geojson" => Some(GEOJSON_FORMAT),
        "json" => Some(JSON_FORMAT),
        "kml" => Some(KML_FORMAT),
        _ => None,
    }
}

/// Read a GPX file.
pub fn read_gpx(path: &Path) -> Result<Gpx, ConvertError> {
    Ok(Gpx::parse_file(path)?)
}

/// Read a GeoJSON file into a [`Gpx`] document.
pub fn read_geojson(path: &Path) -> Result<Gpx, ConvertError> {
    let data = fs::read_to_string(path)?;
    parse_geojson(&data)
}

/// Read a KML 2.2 file into a [`Gpx`] document.
pub fn read_kml(path: &Path) -> Result<Gpx, ConvertError> {
    let data = fs::read_to_string(path)?;
    parse_kml(&data)
}

/// Write a [`Gpx`] document as GeoJSON to a file.
pub fn write_geojson(gpx: &Gpx, path: &Path) -> Result<(), ConvertError> {
    let data = geojson_string(gpx)?;
    fs::write(path, data)?;
    Ok(())
}

/// Write a [`Gpx`] document as KML 2.2 to a file.
pub fn write_kml(gpx: &Gpx, path: &Path, pretty: bool) -> Result<(), ConvertError> {
    fs::write(path, kml_string(gpx, pretty))?;
    Ok(())
}

/// Convert between supported file formats.
///
/// When `input_format` or `output_format` is `None`, the format is inferred from
/// the corresponding path extension. Returns the resolved `(input_format, output_format)`.
pub fn convert_file(
    input: &Path,
    output: &Path,
    input_format: Option<&str>,
    output_format: Option<&str>,
) -> Result<(String, String), ConvertError> {
    let in_fmt = resolve_format(input_format, input)?;
    let out_fmt = resolve_format(output_format, output)?;

    let gpx = read_by_format(input, &in_fmt)?;
    write_by_format(&gpx, output, &out_fmt, true)?;

    Ok((in_fmt, out_fmt))
}

fn resolve_format(explicit: Option<&str>, path: &Path) -> Result<String, ConvertError> {
    if let Some(format) = explicit {
        return Ok(normalize_format(format)?.to_string());
    }
    detect_format(path)
        .map(str::to_string)
        .ok_or(ConvertError::MissingFormat)
}

fn normalize_format(format: &str) -> Result<&'static str, ConvertError> {
    match format.to_ascii_lowercase().as_str() {
        GPX_FORMAT => Ok(GPX_FORMAT),
        GEOJSON_FORMAT | JSON_FORMAT => Ok(GEOJSON_FORMAT),
        KML_FORMAT => Ok(KML_FORMAT),
        other => Err(ConvertError::UnsupportedFormat(other.to_string())),
    }
}

fn read_by_format(path: &Path, format: &str) -> Result<Gpx, ConvertError> {
    match format {
        GPX_FORMAT => read_gpx(path),
        GEOJSON_FORMAT | JSON_FORMAT => read_geojson(path),
        KML_FORMAT => read_kml(path),
        other => Err(ConvertError::UnsupportedFormat(other.to_string())),
    }
}

fn write_by_format(gpx: &Gpx, path: &Path, format: &str, pretty: bool) -> Result<(), ConvertError> {
    match format {
        GPX_FORMAT => serialize::write_file(gpx, path, pretty).map_err(ConvertError::from),
        GEOJSON_FORMAT | JSON_FORMAT => write_geojson(gpx, path),
        KML_FORMAT => write_kml(gpx, path, pretty),
        other => Err(ConvertError::UnsupportedFormat(other.to_string())),
    }
}

fn parse_geojson(data: &str) -> Result<Gpx, ConvertError> {
    let value: Value = serde_json::from_str(data)?;
    let mut gpx = Gpx::default();

    match value.get("type").and_then(Value::as_str) {
        Some("FeatureCollection") => {
            if let Some(features) = value.get("features").and_then(Value::as_array) {
                for feature in features {
                    ingest_feature(feature, &mut gpx)?;
                }
            }
        }
        Some("Feature") => ingest_feature(&value, &mut gpx)?,
        Some("Point" | "LineString" | "MultiPoint" | "MultiLineString") => {
            ingest_geometry(&value, None, None, &mut gpx)?;
        }
        Some(other) => return Err(ConvertError::UnsupportedGeometry(other.to_string())),
        None => return Err(ConvertError::UnsupportedGeometry("missing type".to_string())),
    }

    Ok(gpx)
}

fn ingest_feature(feature: &Value, gpx: &mut Gpx) -> Result<(), ConvertError> {
    let properties = feature.get("properties");
    let name = properties
        .and_then(|p| p.get("name"))
        .or_else(|| properties.and_then(|p| p.get("title")))
        .and_then(Value::as_str)
        .map(str::to_string);
    let desc = properties
        .and_then(|p| p.get("description"))
        .or_else(|| properties.and_then(|p| p.get("desc")))
        .and_then(Value::as_str)
        .map(str::to_string);
    let gpx_type = properties
        .and_then(|p| p.get("gpx_type"))
        .and_then(Value::as_str);

    if let Some(geometry) = feature.get("geometry") {
        ingest_geometry_with_hint(geometry, name, desc, gpx, gpx_type)?;
    }

    Ok(())
}

fn ingest_geometry(
    geometry: &Value,
    name: Option<String>,
    desc: Option<String>,
    gpx: &mut Gpx,
) -> Result<(), ConvertError> {
    ingest_geometry_with_hint(geometry, name, desc, gpx, None)
}

fn ingest_geometry_with_hint(
    geometry: &Value,
    name: Option<String>,
    desc: Option<String>,
    gpx: &mut Gpx,
    gpx_type: Option<&str>,
) -> Result<(), ConvertError> {
    let geo_type = geometry
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ConvertError::UnsupportedGeometry("missing type".to_string()))?;

    match geo_type {
        "Point" => {
            let waypoint = parse_geojson_point(geometry.get("coordinates").ok_or_else(|| {
                ConvertError::InvalidCoordinates("Point missing coordinates".to_string())
            })?)?;
            gpx.waypoints.push(apply_waypoint_meta(waypoint, name, desc));
        }
        "MultiPoint" => {
            let coords = geometry
                .get("coordinates")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ConvertError::InvalidCoordinates("MultiPoint missing coordinates".to_string())
                })?;
            for coord in coords {
                let waypoint = parse_geojson_point(coord)?;
                gpx.waypoints.push(apply_waypoint_meta(waypoint, name.clone(), desc.clone()));
            }
        }
        "LineString" => {
            let points = parse_geojson_line(geometry.get("coordinates").ok_or_else(|| {
                ConvertError::InvalidCoordinates("LineString missing coordinates".to_string())
            })?)?;
            if matches!(gpx_type, Some("track" | "trk")) {
                gpx.tracks.push(Track {
                    name,
                    desc,
                    segments: vec![TrackSegment { points, ..Default::default() }],
                    ..Default::default()
                });
            } else {
                gpx.routes.push(Route {
                    name,
                    desc,
                    points,
                    ..Default::default()
                });
            }
        }
        "MultiLineString" => {
            let lines = geometry
                .get("coordinates")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ConvertError::InvalidCoordinates(
                        "MultiLineString missing coordinates".to_string(),
                    )
                })?;
            let mut segments = Vec::new();
            for line in lines {
                segments.push(TrackSegment {
                    points: parse_geojson_line(line)?,
                    ..Default::default()
                });
            }
            gpx.tracks.push(Track {
                name,
                desc,
                segments,
                ..Default::default()
            });
        }
        other => return Err(ConvertError::UnsupportedGeometry(other.to_string())),
    }

    Ok(())
}

fn apply_waypoint_meta(mut waypoint: Waypoint, name: Option<String>, desc: Option<String>) -> Waypoint {
    if name.is_some() {
        waypoint.name = name;
    }
    if desc.is_some() {
        waypoint.desc = desc;
    }
    waypoint
}

fn parse_geojson_point(value: &Value) -> Result<Waypoint, ConvertError> {
    let coords = value
        .as_array()
        .ok_or_else(|| ConvertError::InvalidCoordinates("expected coordinate array".to_string()))?;
    if coords.len() < 2 {
        return Err(ConvertError::InvalidCoordinates(
            "coordinate array must contain at least lon and lat".to_string(),
        ));
    }
    let lon = coord_component(&coords[0])?;
    let lat = coord_component(&coords[1])?;
    let ele = if coords.len() > 2 {
        Some(coord_component(&coords[2])?)
    } else {
        None
    };
    Ok(waypoint(lat, lon, ele))
}

fn parse_geojson_line(value: &Value) -> Result<Vec<Waypoint>, ConvertError> {
    let coords = value
        .as_array()
        .ok_or_else(|| ConvertError::InvalidCoordinates("expected coordinate array".to_string()))?;
    coords.iter().map(parse_geojson_point).collect()
}

fn coord_component(value: &Value) -> Result<f64, ConvertError> {
    value
        .as_f64()
        .ok_or_else(|| ConvertError::InvalidCoordinates(format!("expected number, got {value}")))
}

fn waypoint(lat: f64, lon: f64, ele: Option<f64>) -> Waypoint {
    Waypoint {
        lat,
        lon,
        ele,
        time: None,
        magvar: None,
        geoidheight: None,
        name: None,
        cmt: None,
        desc: None,
        src: None,
        links: vec![],
        sym: None,
        waypoint_type: None,
        fix: None,
        sat: None,
        hdop: None,
        vdop: None,
        pdop: None,
        ageofdgpsdata: None,
        dgpsid: None,
        extensions: None,
    }
}

fn geojson_string(gpx: &Gpx) -> Result<String, ConvertError> {
    let mut features = Vec::new();

    for waypoint in &gpx.waypoints {
        features.push(waypoint_feature(waypoint));
    }
    for route in &gpx.routes {
        features.push(route_feature(route));
    }
    for track in &gpx.tracks {
        features.extend(track_features(track));
    }

    let collection = json!({
        "type": "FeatureCollection",
        "features": features,
    });
    Ok(serde_json::to_string_pretty(&collection)?)
}

fn waypoint_feature(waypoint: &Waypoint) -> Value {
    let mut properties = json!({ "gpx_type": "waypoint" });
    if let Some(name) = &waypoint.name {
        properties["name"] = json!(name);
    }
    if let Some(desc) = &waypoint.desc {
        properties["description"] = json!(desc);
    }

    json!({
        "type": "Feature",
        "properties": properties,
        "geometry": {
            "type": "Point",
            "coordinates": waypoint_coordinates(waypoint),
        },
    })
}

fn route_feature(route: &Route) -> Value {
    let mut properties = json!({ "gpx_type": "route" });
    if let Some(name) = &route.name {
        properties["name"] = json!(name);
    }
    if let Some(desc) = &route.desc {
        properties["description"] = json!(desc);
    }

    json!({
        "type": "Feature",
        "properties": properties,
        "geometry": {
            "type": "LineString",
            "coordinates": route.points.iter().map(waypoint_coordinates).collect::<Vec<_>>(),
        },
    })
}

fn track_features(track: &Track) -> Vec<Value> {
    track
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let mut properties = json!({ "gpx_type": "track" });
            if let Some(name) = &track.name {
                properties["name"] = json!(name);
            }
            if let Some(desc) = &track.desc {
                properties["description"] = json!(desc);
            }
            if track.segments.len() > 1 {
                properties["segment"] = json!(index);
            }

            json!({
                "type": "Feature",
                "properties": properties,
                "geometry": {
                    "type": "LineString",
                    "coordinates": segment.points.iter().map(waypoint_coordinates).collect::<Vec<_>>(),
                },
            })
        })
        .collect()
}

fn waypoint_coordinates(waypoint: &Waypoint) -> Value {
    match waypoint.ele {
        Some(ele) => json!([waypoint.lon, waypoint.lat, ele]),
        None => json!([waypoint.lon, waypoint.lat]),
    }
}

fn parse_kml(data: &str) -> Result<Gpx, ConvertError> {
    let data = strip_kml_namespace(data);
    let placemarks = parse_kml_placemarks(&data)?;
    let mut gpx = Gpx::default();

    for placemark in placemarks {
        if let Some(point) = placemark.point {
            let mut waypoint = parse_kml_coordinates(&point)?.into_iter().next().ok_or_else(|| {
                ConvertError::InvalidCoordinates("Point placemark has no coordinates".to_string())
            })?;
            waypoint.name = placemark.name.clone();
            waypoint.desc = placemark.description.clone();
            gpx.waypoints.push(waypoint);
        } else if let Some(line) = placemark.line_string {
            let points = parse_kml_coordinates(&line)?;
            gpx.routes.push(Route {
                name: placemark.name.clone(),
                desc: placemark.description.clone(),
                points,
                ..Default::default()
            });
        } else if let Some(lines) = placemark.multi_geometry {
            let mut segments = Vec::new();
            for line in lines {
                segments.push(TrackSegment {
                    points: parse_kml_coordinates(&line)?,
                    ..Default::default()
                });
            }
            gpx.tracks.push(Track {
                name: placemark.name.clone(),
                desc: placemark.description.clone(),
                segments,
                ..Default::default()
            });
        }
    }

    Ok(gpx)
}

fn strip_kml_namespace(xml: &str) -> String {
    xml.replace(&format!(r#" xmlns="{KML_NS}""#), "")
        .replace(&format!(" xmlns='{KML_NS}'"), "")
}

#[derive(Debug, Default)]
struct KmlPlacemarkData {
    name: Option<String>,
    description: Option<String>,
    point: Option<String>,
    line_string: Option<String>,
    multi_geometry: Option<Vec<String>>,
}

fn parse_kml_placemarks(data: &str) -> Result<Vec<KmlPlacemarkData>, ConvertError> {
    let mut reader = Reader::from_str(data);
    reader.config_mut().trim_text(true);

    let mut placemarks = Vec::new();
    let mut current: Option<KmlPlacemarkData> = None;
    let mut element_stack: Vec<String> = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = local_name_start(&e);
                if name == "Placemark" {
                    current = Some(KmlPlacemarkData::default());
                }
                if let Some(placemark) = current.as_mut() {
                    match name.as_str() {
                        "Point" if placemark.point.is_none() && placemark.line_string.is_none() => {}
                        "LineString" if placemark.line_string.is_none() && placemark.multi_geometry.is_none() => {}
                        "MultiGeometry" if placemark.multi_geometry.is_none() => {
                            placemark.multi_geometry = Some(Vec::new());
                        }
                        "coordinates" => text.clear(),
                        _ => {}
                    }
                }
                element_stack.push(name);
            }
            Ok(Event::Text(e)) => {
                if element_stack.last().is_some_and(|tag| tag == "coordinates") {
                    text.push_str(&e.unescape().map_err(|err| ConvertError::Xml(err.to_string()))?);
                } else if element_stack.last().is_some_and(|tag| tag == "name" || tag == "description") {
                    if let Some(placemark) = current.as_mut() {
                        let value = e
                            .unescape()
                            .map_err(|err| ConvertError::Xml(err.to_string()))?
                            .into_owned();
                        match element_stack.last().map(String::as_str) {
                            Some("name") => placemark.name = Some(value),
                            Some("description") => placemark.description = Some(value),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name_end(&e);
                if name == "coordinates" {
                    if let Some(placemark) = current.as_mut() {
                        let coords = text.trim().to_string();
                        if element_stack.iter().any(|tag| tag == "MultiGeometry") {
                            if let Some(lines) = placemark.multi_geometry.as_mut() {
                                lines.push(coords);
                            }
                        } else if element_stack.iter().any(|tag| tag == "LineString") {
                            placemark.line_string = Some(coords);
                        } else if element_stack.iter().any(|tag| tag == "Point") {
                            placemark.point = Some(coords);
                        }
                    }
                    text.clear();
                }
                if name == "Placemark" {
                    if let Some(placemark) = current.take() {
                        placemarks.push(placemark);
                    }
                }
                element_stack.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(ConvertError::Xml(err.to_string())),
        }
    }

    Ok(placemarks)
}

fn local_name_start(event: &quick_xml::events::BytesStart<'_>) -> String {
    String::from_utf8_lossy(event.local_name().as_ref()).into_owned()
}

fn local_name_end(event: &quick_xml::events::BytesEnd<'_>) -> String {
    String::from_utf8_lossy(event.local_name().as_ref()).into_owned()
}

fn parse_kml_coordinates(raw: &str) -> Result<Vec<Waypoint>, ConvertError> {
    let mut points = Vec::new();
    for token in raw.split_whitespace() {
        let parts: Vec<&str> = token.split(',').collect();
        if parts.len() < 2 {
            return Err(ConvertError::InvalidCoordinates(format!(
                "expected lon,lat[,ele], got `{token}`"
            )));
        }
        let lon = parts[0]
            .trim()
            .parse::<f64>()
            .map_err(|_| ConvertError::InvalidCoordinates(format!("invalid longitude `{token}`")))?;
        let lat = parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| ConvertError::InvalidCoordinates(format!("invalid latitude `{token}`")))?;
        let ele = if parts.len() > 2 && !parts[2].trim().is_empty() {
            Some(
                parts[2]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ConvertError::InvalidCoordinates(format!("invalid elevation `{token}`")))?,
            )
        } else {
            None
        };
        points.push(waypoint(lat, lon, ele));
    }
    Ok(points)
}

fn kml_string(gpx: &Gpx, pretty: bool) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    if pretty {
        out.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");
    } else {
        out.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\"><Document>");
    }

    if let Some(metadata) = &gpx.metadata {
        if let Some(name) = &metadata.name {
            write_kml_text("name", name, &mut out, pretty, 2);
        }
        if let Some(desc) = &metadata.desc {
            write_kml_text("description", desc, &mut out, pretty, 2);
        }
    }

    for waypoint in &gpx.waypoints {
        write_kml_waypoint_placemark(waypoint, &mut out, pretty);
    }
    for route in &gpx.routes {
        write_kml_route_placemark(route, &mut out, pretty);
    }
    for track in &gpx.tracks {
        write_kml_track_placemark(track, &mut out, pretty);
    }

    if pretty {
        out.push_str("  </Document>\n</kml>\n");
    } else {
        out.push_str("</Document></kml>");
    }
    out
}

fn write_kml_waypoint_placemark(waypoint: &Waypoint, out: &mut String, pretty: bool) {
    write_kml_placemark_start(out, pretty, 2);
    write_kml_optional_text("name", waypoint.name.as_deref(), out, pretty, 3);
    write_kml_optional_text("description", waypoint.desc.as_deref(), out, pretty, 3);
    write_kml_tag_open("Point", out, pretty, 3);
    write_kml_coordinates(std::slice::from_ref(waypoint), out, pretty, 4);
    write_kml_tag_close("Point", out, pretty, 3);
    write_kml_placemark_end(out, pretty, 2);
}

fn write_kml_route_placemark(route: &Route, out: &mut String, pretty: bool) {
    write_kml_placemark_start(out, pretty, 2);
    write_kml_optional_text("name", route.name.as_deref(), out, pretty, 3);
    write_kml_optional_text("description", route.desc.as_deref(), out, pretty, 3);
    write_kml_tag_open("LineString", out, pretty, 3);
    write_kml_coordinates(&route.points, out, pretty, 4);
    write_kml_tag_close("LineString", out, pretty, 3);
    write_kml_placemark_end(out, pretty, 2);
}

fn write_kml_track_placemark(track: &Track, out: &mut String, pretty: bool) {
    write_kml_placemark_start(out, pretty, 2);
    write_kml_optional_text("name", track.name.as_deref(), out, pretty, 3);
    write_kml_optional_text("description", track.desc.as_deref(), out, pretty, 3);
    if track.segments.len() == 1 {
        write_kml_tag_open("LineString", out, pretty, 3);
        write_kml_coordinates(&track.segments[0].points, out, pretty, 4);
        write_kml_tag_close("LineString", out, pretty, 3);
    } else {
        write_kml_tag_open("MultiGeometry", out, pretty, 3);
        for segment in &track.segments {
            write_kml_tag_open("LineString", out, pretty, 4);
            write_kml_coordinates(&segment.points, out, pretty, 5);
            write_kml_tag_close("LineString", out, pretty, 4);
        }
        write_kml_tag_close("MultiGeometry", out, pretty, 3);
    }
    write_kml_placemark_end(out, pretty, 2);
}

fn write_kml_placemark_start(out: &mut String, pretty: bool, depth: usize) {
    write_kml_tag_open("Placemark", out, pretty, depth);
}

fn write_kml_placemark_end(out: &mut String, pretty: bool, depth: usize) {
    write_kml_tag_close("Placemark", out, pretty, depth);
}

fn write_kml_coordinates(points: &[Waypoint], out: &mut String, pretty: bool, depth: usize) {
    let coords = points
        .iter()
        .map(|point| match point.ele {
            Some(ele) => format!("{},{},{}", point.lon, point.lat, ele),
            None => format!("{},{}", point.lon, point.lat),
        })
        .collect::<Vec<_>>()
        .join(if pretty { "\n" } else { " " });
    write_kml_text("coordinates", &coords, out, pretty, depth);
}

fn write_kml_optional_text(tag: &str, value: Option<&str>, out: &mut String, pretty: bool, depth: usize) {
    if let Some(value) = value {
        write_kml_text(tag, value, out, pretty, depth);
    }
}

fn write_kml_text(tag: &str, value: &str, out: &mut String, pretty: bool, depth: usize) {
    if pretty {
        out.push_str(&"  ".repeat(depth));
    }
    out.push('<');
    out.push_str(tag);
    out.push('>');
    out.push_str(&escape_xml(value));
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
    if pretty {
        out.push('\n');
    }
}

fn write_kml_tag_open(tag: &str, out: &mut String, pretty: bool, depth: usize) {
    if pretty {
        out.push_str(&"  ".repeat(depth));
    }
    out.push('<');
    out.push_str(tag);
    out.push('>');
    if pretty {
        out.push('\n');
    }
}

fn write_kml_tag_close(tag: &str, out: &mut String, pretty: bool, depth: usize) {
    if pretty {
        out.push_str(&"  ".repeat(depth));
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
    if pretty {
        out.push('\n');
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_format_extensions() {
        assert_eq!(detect_format(Path::new("a.gpx")), Some("gpx"));
        assert_eq!(detect_format(Path::new("a.geojson")), Some("geojson"));
        assert_eq!(detect_format(Path::new("a.json")), Some("json"));
        assert_eq!(detect_format(Path::new("a.kml")), Some("kml"));
        assert_eq!(detect_format(Path::new("a.txt")), None);
    }

    #[test]
    fn geojson_point_to_waypoint() {
        let gpx = parse_geojson(
            r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[-122.3,47.6,12.0]},"properties":{"name":"Test"}}"#,
        )
        .unwrap();
        assert_eq!(gpx.waypoints.len(), 1);
        assert!((gpx.waypoints[0].lon - (-122.3)).abs() < f64::EPSILON);
        assert!((gpx.waypoints[0].lat - 47.6).abs() < f64::EPSILON);
        assert_eq!(gpx.waypoints[0].ele, Some(12.0));
        assert_eq!(gpx.waypoints[0].name.as_deref(), Some("Test"));
    }

    #[test]
    fn geojson_linestring_to_route() {
        let gpx = parse_geojson(
            r#"{"type":"Feature","geometry":{"type":"LineString","coordinates":[[-122.3,47.6],[-122.4,47.7]]}}"#,
        )
        .unwrap();
        assert_eq!(gpx.routes.len(), 1);
        assert_eq!(gpx.routes[0].points.len(), 2);
    }

    #[test]
    fn geojson_multilinestring_to_track() {
        let gpx = parse_geojson(
            r#"{"type":"Feature","geometry":{"type":"MultiLineString","coordinates":[[[-122.3,47.6],[-122.4,47.7]],[[-122.5,47.8],[-122.6,47.9]]]}}"#,
        )
        .unwrap();
        assert_eq!(gpx.tracks.len(), 1);
        assert_eq!(gpx.tracks[0].segments.len(), 2);
    }

    #[test]
    fn kml_point_and_linestring() {
        let kml = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <Placemark>
      <name>Point A</name>
      <Point><coordinates>-122.3,47.6,10</coordinates></Point>
    </Placemark>
    <Placemark>
      <name>Route B</name>
      <LineString><coordinates>-122.3,47.6 -122.4,47.7</coordinates></LineString>
    </Placemark>
    <Placemark>
      <name>Track C</name>
      <MultiGeometry>
        <LineString><coordinates>-122.3,47.6 -122.4,47.7</coordinates></LineString>
        <LineString><coordinates>-122.5,47.8 -122.6,47.9</coordinates></LineString>
      </MultiGeometry>
    </Placemark>
  </Document>
</kml>"#;
        let gpx = parse_kml(kml).unwrap();
        assert_eq!(gpx.waypoints.len(), 1);
        assert_eq!(gpx.routes.len(), 1);
        assert_eq!(gpx.tracks.len(), 1);
        assert_eq!(gpx.tracks[0].segments.len(), 2);
    }
}
