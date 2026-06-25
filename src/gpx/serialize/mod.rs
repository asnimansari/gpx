use std::fmt::Write as _;
use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::gpx::extensions::{PowerExtension, TrackExtension, TrackPointExtension, GPXTPTX_NS_V1, GPPXPX_NS_V1, GPXX_NS_V3};
use crate::gpx::types::{
    Bounds, Copyright, Extensions, Fix, Gpx, Link, Metadata, Person, Route, Track, TrackSegment,
    Waypoint,
};

const GPX_NS: &str = "http://www.topografix.com/GPX/1/1";
const XSI_NS: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Serialize a GPX document to XML text.
pub fn to_string(gpx: &Gpx, pretty: bool) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let mut writer = XmlWriter::new(&mut out, pretty);
    write_gpx(gpx, &mut writer);
    out
}

/// Write a GPX document to a file.
pub fn write_file(gpx: &Gpx, path: impl AsRef<Path>, pretty: bool) -> io::Result<()> {
    std::fs::write(path, to_string(gpx, pretty))
}

fn write_gpx(gpx: &Gpx, w: &mut XmlWriter<'_>) {
    w.start(
        "gpx",
        &[
            ("xmlns", GPX_NS),
            ("xmlns:xsi", XSI_NS),
            ("xsi:schemaLocation", &format!("{GPX_NS} {GPX_NS}/gpx.xsd")),
            ("version", gpx.version.as_deref().unwrap_or("1.1")),
            ("creator", gpx.creator.as_deref().unwrap_or("gpx-rs")),
        ],
    );
    if let Some(metadata) = &gpx.metadata {
        write_metadata(metadata, w);
    }
    for waypoint in &gpx.waypoints {
        write_waypoint("wpt", waypoint, w);
    }
    for route in &gpx.routes {
        write_route(route, w);
    }
    for track in &gpx.tracks {
        write_track(track, w);
    }
    if let Some(extensions) = &gpx.extensions {
        write_extensions(extensions, w);
    }
    w.end("gpx");
}

fn write_metadata(metadata: &Metadata, w: &mut XmlWriter<'_>) {
    w.start("metadata", &[]);
    write_opt_text("name", metadata.name.as_deref(), w);
    write_opt_text("desc", metadata.desc.as_deref(), w);
    if let Some(author) = &metadata.author {
        write_person(author, w);
    }
    if let Some(copyright) = &metadata.copyright {
        write_copyright(copyright, w);
    }
    for link in &metadata.links {
        write_link(link, w);
    }
    write_opt_time("time", metadata.time, w);
    write_opt_text("keywords", metadata.keywords.as_deref(), w);
    if let Some(bounds) = &metadata.bounds {
        write_bounds(bounds, w);
    }
    if let Some(extensions) = &metadata.extensions {
        write_extensions(extensions, w);
    }
    w.end("metadata");
}

fn write_person(person: &Person, w: &mut XmlWriter<'_>) {
    w.start("author", &[]);
    write_opt_text("name", person.name.as_deref(), w);
    if let Some(email) = &person.email {
        w.empty("email", &[("id", email.id.as_str()), ("domain", email.domain.as_str())]);
    }
    if let Some(link) = &person.link {
        write_link(link, w);
    }
    w.end("author");
}

fn write_copyright(copyright: &Copyright, w: &mut XmlWriter<'_>) {
    w.start("copyright", &[("author", copyright.author.as_str())]);
    write_opt_text("year", copyright.year.as_deref(), w);
    write_opt_text("license", copyright.license.as_deref(), w);
    w.end("copyright");
}

fn write_link(link: &Link, w: &mut XmlWriter<'_>) {
    w.start("link", &[("href", link.href.as_str())]);
    write_opt_text("text", link.text.as_deref(), w);
    write_opt_text("type", link.link_type.as_deref(), w);
    w.end("link");
}

fn write_bounds(bounds: &Bounds, w: &mut XmlWriter<'_>) {
    w.empty(
        "bounds",
        &[
            ("minlat", &bounds.minlat.to_string()),
            ("minlon", &bounds.minlon.to_string()),
            ("maxlat", &bounds.maxlat.to_string()),
            ("maxlon", &bounds.maxlon.to_string()),
        ],
    );
}

