/*
 * SPDX-FileCopyrightText: 2026 Alix Anneraud <alix@anneraud.fr>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! URI and CAL-ADDRESS values round-trip verbatim (RFC 5545, Section 3.3.13).

mod common;

use calcard::icalendar::{ICalendarParameterValue, ICalendarProperty, ICalendarValue, Uri};
use common::*;

#[test]
fn uri_url_with_comma_and_semicolon() {
    assert_round_trip(
        &["URL:https://x.example/a,b;c"],
        &["URL:https://x.example/a,b;c"],
    );
}

#[test]
fn uri_data_attach() {
    let ical = assert_round_trip(
        &["ATTACH:data:text/plain;base64,SGVsbG8="],
        &["ATTACH:data:text/plain;base64,SGVsbG8="],
    );
    assert!(matches!(
        &event_values(&ical, "ATTACH")[0][0],
        ICalendarValue::Uri(Uri::Data(d)) if d.data == b"Hello"
    ));
}

#[test]
fn uri_data_non_canonical_is_kept_verbatim() {
    assert_round_trip(
        &[
            "ATTACH:data:text/plain;charset=utf-8;base64,SGVsbG8=",
            "ATTACH:data:,Hello%20World",
        ],
        &[
            "ATTACH:data:text/plain;charset=utf-8;base64,SGVsbG8=",
            "ATTACH:data:,Hello%20World",
        ],
    );
}

#[test]
fn uri_geo_in_x_property() {
    assert_round_trip(
        &["X-APPLE-STRUCTURED-LOCATION;VALUE=URI;X-TITLE=Home:geo:48.85,2.35"],
        &["X-APPLE-STRUCTURED-LOCATION;X-TITLE=Home;VALUE=URI:geo:48.85,2.35"],
    );
}

#[test]
fn uri_geo_with_parameters() {
    assert_round_trip(
        &["URL:geo:37.786971,-122.399677;u=35"],
        &["URL:geo:37.786971,-122.399677;u=35"],
    );
}

#[test]
fn cal_address_is_written_as_is() {
    assert_round_trip(
        &[
            "ORGANIZER;CN=Org:mailto:a,b@x.example",
            "ATTENDEE;VALUE=CAL-ADDRESS:mailto:c;d@x.example",
        ],
        &[
            "ORGANIZER;CN=Org:mailto:a,b@x.example",
            "ATTENDEE;VALUE=CAL-ADDRESS:mailto:c;d@x.example",
        ],
    );
}

#[test]
fn uri_backslashes_are_literal() {
    let ical = assert_round_trip(
        &["URL:file:///C:\\x", "URL:https://x.example/a\\,b\\;c"],
        &["URL:file:///C:\\x", "URL:https://x.example/a\\,b\\;c"],
    );
    let values = event_values(&ical, "URL");
    assert_eq!(
        values[0],
        [ICalendarValue::Uri(Uri::Location("file:///C:\\x".into()))]
    );
    assert_eq!(
        values[1],
        [ICalendarValue::Uri(Uri::Location(
            "https://x.example/a\\,b\\;c".into()
        ))]
    );
}

#[test]
fn uri_x_property_is_single_valued() {
    let ical = assert_round_trip(
        &[
            "X-LINKS;VALUE=URI:https://a.example/1;https://b.example/2,3",
            "X-ESCAPED;VALUE=URI:https://a.example/x\\;y",
        ],
        &[
            "X-LINKS;VALUE=URI:https://a.example/1;https://b.example/2,3",
            "X-ESCAPED;VALUE=URI:https://a.example/x\\;y",
        ],
    );
    assert_eq!(
        event_values(&ical, "X-LINKS")[0],
        [ICalendarValue::Uri(Uri::Location(
            "https://a.example/1;https://b.example/2,3".into()
        ))]
    );
    assert_eq!(
        event_values(&ical, "X-ESCAPED")[0],
        [ICalendarValue::Uri(Uri::Location(
            "https://a.example/x\\;y".into()
        ))]
    );
}

#[test]
fn x_property_text_is_still_multi_valued() {
    let ical = assert_round_trip(&["X-PARTS:a;b\\;c"], &["X-PARTS:a;b\\;c"]);
    assert_eq!(
        event_values(&ical, "X-PARTS")[0],
        [
            ICalendarValue::Text("a".into()),
            ICalendarValue::Text("b;c".into())
        ]
    );
}

#[test]
fn long_uri_is_folded_and_restored() {
    let uri = format!(
        "https://x.example/{}?q=a,b;c&r={}",
        "segment/".repeat(20),
        "z".repeat(40)
    );
    let line = format!("URL:{uri}");
    let ical = parse(&wrap(&[&line]));
    let written = ical.to_string();
    assert_folded(&written);
    assert!(written.contains("\r\n "), "expected a fold:\n{written}");
    assert_eq!(written_lines(&written), [line.as_str()]);
    let reparsed = parse(&written);
    assert_eq!(
        event_values(&reparsed, "URL")[0],
        [ICalendarValue::Uri(Uri::Location(uri))]
    );
}

#[test]
fn uri_line_breaks_are_percent_encoded() {
    let mut ical = parse(&wrap(&["URL:https://x.example/"]));
    let entry = ical.components[1]
        .entries
        .iter_mut()
        .find(|e| e.name == ICalendarProperty::Url)
        .unwrap();
    entry.values = vec![ICalendarValue::Uri(Uri::Location(
        "https://x.example/a\r\nb\tc\u{1}d".into(),
    ))];
    let written = ical.to_string();
    assert_eq!(
        written_lines(&written),
        ["URL:https://x.example/a%0D%0Ab\tc\u{1}d"]
    );
}

// --- URI parameter values follow the same rule ---

#[test]
fn uri_parameter_data_non_canonical_is_kept_verbatim() {
    assert_round_trip(
        &["DESCRIPTION;ALTREP=\"data:text/plain;charset=iso-8859-1,caf%E9\":Café"],
        &["DESCRIPTION;ALTREP=\"data:text/plain;charset=iso-8859-1,caf%E9\":Café"],
    );
}

#[test]
fn uri_parameter_data_canonical_is_decoded() {
    let ical = assert_round_trip(
        &["DESCRIPTION;ALTREP=\"data:text/plain;base64,SGVsbG8=\":Hello"],
        &["DESCRIPTION;ALTREP=\"data:text/plain;base64,SGVsbG8=\":Hello"],
    );
    let entry = ical.components[1]
        .entries
        .iter()
        .find(|e| e.name == ICalendarProperty::Description)
        .unwrap();
    assert!(matches!(
        &entry.params[0].value,
        ICalendarParameterValue::Uri(Uri::Data(d)) if d.data == b"Hello"
    ));
}

#[test]
fn uri_parameter_location() {
    assert_round_trip(
        &[
            "ATTENDEE;SENT-BY=\"mailto:a,b@x.example\";DELEGATED-FROM=\"mailto:c@x.example\":mailto:d@x.example",
        ],
        &[
            "ATTENDEE;SENT-BY=\"mailto:a,b@x.example\";DELEGATED-FROM=\"mailto:c@x.example\":mailto:d@x.example",
        ],
    );
}
