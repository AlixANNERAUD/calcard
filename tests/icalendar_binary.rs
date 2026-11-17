/*
 * SPDX-FileCopyrightText: 2026 Alix Anneraud <alix@anneraud.fr>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Inline binary, CHARSET, ENCODING and QUOTED-PRINTABLE round trips.

mod common;

use calcard::icalendar::{ICalendarProperty, ICalendarValue};
use common::*;

#[test]
fn binary_attach_valid_utf8() {
    let ical = assert_round_trip(
        &["ATTACH;ENCODING=BASE64;VALUE=BINARY:SGVsbG8="],
        &["ATTACH;ENCODING=BASE64;VALUE=BINARY:SGVsbG8="],
    );
    assert_eq!(
        event_values(&ical, "ATTACH")[0],
        [ICalendarValue::Binary(b"Hello".to_vec())]
    );
}

#[test]
fn binary_attach_without_value_parameter() {
    assert_round_trip(
        &[
            "ATTACH;FMTTYPE=text/plain;ENCODING=BASE64:SGVsbG8=",
            "ATTACH;BASE64:SGVsbG8=",
        ],
        &[
            "ATTACH;ENCODING=BASE64;FMTTYPE=text/plain;VALUE=BINARY:SGVsbG8=",
            "ATTACH;ENCODING=BASE64;VALUE=BINARY:SGVsbG8=",
        ],
    );
}

#[test]
fn binary_not_utf8() {
    let ical = assert_round_trip(
        &["ATTACH;ENCODING=BASE64;VALUE=BINARY:/9j/4AAQSkZJRg=="],
        &["ATTACH;ENCODING=BASE64;VALUE=BINARY:/9j/4AAQSkZJRg=="],
    );
    assert_eq!(
        event_values(&ical, "ATTACH")[0],
        [ICalendarValue::Binary(vec![
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46
        ])]
    );
}

#[test]
fn binary_image() {
    assert_round_trip(
        &["IMAGE;VALUE=BINARY;ENCODING=BASE64;FMTTYPE=image/png:iVBORw0KGgo="],
        &["IMAGE;ENCODING=BASE64;FMTTYPE=image/png;VALUE=BINARY:iVBORw0KGgo="],
    );
}

#[test]
fn binary_x_property_with_value_binary() {
    assert_round_trip(
        &["X-BLOB;ENCODING=BASE64;VALUE=BINARY:SGVsbG8="],
        &["X-BLOB;ENCODING=BASE64;VALUE=BINARY:SGVsbG8="],
    );
}

#[test]
fn long_binary_is_folded() {
    let bytes: Vec<u8> = (0u8..=255).cycle().take(300).collect();
    let mut ical = parse(&wrap(&["ATTACH;ENCODING=BASE64;VALUE=BINARY:AA=="]));
    let entry = ical.components[1]
        .entries
        .iter_mut()
        .find(|e| e.name == ICalendarProperty::Attach)
        .unwrap();
    entry.values = vec![ICalendarValue::Binary(bytes.clone())];
    let written = ical.to_string();
    assert_folded(&written);
    let reparsed = parse(&written);
    assert_eq!(
        event_values(&reparsed, "ATTACH")[0],
        [ICalendarValue::Binary(bytes)]
    );
    assert_eq!(reparsed.to_string(), written);
}

#[test]
fn base64_text_property_is_decoded_to_text() {
    // ENCODING=BASE64 on a TEXT property is a transfer encoding: the text is
    // decoded and written as plain TEXT, which reads back to the same bytes.
    let ical = assert_round_trip(
        &["DESCRIPTION;ENCODING=BASE64:SGVsbG8sIHdvcmxkOwo="],
        &["DESCRIPTION:Hello\\, world\\;\\n"],
    );
    assert_eq!(
        event_values(&ical, "DESCRIPTION")[0],
        [ICalendarValue::Text("Hello, world;\n".into())]
    );
}

#[test]
fn base64_text_property_not_utf8_stays_binary() {
    assert_round_trip(
        &["X-FOO;ENCODING=BASE64:/w=="],
        &["X-FOO;ENCODING=BASE64:/w=="],
    );
}

#[test]
fn base64_invalid_keeps_parameter() {
    assert_round_trip(
        &["DESCRIPTION;ENCODING=BASE64:***"],
        &["DESCRIPTION;ENCODING=BASE64:***"],
    );
}

// --- CHARSET / ENCODING / QUOTED-PRINTABLE ---

#[test]
fn charset_without_encoding_is_kept() {
    assert_round_trip(
        &["SUMMARY;CHARSET=UTF-8:Café"],
        &["SUMMARY;CHARSET=UTF-8:Café"],
    );
}

#[test]
fn unknown_encoding_is_kept() {
    assert_round_trip(
        &["DESCRIPTION;ENCODING=8BIT:hello"],
        &["DESCRIPTION;ENCODING=8BIT:hello"],
    );
}

#[test]
fn quoted_printable_without_charset() {
    let ical = assert_round_trip(
        &[
            "SUMMARY;ENCODING=QUOTED-PRINTABLE:Caf=C3=A9 cr=C3=A8me",
            "LOCATION;ENCODING=QUOTED-PRINTABLE:Caf=E9",
        ],
        &["SUMMARY:Café crème", "LOCATION:Café"],
    );
    assert_eq!(
        event_values(&ical, "SUMMARY")[0],
        [ICalendarValue::Text("Café crème".into())]
    );
}

#[test]
fn quoted_printable_soft_line_break() {
    assert_round_trip(
        &["DESCRIPTION;ENCODING=QUOTED-PRINTABLE:Caf=C3=\r\n=A9 au lait"],
        &["DESCRIPTION:Café au lait"],
    );
}

#[test]
fn quoted_printable_with_known_charset() {
    assert_round_trip(
        &["SUMMARY;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:Caf=E9"],
        &["SUMMARY:Café"],
    );
}

#[test]
fn quoted_printable_with_unknown_charset_is_kept_raw() {
    assert_round_trip(
        &["SUMMARY;CHARSET=X-UNKNOWN;ENCODING=QUOTED-PRINTABLE:Caf=E9"],
        &["SUMMARY;CHARSET=X-UNKNOWN;ENCODING=QUOTED-PRINTABLE:Caf=E9"],
    );
}

#[test]
fn quoted_printable_invalid_is_kept_raw() {
    assert_round_trip(
        &["SUMMARY;ENCODING=QUOTED-PRINTABLE:a=ZZ"],
        &["SUMMARY;ENCODING=QUOTED-PRINTABLE:a=ZZ"],
    );
}
