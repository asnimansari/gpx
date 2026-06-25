//! GPX 1.1 schema validation against a declarative rule table.

use std::fmt;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use serde::Serialize;

use crate::gpx::types::Fix;

/// GPX 1.1 namespace URI.
pub const GPX_NAMESPACE: &str = "http://www.topografix.com/GPX/1/1";

/// GPX 1.0 namespace URI (unsupported; used for error hints).
pub const GPX_10_NAMESPACE: &str = "http://www.topografix.com/GPX/1/0";

const EXTENSIONS: &str = "extensions";

fn fix_as_str(fix: Fix) -> &'static str {
    match fix {
        Fix::None => "none",
        Fix::TwoD => "2d",
        Fix::ThreeD => "3d",
        Fix::Dgps => "dgps",
        Fix::Pps => "pps",
    }
}

const FIX_VALUES: &[Fix] = &[
    Fix::None,
    Fix::TwoD,
    Fix::ThreeD,
    Fix::Dgps,
    Fix::Pps,
];

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Schema violation.
    Error,
    /// Non-fatal deviation from recommended practice.
    Warning,
}

/// A single GPX schema validation issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationIssue {
    /// Whether the issue is an error or a warning.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// Breadcrumb path (e.g. `gpx > trk[0] > trkseg[2] > trkpt[14]`).
    pub path: String,
    /// 1-based source line number, when available.
    pub line: Option<u32>,
}

impl ValidationIssue {
    /// Return a JSON value suitable for CLI output.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("ValidationIssue is always JSON-serializable")
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
        };
        if let Some(line) = self.line {
            write!(
                f,
                "{severity:<7} line {line}  {}: {}",
                self.path, self.message
            )
        } else {
            write!(f, "{severity:<7} {}: {}", self.path, self.message)
        }
    }
}

/// Result of validating GPX data against the GPX 1.1 schema.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// All issues found during validation.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// All issues with error severity.
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .collect()
    }

    /// All issues with warning severity.
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
            .collect()
    }

    /// `true` when there are no errors (warnings are allowed).
    pub fn is_valid(&self) -> bool {
        self.errors().is_empty()
    }
}

/// Raised when strict parsing encounters an invalid GPX document.
#[derive(Debug)]
pub struct InvalidGpxError {
    /// Full validation result carrying errors and warnings.
    pub result: ValidationResult,
}

impl InvalidGpxError {
    /// Create from a validation result that failed schema checks.
    pub fn new(result: ValidationResult) -> Self {
        Self { result }
    }

    /// Shortcut for schema errors only.
    pub fn issues(&self) -> Vec<&ValidationIssue> {
        self.result.errors()
    }
}

