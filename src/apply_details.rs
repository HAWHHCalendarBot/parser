use crate::generate_ics::SoonToBeIcsEvent;
use crate::userconfig::EventDetails;

pub fn apply_details(event: &mut SoonToBeIcsEvent, details: &EventDetails) {
    event.alert_minutes_before = details.alert_minutes_before;

    if let Some(notes) = &details.notes
        && !notes.is_empty()
    {
        event.description = if event.description.is_empty() {
            notes.clone()
        } else {
            format!("{}\n\n{notes}", event.description)
        };
    }
}

#[cfg(test)]
fn create_event(description: &str) -> SoonToBeIcsEvent {
    SoonToBeIcsEvent {
        name: "BTI5-VSP/01".to_owned(),
        pretty_name: "BTI5-VSP/01".to_owned(),
        status: crate::generate_ics::EventStatus::Confirmed,
        start_time: chrono::NaiveDate::from_ymd_opt(2020, 4, 2)
            .unwrap()
            .and_hms_opt(8, 15, 0)
            .unwrap(),
        end_time: chrono::NaiveDate::from_ymd_opt(2020, 4, 2)
            .unwrap()
            .and_hms_opt(11, 15, 0)
            .unwrap(),
        alert_minutes_before: None,
        description: description.to_owned(),
        location: String::new(),
    }
}

#[cfg(test)]
fn check_alert(alert_minutes_before: Option<u16>) {
    let details = EventDetails {
        alert_minutes_before,
        ..EventDetails::default()
    };
    let mut event = create_event("");
    apply_details(&mut event, &details);
    assert_eq!(event.alert_minutes_before, alert_minutes_before);
}

#[test]
fn alert_examples() {
    check_alert(None);
    check_alert(Some(10));
    check_alert(Some(30));
    check_alert(Some(60));
}

#[cfg(test)]
fn check_description(notes: Option<&str>, event_description: &str, expected: &str) {
    let details = EventDetails {
        notes: notes.map(ToOwned::to_owned),
        ..EventDetails::default()
    };
    let mut event = create_event(event_description);
    apply_details(&mut event, &details);
    assert_eq!(event.description, expected);
}

#[test]
fn no_note_no_description() {
    check_description(None, "", "");
}

#[test]
fn no_note_some_description() {
    check_description(None, "bla", "bla");
}

#[test]
fn some_note_no_description() {
    check_description(Some("bla"), "", "bla");
}

#[test]
fn some_note_some_description() {
    check_description(Some("foo"), "bar", "bar\n\nfoo");
}