fn write_route(route: &Route, w: &mut XmlWriter<'_>) {
    w.start("rte", &[]);
    write_opt_text("name", route.name.as_deref(), w);
    write_opt_text("cmt", route.cmt.as_deref(), w);
    write_opt_text("desc", route.desc.as_deref(), w);
    write_opt_text("src", route.src.as_deref(), w);
    for link in &route.links {
        write_link(link, w);
    }
    if let Some(number) = route.number {
        write_text("number", &number.to_string(), w);
    }
    write_opt_text("type", route.route_type.as_deref(), w);
    if let Some(extensions) = &route.extensions {
        write_extensions(extensions, w);
    }
    for point in &route.points {
        write_waypoint("rtept", point, w);
    }
    w.end("rte");
}

fn write_track(track: &Track, w: &mut XmlWriter<'_>) {
    w.start("trk", &[]);
    write_opt_text("name", track.name.as_deref(), w);
    write_opt_text("cmt", track.cmt.as_deref(), w);
    write_opt_text("desc", track.desc.as_deref(), w);
    write_opt_text("src", track.src.as_deref(), w);
    for link in &track.links {
        write_link(link, w);
    }
    if let Some(number) = track.number {
        write_text("number", &number.to_string(), w);
    }
    write_opt_text("type", track.track_type.as_deref(), w);
    if let Some(extensions) = &track.extensions {
        write_extensions(extensions, w);
    }
    for segment in &track.segments {
        write_track_segment(segment, w);
    }
    w.end("trk");
}

fn write_track_segment(segment: &TrackSegment, w: &mut XmlWriter<'_>) {
    w.start("trkseg", &[]);
    for point in &segment.points {
        write_waypoint("trkpt", point, w);
    }
    if let Some(extensions) = &segment.extensions {
        write_extensions(extensions, w);
    }
    w.end("trkseg");
}

fn write_waypoint(tag: &str, waypoint: &Waypoint, w: &mut XmlWriter<'_>) {
    w.start(
        tag,
        &[
            ("lat", &format_coord(waypoint.lat)),
            ("lon", &format_coord(waypoint.lon)),
        ],
    );
    write_opt_decimal("ele", waypoint.ele, w);
    write_opt_time("time", waypoint.time, w);
    write_opt_decimal("magvar", waypoint.magvar, w);
    write_opt_decimal("geoidheight", waypoint.geoidheight, w);
    write_opt_text("name", waypoint.name.as_deref(), w);
    write_opt_text("cmt", waypoint.cmt.as_deref(), w);
    write_opt_text("desc", waypoint.desc.as_deref(), w);
    write_opt_text("src", waypoint.src.as_deref(), w);
    for link in &waypoint.links {
        write_link(link, w);
    }
    write_opt_text("sym", waypoint.sym.as_deref(), w);
    write_opt_text("type", waypoint.waypoint_type.as_deref(), w);
    if let Some(fix) = waypoint.fix {
        write_text("fix", fix_to_str(fix), w);
    }
    if let Some(sat) = waypoint.sat {
        write_text("sat", &sat.to_string(), w);
    }
    write_opt_decimal("hdop", waypoint.hdop, w);
    write_opt_decimal("vdop", waypoint.vdop, w);
    write_opt_decimal("pdop", waypoint.pdop, w);
    write_opt_decimal("ageofdgpsdata", waypoint.ageofdgpsdata, w);
    if let Some(dgpsid) = waypoint.dgpsid {
        write_text("dgpsid", &dgpsid.to_string(), w);
    }
    if let Some(extensions) = &waypoint.extensions {
        write_extensions(extensions, w);
    }
    w.end(tag);
}

fn write_extensions(extensions: &Extensions, w: &mut XmlWriter<'_>) {
    if extensions.is_empty() {
        w.empty("extensions", &[]);
        return;
    }

    w.start("extensions", &[]);
    if let Some(tpx) = &extensions.track_point {
        w.raw_inline(&track_point_extension_xml(tpx));
    }
    if let Some(watts) = extensions.power {
        write_text("power", &watts.to_string(), w);
    }
    if let Some(power) = &extensions.power_extension {
        w.raw_inline(&power_extension_xml(power));
    }
    if let Some(track) = &extensions.track {
        w.raw_inline(&track_extension_xml(track));
    }
    if !extensions.inner_xml.is_empty() {
        w.raw_inline(&extensions.inner_xml);
    }
    w.end("extensions");
}