impl fmt::Display for InvalidGpxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errors = self.result.errors();
        let count = errors.len();
        let noun = if count == 1 { "error" } else { "errors" };
        writeln!(f, "Invalid GPX ({count} {noun}):")?;
        for issue in errors {
            writeln!(f, "  {issue}")?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidGpxError {}

type ContentValidator = fn(&str) -> Option<(Severity, String)>;

struct AttrRule {
    name: &'static str,
    validator: Option<ContentValidator>,
}

struct ChildRule {
    tag: &'static str,
    max_occurs: Option<u32>,
    content: Option<ContentValidator>,
    type_name: Option<&'static str>,
}

struct ComplexType {
    attrs: &'static [AttrRule],
    children: &'static [ChildRule],
}

static SCHEMA: &[(&str, ComplexType)] = &[
    (
        "gpx",
        ComplexType {
            attrs: &[
                AttrRule {
                    name: "version",
                    validator: Some(validate_version),
                },
                AttrRule {
                    name: "creator",
                    validator: None,
                },
            ],
            children: &[
                child("metadata", 1, None, Some("metadata")),
                child_unbounded("wpt", None, Some("wpt")),
                child_unbounded("rte", None, Some("rte")),
                child_unbounded("trk", None, Some("trk")),
                child("extensions", 1, None, Some(EXTENSIONS)),
            ],
        },
    ),
    (
        "metadata",
        ComplexType {
            attrs: &[],
            children: &[
                child("name", 1, None, None),
                child("desc", 1, None, None),
                child("author", 1, None, Some("person")),
                child("copyright", 1, None, Some("copyright")),
                child_unbounded("link", None, Some("link")),
                child("time", 1, Some(validate_time), None),
                child("keywords", 1, None, None),
                child("bounds", 1, None, Some("bounds")),
                child("extensions", 1, None, Some(EXTENSIONS)),
            ],
        },
    ),
    (
        "wpt",
        ComplexType {
            attrs: &[
                AttrRule {
                    name: "lat",
                    validator: Some(validate_latitude),
                },
                AttrRule {
                    name: "lon",
                    validator: Some(validate_longitude),
                },
            ],
            children: &[
                child("ele", 1, Some(validate_decimal), None),
                child("time", 1, Some(validate_time), None),
                child("magvar", 1, Some(validate_degrees), None),
                child("geoidheight", 1, Some(validate_decimal), None),
                child("name", 1, None, None),
                child("cmt", 1, None, None),
                child("desc", 1, None, None),
                child("src", 1, None, None),
                child_unbounded("link", None, Some("link")),
                child("sym", 1, None, None),
                child("type", 1, None, None),
                child("fix", 1, Some(validate_fix), None),
                child("sat", 1, Some(validate_non_negative_int), None),
                child("hdop", 1, Some(validate_decimal), None),
                child("vdop", 1, Some(validate_decimal), None),
                child("pdop", 1, Some(validate_decimal), None),
                child("ageofdgpsdata", 1, Some(validate_decimal), None),
                child("dgpsid", 1, Some(validate_dgpsid), None),
                child("extensions", 1, None, Some(EXTENSIONS)),
            ],
        },
    ),
    (
        "rte",
        ComplexType {
            attrs: &[],
            children: &[
                child("name", 1, None, None),
                child("cmt", 1, None, None),
                child("desc", 1, None, None),
                child("src", 1, None, None),
                child_unbounded("link", None, Some("link")),
                child("number", 1, Some(validate_non_negative_int), None),
                child("type", 1, None, None),
                child("extensions", 1, None, Some(EXTENSIONS)),
                child_unbounded("rtept", None, Some("wpt")),
            ],
        },
    ),
    (
        "trk",
        ComplexType {
            attrs: &[],
            children: &[
                child("name", 1, None, None),
                child("cmt", 1, None, None),
                child("desc", 1, None, None),
                child("src", 1, None, None),
                child_unbounded("link", None, Some("link")),
                child("number", 1, Some(validate_non_negative_int), None),
                child("type", 1, None, None),
                child("extensions", 1, None, Some(EXTENSIONS)),
                child_unbounded("trkseg", None, Some("trkseg")),
            ],
        },
    ),
    (
        "trkseg",
        ComplexType {
            attrs: &[],
            children: &[
                child_unbounded("trkpt", None, Some("wpt")),
                child("extensions", 1, None, Some(EXTENSIONS)),
            ],
        },
    ),
    (
        "person",
        ComplexType {
            attrs: &[],
            children: &[
                child("name", 1, None, None),
                child("email", 1, None, Some("email")),
                child("link", 1, None, Some("link")),
            ],
        },
    ),
    (
        "copyright",
        ComplexType {
            attrs: &[AttrRule {
                name: "author",
                validator: None,
            }],
            children: &[
                child("year", 1, Some(validate_gyear), None),
                child("license", 1, None, None),
            ],
        },
    ),
    (
        "link",
        ComplexType {
            attrs: &[AttrRule {
                name: "href",
                validator: None,
            }],
            children: &[child("text", 1, None, None), child("type", 1, None, None)],
        },
    ),
    (
        "email",
        ComplexType {
            attrs: &[
                AttrRule {
                    name: "id",
                    validator: None,
                },
                AttrRule {
                    name: "domain",
                    validator: None,
                },
            ],
            children: &[],
        },
    ),
    (
        "bounds",
        ComplexType {
            attrs: &[
                AttrRule {
                    name: "minlat",
                    validator: Some(validate_latitude),
                },
                AttrRule {
                    name: "minlon",
                    validator: Some(validate_longitude),
                },
                AttrRule {
                    name: "maxlat",
                    validator: Some(validate_latitude),
                },
                AttrRule {
                    name: "maxlon",
                    validator: Some(validate_longitude),
                },
            ],
            children: &[],
        },
    ),
];

const fn child(
    tag: &'static str,
    max_occurs: u32,
    content: Option<ContentValidator>,
    type_name: Option<&'static str>,
) -> ChildRule {
    ChildRule {
        tag,
        max_occurs: Some(max_occurs),
        content,
        type_name,
    }
}

const fn child_unbounded(
    tag: &'static str,
    content: Option<ContentValidator>,
    type_name: Option<&'static str>,
) -> ChildRule {
    ChildRule {
        tag,
        max_occurs: None,
        content,
        type_name,
    }
}

fn schema(type_name: &str) -> Option<&'static ComplexType> {
    SCHEMA
        .iter()
        .find(|(name, _)| *name == type_name)
        .map(|(_, ty)| ty)
}

