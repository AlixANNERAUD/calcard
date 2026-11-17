/*
 * SPDX-FileCopyrightText: 2026 Alix Anneraud <alix@anneraud.fr>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Backslash escapes in TEXT values.

mod common;

use calcard::icalendar::ICalendarValue;
use common::*;

#[test]
fn text_known_escapes() {
    let ical = assert_round_trip(
        &["SUMMARY:a\\,b\\;c\\\\d\\ne\\Nf"],
        &["SUMMARY:a\\,b\\;c\\\\d\\ne\\nf"],
    );
    assert_eq!(
        event_values(&ical, "SUMMARY")[0],
        [ICalendarValue::Text("a,b;c\\d\ne\nf".into())]
    );
}

#[test]
fn text_unknown_escape_is_literal() {
    let ical = assert_round_trip(
        &["SUMMARY:C:\\x", "LOCATION:trailing\\"],
        &["SUMMARY:C:\\\\x", "LOCATION:trailing\\\\"],
    );
    assert_eq!(
        event_values(&ical, "SUMMARY")[0],
        [ICalendarValue::Text("C:\\x".into())]
    );
    assert_eq!(
        event_values(&ical, "LOCATION")[0],
        [ICalendarValue::Text("trailing\\".into())]
    );
}

#[test]
fn vcard_unknown_escape_is_literal() {
    // The tokenizer is shared with the vCard parser (RFC 6350, Section 3.4).
    let input = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nNOTE:C:\\path\\, ok\r\nEND:VCARD\r\n";
    let written = calcard::vcard::VCard::parse(input).unwrap().to_string();
    assert!(
        written.contains("\r\nNOTE:C:\\\\path\\, ok\r\n"),
        "raw output:\n{written}"
    );
}
