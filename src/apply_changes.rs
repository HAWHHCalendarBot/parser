#![expect(clippy::non_ascii_literal)]

use chrono::NaiveDateTime;

use crate::generate_ics::{EventStatus, SoonToBeIcsEvent};
use crate::userconfig::{Change, RemovedEvents};

pub fn apply_change(
    events: &mut Vec<SoonToBeIcsEvent>,
    date: NaiveDateTime,
    change: Change,
    removed_events: RemovedEvents,
) {
    if let Some(i) = events.iter().position(|event| event.start_time == date) {
        let event = &mut events[i];
        if change.remove {
            match removed_events {
                RemovedEvents::Cancelled => event.status = EventStatus::Cancelled,
                RemovedEvents::Emoji => event.pretty_name.insert_str(0, "🚫 "),
                RemovedEvents::Removed => {
                    events.remove(i);
                    return;
                }
            }
        }

        if let Some(namesuffix) = &change.namesuffix {
            event.pretty_name += " ";
            event.pretty_name += namesuffix;
        }

        if let Some(room) = change.room {
            event.location = room;
        }

        if let Some(time) = change.starttime {
            event.start_time = date.date().and_time(time);
        }

        if let Some(time) = change.endtime {
            event.end_time = date.date().and_time(time);
        }
    } else {
        // Event for this change doesnt exist.
        // This not nice but the TelegramBot has to solve this via user feedback.
    }
}

#[cfg(test)]
fn generate_events() -> Vec<SoonToBeIcsEvent> {
    vec![
        SoonToBeIcsEvent {
            name: "BTI5-VSP/01".to_owned(),
            pretty_name: "BTI5-VSP/01".to_owned(),
            status: EventStatus::Confirmed,
            start_time: chrono::NaiveDate::from_ymd_opt(2020, 4, 2)
                .unwrap()
                .and_hms_opt(8, 15, 0)
                .unwrap(),
            end_time: chrono::NaiveDate::from_ymd_opt(2020, 4, 2)
                .unwrap()
                .and_hms_opt(11, 15, 0)
                .unwrap(),
            alert_minutes_before: None,
            description: String::new(),
            location: String::new(),
        },
        SoonToBeIcsEvent {
            name: "BTI5-VSP/01".to_owned(),
            pretty_name: "BTI5-VSP/01".to_owned(),
            status: EventStatus::Confirmed,
            start_time: chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
                .unwrap()
                .and_hms_opt(8, 15, 0)
                .unwrap(),
            end_time: chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
                .unwrap()
                .and_hms_opt(11, 15, 0)
                .unwrap(),
            alert_minutes_before: None,
            description: String::new(),
            location: String::new(),
        },
    ]
}

#[test]
fn non_existing_event_of_change_is_skipped() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(13, 37, 0)
        .unwrap();
    let change = Change {
        remove: true,
        starttime: None,
        endtime: None,
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(events.len(), 2);

    let expected = generate_events();
    assert_eq!(events[0], expected[0]);
    assert_eq!(events[1], expected[1]);
}

#[test]
fn remove_event_is_removed_completly() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: true,
        starttime: None,
        endtime: None,
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Removed);
    assert_eq!(events.len(), 1);
}

#[test]
fn remove_event_gets_marked_as_cancelled() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: true,
        starttime: None,
        endtime: None,
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].status, EventStatus::Cancelled);
}

#[test]
fn remove_event_gets_emoji_prefix() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: true,
        starttime: None,
        endtime: None,
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Emoji);
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].pretty_name, "🚫 BTI5-VSP/01");
}

#[test]
fn namesuffix_is_added() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: false,
        starttime: None,
        endtime: None,
        namesuffix: Some("whatever".to_owned()),
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(events[1].pretty_name, "BTI5-VSP/01 whatever");
}

#[test]
fn room_is_overwritten() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: false,
        starttime: None,
        endtime: None,
        namesuffix: None,
        room: Some("whereever".to_owned()),
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(events[1].location, "whereever");
}

#[test]
fn starttime_changed() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: false,
        starttime: Some(chrono::NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
        endtime: None,
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(
        events[1].start_time,
        chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
            .unwrap()
            .and_hms_opt(8, 30, 0)
            .unwrap()
    );
}

#[test]
fn endtime_changed() {
    let mut events = generate_events();
    let date = chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
        .unwrap()
        .and_hms_opt(8, 15, 0)
        .unwrap();
    let change = Change {
        remove: false,
        starttime: None,
        endtime: Some(chrono::NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
        namesuffix: None,
        room: None,
    };
    apply_change(&mut events, date, change, RemovedEvents::Cancelled);
    assert_eq!(
        events[1].end_time,
        chrono::NaiveDate::from_ymd_opt(2020, 5, 14)
            .unwrap()
            .and_hms_opt(8, 30, 0)
            .unwrap()
    );
}