struct XmlNode {
    namespace: String,
    local: String,
    attrs: Vec<(String, String)>,
    text: String,
    children: Vec<XmlNode>,
    line: Option<u32>,
}

/// Validate GPX XML from a string.
pub fn validate_str(data: &str) -> ValidationResult {
    match parse_xml_tree(data) {
        Ok(root) => {
            let mut validator = Validator::new();
            validator.validate_root(&root);
            ValidationResult {
                issues: validator.issues,
            }
        }
        Err(message) => ValidationResult {
            issues: vec![ValidationIssue {
                severity: Severity::Error,
                message,
                path: "gpx".to_string(),
                line: None,
            }],
        },
    }
}

/// Validate GPX XML from a file path.
pub fn validate_file(path: &Path) -> std::io::Result<ValidationResult> {
    let data = std::fs::read_to_string(path)?;
    Ok(validate_str(&data))
}

fn parse_xml_tree(data: &str) -> Result<XmlNode, String> {
    let mut reader = NsReader::from_str(data);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root: Option<XmlNode> = None;

    loop {
        let line = line_number(data, reader.buffer_position() as usize);
        match reader.read_resolved_event_into(&mut buf) {
            Ok((ns_result, Event::Start(e))) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                let namespace = namespace_from_result(ns_result);
                let attrs = collect_attributes(&reader, &e)?;
                stack.push(XmlNode {
                    namespace,
                    local,
                    attrs,
                    text: String::new(),
                    children: Vec::new(),
                    line: Some(line),
                });
            }
            Ok((ns_result, Event::Empty(e))) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                let namespace = namespace_from_result(ns_result);
                let attrs = collect_attributes(&reader, &e)?;
                let node = XmlNode {
                    namespace,
                    local,
                    attrs,
                    text: String::new(),
                    children: Vec::new(),
                    line: Some(line),
                };
                push_node(node, &mut stack, &mut root);
            }
            Ok((_, Event::End(_))) => {
                let node = stack
                    .pop()
                    .ok_or_else(|| "unexpected end element".to_string())?;
                push_node(node, &mut stack, &mut root);
            }
            Ok((_, Event::Text(e))) => {
                if let Some(node) = stack.last_mut() {
                    let text = e
                        .unescape()
                        .map_err(|err| format!("not well-formed XML: {err}"))?;
                    node.text.push_str(&text);
                }
            }
            Ok((_, Event::CData(e))) => {
                if let Some(node) = stack.last_mut() {
                    let text = e
                        .escape()
                        .map_err(|err| format!("not well-formed XML: {err}"))?;
                    node.text.push_str(std::str::from_utf8(&text).unwrap_or_default());
                }
            }
            Ok((_, Event::Eof)) => break,
            Ok((_, _)) => {}
            Err(err) => {
                return Err(format!("not well-formed XML: {err}"));
            }
        }
        buf.clear();
    }

    root.ok_or_else(|| "not well-formed XML: no root element".to_string())
}

fn push_node(node: XmlNode, stack: &mut [XmlNode], root: &mut Option<XmlNode>) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        *root = Some(node);
    }
}

fn collect_attributes(
    reader: &NsReader<&[u8]>,
    e: &quick_xml::events::BytesStart<'_>,
) -> Result<Vec<(String, String)>, String> {
    let decoder = reader.decoder();
    let mut attrs = Vec::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).into_owned();
        let value = attr
            .decode_and_unescape_value(decoder)
            .map_err(|err| format!("not well-formed XML: {err}"))?
            .into_owned();
        attrs.push((key, value));
    }
    Ok(attrs)
}

fn namespace_from_result(result: ResolveResult<'_>) -> String {
    match result {
        ResolveResult::Bound(namespace) => String::from_utf8_lossy(namespace.0).into_owned(),
        ResolveResult::Unbound | ResolveResult::Unknown(_) => String::new(),
    }
}

