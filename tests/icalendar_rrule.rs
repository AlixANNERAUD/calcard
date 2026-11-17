/*
 * SPDX-FileCopyrightText: 2026 Alix Anneraud <alix@anneraud.fr>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! RRULE parts that are not modelled survive a round trip.

mod common;

use calcard::icalendar::ICalendarValue;
use common::*;

fn other_parts(values: &[ICalendarValue]) -> Vec<(String, String)> {
    match values {
        [ICalendarValue::RecurrenceRule(rule)] => rule.other_parts.clone(),
        other => panic!("expected a recurrence rule, got {other:?}"),
    }
}

#[test]
fn rrule_x_part_is_kept() {
    let ical = assert_round_trip(
        &["RRULE:FREQ=WEEKLY;X-NAME=1;BYDAY=MO"],
        &["RRULE:FREQ=WEEKLY;BYDAY=MO;X-NAME=1"],
    );
    assert_eq!(
        other_parts(&event_values(&ical, "RRULE")[0]),
        [("X-NAME".to_string(), "1".to_string())]
    );
}

#[test]
fn rrule_unknown_parts_keep_their_raw_value_and_order() {
    let ical = assert_round_trip(
        &["RRULE:X-FIRST=a,b;FREQ=MONTHLY;BYEASTER=-2;x-lower=c=d;BYMONTHDAY=1;X-EMPTY="],
        &["RRULE:FREQ=MONTHLY;BYMONTHDAY=1;X-FIRST=a,b;BYEASTER=-2;x-lower=c=d;X-EMPTY="],
    );
    assert_eq!(
        other_parts(&event_values(&ical, "RRULE")[0]),
        [
            ("X-FIRST".to_string(), "a,b".to_string()),
            ("BYEASTER".to_string(), "-2".to_string()),
            ("x-lower".to_string(), "c=d".to_string()),
            ("X-EMPTY".to_string(), String::new()),
        ]
    );
}

#[test]
fn rrule_unknown_part_value_is_unfolded() {
    let ical = parse(&wrap(&[
        "RRULE:FREQ=DAILY;COUNT=3;X-LONG=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n bbb",
    ]));
    assert_eq!(
        other_parts(&event_values(&ical, "RRULE")[0]),
        [(
            "X-LONG".to_string(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbb".to_string()
        )]
    );
    let written = ical.to_string();
    assert_eq!(
        written_lines(&written),
        [
            "RRULE:FREQ=DAILY;COUNT=3;X-LONG=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbb"
        ]
    );
    assert_eq!(parse(&written).to_string(), written);
}

#[test]
fn rrule_known_parts_unchanged() {
    assert_round_trip(
        &["RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU;UNTIL=20271231T000000Z"],
        &["RRULE:FREQ=YEARLY;UNTIL=20271231T000000Z;BYDAY=-1SU;BYMONTH=3"],
    );
}

#[cfg(feature = "serde")]
#[test]
fn rrule_serde_other_parts() {
    use calcard::icalendar::ICalendarRecurrenceRule;

    let rule = ICalendarRecurrenceRule {
        other_parts: vec![("X-NAME".into(), "1".into())],
        ..Default::default()
    };
    let json = serde_json::to_string(&rule).unwrap();
    assert!(
        json.contains("\"other_parts\":[[\"X-NAME\",\"1\"]]"),
        "{json}"
    );
    assert_eq!(
        serde_json::from_str::<ICalendarRecurrenceRule>(&json).unwrap(),
        rule
    );

    // Data serialized before the field existed still deserializes.
    let mut value = serde_json::to_value(ICalendarRecurrenceRule::default()).unwrap();
    assert!(value.get("other_parts").is_none());
    value.as_object_mut().unwrap().remove("other_parts");
    assert_eq!(
        serde_json::from_value::<ICalendarRecurrenceRule>(value).unwrap(),
        ICalendarRecurrenceRule::default()
    );
}
