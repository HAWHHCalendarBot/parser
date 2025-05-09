use std::fs;
use std::path::Path;

use anyhow::Context as _;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use crate::generate_ics::{EventStatus, SoonToBeIcsEvent};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    pub start_time: DateTime<FixedOffset>,
    pub end_time: DateTime<FixedOffset>,
}

pub const FOLDER: &str = "eventfiles";

pub fn read(name: &str) -> anyhow::Result<Vec<EventEntry>> {
    let filename = name.replace('/', "-");
    let path = Path::new(FOLDER).join(filename + ".json");
    let content = fs::read_to_string(path).context("failed to read")?;
    let event_entries: Vec<EventEntry> =
        serde_json::from_str(&content).context("failed to parse")?;

    Ok(event_entries)
}

impl From<EventEntry> for SoonToBeIcsEvent {
    fn from(event: EventEntry) -> Self {
        Self {
            start_time: event.start_time.naive_local(),
            end_time: event.end_time.naive_local(),
            name: event.name.clone(),
            pretty_name: event.name,
            status: EventStatus::Confirmed,
            alert_minutes_before: None,
            description: event.description,
            location: event.location,
        }
    }
}

#[test]
fn can_deserialize_event_entry() -> Result<(), serde_json::Error> {
    let test: EventEntry = serde_json::from_str(
        r#"{"Name": "BTI1-TI", "Location": "1060", "Description": "Dozent: HTM", "StartTime": "2022-01-13T11:40:00+01:00", "EndTime": "2022-01-13T12:00:00+01:00"}"#,
    )?;

    assert_eq!(test.name, "BTI1-TI");
    assert_eq!(test.location, "1060");
    assert_eq!(test.description, "Dozent: HTM");
    assert_eq!(
        test.start_time,
        DateTime::parse_from_rfc3339("2022-01-13T11:40:00+01:00").unwrap()
    );
    assert_eq!(
        test.end_time,
        DateTime::parse_from_rfc3339("2022-01-13T12:00:00+01:00").unwrap()
    );

    Ok(())
}