fn line_number(source: &str, byte_offset: usize) -> u32 {
    1 + source[..byte_offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
}

struct Validator {
    issues: Vec<ValidationIssue>,
}

impl Validator {
    fn new() -> Self {
        Self {
            issues: Vec::new(),
        }
    }

    fn add(&mut self, severity: Severity, message: impl Into<String>, path: &str, line: Option<u32>) {
        self.issues.push(ValidationIssue {
            severity,
            message: message.into(),
            path: path.to_string(),
            line,
        });
    }

    fn validate_root(&mut self, root: &XmlNode) {
        let line = root.line;

        if root.local != "gpx" {
            self.add(
                Severity::Error,
                format!("root element is <{}>, expected <gpx>", root.local),
                "gpx",
                line,
            );
        }

        if root.namespace != GPX_NAMESPACE {
            if root.namespace == GPX_10_NAMESPACE {
                self.add(
                    Severity::Error,
                    "document uses the GPX 1.0 namespace; only GPX 1.1 is supported",
                    "gpx",
                    line,
                );
            } else if root.namespace.is_empty() {
                self.add(
                    Severity::Error,
                    format!("missing GPX namespace (expected '{GPX_NAMESPACE}')"),
                    "gpx",
                    line,
                );
            } else {
                self.add(
                    Severity::Error,
                    format!(
                        "unexpected namespace '{}' (expected '{GPX_NAMESPACE}')",
                        root.namespace
                    ),
                    "gpx",
                    line,
                );
            }
        }

        self.validate_complex(root, "gpx", "gpx");
    }

    fn validate_complex(&mut self, node: &XmlNode, type_name: &str, path: &str) {
        if type_name == EXTENSIONS {
            self.validate_extensions(node, path);
            return;
        }

        let Some(complex_type) = schema(type_name) else {
            return;
        };

        self.validate_attributes(node, complex_type, path);

        let rules_by_tag: Vec<(&str, usize, &ChildRule)> = complex_type
            .children
            .iter()
            .enumerate()
            .map(|(index, rule)| (rule.tag, index, rule))
            .collect();
        let allowed_tags: Vec<&str> = rules_by_tag.iter().map(|(tag, _, _)| *tag).collect();

        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        let mut last_index: isize = -1;
        let mut last_tag = "";

        for child in &node.children {
            let child_line = child.line;

            if !child.namespace.is_empty()
                && child.namespace != GPX_NAMESPACE
            {
                self.add(
                    Severity::Warning,
                    format!(
                        "foreign-namespace element <{}> outside <extensions>",
                        child.local
                    ),
                    path,
                    child_line,
                );
                continue;
            }

            let Some((index, rule)) = rules_by_tag
                .iter()
                .find(|(tag, _, _)| *tag == child.local)
                .map(|(_, index, rule)| (*index, *rule))
            else {
                self.add(
                    Severity::Error,
                    unknown_element_message(&child.local, &allowed_tags),
                    path,
                    child_line,
                );
                continue;
            };

            let occurrence = counts.entry(rule.tag).or_insert(0);
            let current = *occurrence;
            *occurrence = current + 1;

            if (index as isize) < last_index {
                self.add(
                    Severity::Warning,
                    format!(
                        "<{}> appears after <{}> (out of order per GPX 1.1 schema)",
                        child.local, last_tag
                    ),
                    path,
                    child_line,
                );
            } else {
                last_index = index as isize;
                last_tag = rule.tag;
            }

            let child_path = child_path(path, rule, current);

            if let Some(validator) = rule.content {
                self.validate_content(child, validator, &child_path);
            }
            if let Some(child_type) = rule.type_name {
                self.validate_complex(child, child_type, &child_path);
            }
        }

        self.validate_cardinality(complex_type, &counts, node, path);
    }

    fn validate_extensions(&mut self, node: &XmlNode, path: &str) {
        for child in &node.children {
            let qualifier = if child.namespace == GPX_NAMESPACE {
                Some("the GPX namespace")
            } else if child.namespace.is_empty() {
                Some("no namespace")
            } else {
                None
            };

            if let Some(qualifier) = qualifier {
                self.add(
                    Severity::Warning,
                    format!(
                        "<extensions> child <{}> is in {qualifier} \
                         (extension content must use a foreign namespace per GPX 1.1)",
                        child.local
                    ),
                    &format!("{path} > {}", child.local),
                    child.line,
                );
            }
        }
    }

    fn validate_attributes(&mut self, node: &XmlNode, complex_type: &ComplexType, path: &str) {
        let line = node.line;
        for attr in complex_type.attrs {
            let value = node.attrs.iter().find(|(key, _)| key == attr.name).map(|(_, v)| v);
            let Some(value) = value else {
                self.add(
                    Severity::Error,
                    format!("missing required '{}' attribute", attr.name),
                    path,
                    line,
                );
                continue;
            };

            if let Some(validator) = attr.validator {
                if let Some((severity, message)) = validator(value) {
                    self.add(severity, message, path, line);
                }
            }
        }
    }

    fn validate_content(
        &mut self,
        node: &XmlNode,
        validator: ContentValidator,
        path: &str,
    ) {
        let text = node.text.trim();
        if text.is_empty() {
            return;
        }
        if let Some((severity, message)) = validator(text) {
            self.add(severity, message, path, node.line);
        }
    }

    fn validate_cardinality(
        &mut self,
        complex_type: &ComplexType,
        counts: &std::collections::HashMap<&str, u32>,
        node: &XmlNode,
        path: &str,
    ) {
        let line = node.line;
        for rule in complex_type.children {
            let Some(max_occurs) = rule.max_occurs else {
                continue;
            };
            let count = counts.get(rule.tag).copied().unwrap_or(0);
            if count > max_occurs {
                self.add(
                    Severity::Error,
                    format!(
                        "duplicate <{}> element (at most {max_occurs} allowed per GPX 1.1 schema)",
                        rule.tag
                    ),
                    path,
                    line,
                );
            }
        }
    }
}

fn child_path(path: &str, rule: &ChildRule, occurrence: u32) -> String {
    if rule.max_occurs == Some(1) {
        format!("{path} > {}", rule.tag)
    } else {
        format!("{path} > {}[{occurrence}]", rule.tag)
    }
}

fn unknown_element_message(local: &str, allowed_tags: &[&str]) -> String {
    if let Some(suggestion) = suggest_similar(local, allowed_tags) {
        format!("unknown element <{local}> (did you mean <{suggestion}>?)")
    } else {
        format!("unknown element <{local}>")
    }
}

fn suggest_similar<'a>(unknown: &str, allowed: &'a [&'a str]) -> Option<&'a str> {
    allowed
        .iter()
        .copied()
        .filter(|candidate| {
            let distance = levenshtein(unknown, candidate);
            distance > 0 && distance <= 2
        })
        .min_by_key(|candidate| levenshtein(unknown, candidate))
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, a_ch) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b_chars.iter().enumerate() {
            let cost = usize::from(a_ch != b_ch);
            curr[j + 1] = (prev[j + 1] + 1)
                .min(curr[j] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_chars.len()]
}

fn parse_decimal(text: &str) -> Result<f64, ()> {
    text.parse::<f64>().map_err(|_| ())
}

fn validate_decimal(text: &str) -> Option<(Severity, String)> {
    if parse_decimal(text).is_err() {
        return Some((
            Severity::Error,
            format!("'{text}' is not a valid decimal number"),
        ));
    }
    None
}

fn validate_latitude(text: &str) -> Option<(Severity, String)> {
    let value = match parse_decimal(text) {
        Ok(value) => value,
        Err(()) => {
            return Some((
                Severity::Error,
                format!("invalid latitude '{text}' (not a number)"),
            ));
        }
    };
    if !(-90.0..=90.0).contains(&value) {
        return Some((
            Severity::Error,
            format!("invalid latitude '{text}' (must be in [-90, 90])"),
        ));
    }
    None
}

fn validate_longitude(text: &str) -> Option<(Severity, String)> {
    let value = match parse_decimal(text) {
        Ok(value) => value,
        Err(()) => {
            return Some((
                Severity::Error,
                format!("invalid longitude '{text}' (not a number)"),
            ));
        }
    };
    if !(-180.0..=180.0).contains(&value) {
        return Some((
            Severity::Error,
            format!("invalid longitude '{text}' (must be in [-180, 180])"),
        ));
    }
    if value == 180.0 {
        return Some((
            Severity::Warning,
            format!(
                "longitude {text} equals 180.0 \
                 (the GPX 1.1 schema upper bound is exclusive)"
            ),
        ));
    }
    None
}

fn validate_degrees(text: &str) -> Option<(Severity, String)> {
    let value = match parse_decimal(text) {
        Ok(value) => value,
        Err(()) => {
            return Some((
                Severity::Error,
                format!("invalid degrees value '{text}' (not a number)"),
            ));
        }
    };
    if !(0.0..360.0).contains(&value) {
        return Some((
            Severity::Error,
            format!("invalid degrees value '{text}' (must be in [0, 360))"),
        ));
    }
    None
}

fn validate_fix(text: &str) -> Option<(Severity, String)> {
    if FIX_VALUES.iter().any(|fix| fix_as_str(*fix) == text) {
        return None;
    }
    let allowed = FIX_VALUES
        .iter()
        .map(|fix| fix_as_str(*fix))
        .collect::<Vec<_>>()
        .join(", ");
    Some((
        Severity::Error,
        format!("invalid fix '{text}' (must be one of {allowed})"),
    ))
}

fn validate_dgpsid(text: &str) -> Option<(Severity, String)> {
    let value = match text.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            return Some((
                Severity::Error,
                format!("invalid dgpsid '{text}' (not an integer)"),
            ));
        }
    };
    if !(0..=1023).contains(&value) {
        return Some((
            Severity::Error,
            format!("invalid dgpsid '{text}' (must be in [0, 1023])"),
        ));
    }
    None
}

