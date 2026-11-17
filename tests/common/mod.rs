/*
 * SPDX-FileCopyrightText: 2026 Alix Anneraud <alix@anneraud.fr>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Helpers for parse -> write round trips that assert on the raw written text.

#![allow(dead_code)]

use calcard::icalendar::{ICalendar, ICalendarValue};

pub const HEAD: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//fronde//EN\r\nBEGIN:VEVENT\r\n\
                    UID:1\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260101T100000Z\r\n";
pub const TAIL: &str = "END:VEVENT\r\nEND:VCALENDAR\r\n";

pub fn wrap(lines: &[&str]) -> String {
    let mut ics = String::from(HEAD);
    for line in lines {
        ics.push_str(line);
        ics.push_str("\r\n");
    }
    ics.push_str(TAIL);
    ics
}

pub fn parse(ics: &str) -> ICalendar {
    ICalendar::parse(ics).expect("valid iCalendar")
}

/// Raw content lines written for the event properties under test, i.e. the
/// lines between the fixed header and footer, still folded.
pub fn written_lines(written: &str) -> Vec<String> {
    let body = written
        .strip_prefix(HEAD)
        .and_then(|s| s.strip_suffix(TAIL))
        .unwrap_or_else(|| panic!("unexpected envelope:\n{written}"));
    let mut lines: Vec<String> = Vec::new();
    for raw in body.split("\r\n").filter(|l| !l.is_empty()) {
        if let Some(cont) = raw.strip_prefix(' ') {
            lines.last_mut().unwrap().push_str(cont);
        } else {
            lines.push(raw.to_string());
        }
    }
    lines
}

/// Parses `input` lines, writes them back and checks the raw output lines
/// against `expected`. Also checks that a second round trip is stable.
#[track_caller]
pub fn assert_round_trip(input: &[&str], expected: &[&str]) -> ICalendar {
    let ical = parse(&wrap(input));
    let written = ical.to_string();
    assert_eq!(written_lines(&written), expected, "raw output:\n{written}");
    let again = parse(&written).to_string();
    assert_eq!(again, written, "second round trip is not stable");
    assert_archived_writer_matches(&ical);
    ical
}

/// The archived (rkyv) writer must produce the same text as the owned one.
#[cfg(feature = "rkyv")]
pub fn assert_archived_writer_matches(ical: &ICalendar) {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(ical).expect("archive");
    let archived =
        rkyv::access::<calcard::icalendar::ArchivedICalendar, rkyv::rancor::Error>(&bytes)
            .expect("access archive");
    assert_eq!(
        archived.to_string(),
        ical.to_string(),
        "archived writer differs"
    );
}

#[cfg(not(feature = "rkyv"))]
pub fn assert_archived_writer_matches(_ical: &ICalendar) {}

pub fn event_values(ical: &ICalendar, name: &str) -> Vec<Vec<ICalendarValue>> {
    ical.components[1]
        .entries
        .iter()
        .filter(|e| e.name.as_str().eq_ignore_ascii_case(name))
        .map(|e| e.values.clone())
        .collect()
}

pub fn assert_folded(written: &str) {
    for line in written.split("\r\n") {
        assert!(line.len() <= 75, "line exceeds 75 octets: {line:?}");
    }
}