fn track_point_extension_xml(tpx: &TrackPointExtension) -> String {
    let mut out = format!(
        r#"<gpxtpx:TrackPointExtension xmlns:gpxtpx="{GPXTPTX_NS_V1}">"#
    );
    if let Some(atemp) = tpx.atemp {
        out.push_str(&format!("<gpxtpx:atemp>{atemp}</gpxtpx:atemp>"));
    }
    if let Some(wtemp) = tpx.wtemp {
        out.push_str(&format!("<gpxtpx:wtemp>{wtemp}</gpxtpx:wtemp>"));
    }
    if let Some(depth) = tpx.depth {
        out.push_str(&format!("<gpxtpx:depth>{depth}</gpxtpx:depth>"));
    }
    if let Some(hr) = tpx.hr {
        out.push_str(&format!("<gpxtpx:hr>{hr}</gpxtpx:hr>"));
    }
    if let Some(cad) = tpx.cad {
        out.push_str(&format!("<gpxtpx:cad>{cad}</gpxtpx:cad>"));
    }
    out.push_str("</gpxtpx:TrackPointExtension>");
    out
}

fn power_extension_xml(power: &PowerExtension) -> String {
    let mut out = format!(r#"<gpxpx:PowerExtension xmlns:gpxpx="{GPPXPX_NS_V1}">"#);
    if let Some(watts) = power.power_in_watts {
        out.push_str(&format!("<gpxpx:PowerInWatts>{watts}</gpxpx:PowerInWatts>"));
    }
    out.push_str("</gpxpx:PowerExtension>");
    out
}

fn track_extension_xml(track: &TrackExtension) -> String {
    let mut out = format!(r#"<gpxx:TrackExtension xmlns:gpxx="{GPXX_NS_V3}">"#);
    if let Some(color) = &track.display_color {
        out.push_str(&format!(
            "<gpxx:DisplayColor>{}</gpxx:DisplayColor>",
            escape_text(color)
        ));
    }
    out.push_str("</gpxx:TrackExtension>");
    out
}

fn write_opt_text(tag: &str, value: Option<&str>, w: &mut XmlWriter<'_>) {
    if let Some(value) = value {
        write_text(tag, value, w);
    }
}

fn write_text(tag: &str, value: &str, w: &mut XmlWriter<'_>) {
    w.text_element(tag, value);
}

fn write_opt_decimal(tag: &str, value: Option<f64>, w: &mut XmlWriter<'_>) {
    if let Some(value) = value {
        write_text(tag, &format_decimal(value), w);
    }
}

fn write_opt_time(tag: &str, value: Option<DateTime<Utc>>, w: &mut XmlWriter<'_>) {
    if let Some(value) = value {
        write_text(tag, &value.to_rfc3339(), w);
    }
}

fn format_coord(value: f64) -> String {
    format_decimal(value)
}

fn format_decimal(value: f64) -> String {
    let s = format!("{value}");
    if s.contains('e') || s.contains('E') {
        format!("{value:.12}")
    } else {
        s
    }
}

fn fix_to_str(fix: Fix) -> &'static str {
    match fix {
        Fix::None => "none",
        Fix::TwoD => "2d",
        Fix::ThreeD => "3d",
        Fix::Dgps => "dgps",
        Fix::Pps => "pps",
    }
}

struct XmlWriter<'a> {
    out: &'a mut String,
    pretty: bool,
    depth: usize,
}

impl<'a> XmlWriter<'a> {
    fn new(out: &'a mut String, pretty: bool) -> Self {
        Self {
            out,
            pretty,
            depth: 0,
        }
    }

    fn indent(&mut self) {
        if self.pretty {
            self.out.push_str(&"  ".repeat(self.depth));
        }
    }

    fn start(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        self.indent();
        self.out.push('<');
        self.out.push_str(tag);
        for (name, value) in attrs {
            let _ = write!(self.out, " {name}=\"{}\"", escape_attr(value));
        }
        self.out.push('>');
        if self.pretty {
            self.out.push('\n');
        }
        self.depth += 1;
    }

    fn end(&mut self, tag: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.indent();
        let _ = write!(self.out, "</{tag}>");
        if self.pretty {
            self.out.push('\n');
        }
    }

    fn empty(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        self.indent();
        self.out.push('<');
        self.out.push_str(tag);
        for (name, value) in attrs {
            let _ = write!(self.out, " {name}=\"{}\"", escape_attr(value));
        }
        self.out.push_str("/>");
        if self.pretty {
            self.out.push('\n');
        }
    }

    fn text_element(&mut self, tag: &str, text: &str) {
        self.indent();
        let _ = write!(
            self.out,
            "<{tag}>{}</{tag}>",
            escape_text(text),
            tag = tag
        );
        if self.pretty {
            self.out.push('\n');
        }
    }

    fn raw_inline(&mut self, inner: &str) {
        if self.pretty {
            self.indent();
        }
        self.out.push_str(inner);
        if self.pretty {
            self.out.push('\n');
        }
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}