fn validate_non_negative_int(text: &str) -> Option<(Severity, String)> {
    let value = match text.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            return Some((
                Severity::Error,
                format!("'{text}' is not a valid integer"),
            ));
        }
    };
    if value < 0 {
        return Some((
            Severity::Error,
            format!("'{text}' must be a non-negative integer"),
        ));
    }
    None
}

fn validate_gyear(text: &str) -> Option<(Severity, String)> {
    let re = regex_simple_gyear(text);
    if !re {
        return Some((
            Severity::Error,
            format!("invalid year '{text}' (must be a year, e.g. 2004)"),
        ));
    }
    None
}

fn regex_simple_gyear(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'-') {
        idx = 1;
    }
    if bytes.len() < idx + 4 {
        return false;
    }
    if !bytes[idx..idx + 4].iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    idx += 4;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    if idx == bytes.len() {
        return true;
    }
    if bytes[idx] == b'Z' && idx + 1 == bytes.len() {
        return true;
    }
    if bytes.len() >= idx + 6
        && (bytes[idx] == b'+' || bytes[idx] == b'-')
        && bytes[idx + 3] == b':'
        && bytes[idx + 1..idx + 3].iter().all(|b| b.is_ascii_digit())
        && bytes[idx + 4..idx + 6].iter().all(|b| b.is_ascii_digit())
        && idx + 6 == bytes.len()
    {
        return true;
    }
    false
}

fn validate_time(text: &str) -> Option<(Severity, String)> {
    if chrono::DateTime::parse_from_rfc3339(text).is_ok() {
        if !has_timezone(text) {
            return Some((
                Severity::Warning,
                format!(
                    "time '{text}' has no timezone (GPX timestamps should be in UTC)"
                ),
            ));
        }
        return None;
    }

    if chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f").is_ok() {
        return Some((
            Severity::Warning,
            format!(
                "time '{text}' has no timezone (GPX timestamps should be in UTC)"
            ),
        ));
    }

    Some((
        Severity::Error,
        format!("invalid time '{text}' (must be an ISO 8601 / xsd:dateTime value)"),
    ))
}

fn has_timezone(text: &str) -> bool {
    text.ends_with('Z')
        || text
            .rfind(|c| ['+', '-'].contains(&c))
            .is_some_and(|idx| idx > text.find('T').unwrap_or(0))
}

fn validate_version(text: &str) -> Option<(Severity, String)> {
    if text != "1.1" {
        return Some((
            Severity::Warning,
            format!(
                "version is '{text}' but only '1.1' is supported by this library"
            ),
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_minimal_gpx_passes() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1" creator="test" xmlns="http://www.topografix.com/GPX/1/1"/>"#;
        let result = validate_str(xml);
        assert!(result.is_valid(), "{:?}", result.issues);
    }

    #[test]
    fn missing_namespace_is_error() {
        let xml = r#"<gpx version="1.1" creator="test"/>"#;
        let result = validate_str(xml);
        assert!(!result.is_valid());
        assert!(result
            .errors()
            .iter()
            .any(|issue| issue.message.contains("missing GPX namespace")));
    }

    #[test]
    fn unknown_element_suggestion() {
        let xml = r#"<gpx version="1.1" creator="test" xmlns="http://www.topografix.com/GPX/1/1">
  <metadata><nmae>x</nmae></metadata>
</gpx>"#;
        let result = validate_str(xml);
        assert!(!result.is_valid());
        assert!(result
            .errors()
            .iter()
            .any(|issue| issue.message.contains("did you mean <name>?")));
    }
}
