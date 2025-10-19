use std::fs;
use std::path::Path;

use anyhow::Context as _;
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::generate_ics::{EventStatus, SoonToBeIcsEvent};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

pub const FOLDER: &str = "eventfiles";

pub fn read(filename: &str) -> anyhow::Result<Vec<EventEntry>> {
    let filename = filename.replace('/', "-"); // historically the filename was the name with replaced /. Probably not needed anymore
    let mut path = Path::new(FOLDER).join(filename);
    path.set_extension("json");
    let content = fs::read_to_string(path).context("failed to read")?;
    let event_entries: Vec<EventEntry> =
        serde_json::from_str(&content).context("failed to parse")?;

    Ok(event_entries)
}

impl From<EventEntry> for SoonToBeIcsEvent {
    fn from(event: EventEntry) -> Self {
        Self {
            start_time: event.start_time,
            end_time: event.end_time,
            name: event.name,
            status: EventStatus::Confirmed,
            alert_minutes_before: None,
            description: event.description,
            location: event.location,
        }
    }
}

#[test]
fn can_deserialize_event_entry() -> Result<(), serde_json::Error> {
    use chrono::NaiveDate;

    let test: EventEntry = serde_json::from_str(
        r#"{"Name": "BTI1-TI", "Location": "1060", "Description": "Dozent: HTM", "StartTime": "2022-01-13T11:40:00", "EndTime": "2022-01-13T12:00:00"}"#,
    )?;

    assert_eq!(test.name, "BTI1-TI");
    assert_eq!(test.location, "1060");
    assert_eq!(test.description, "Dozent: HTM");
    assert_eq!(
        test.start_time,
        NaiveDate::from_ymd_opt(2022, 1, 13)
            .unwrap()
            .and_hms_opt(11, 40, 0)
            .unwrap()
    );
    assert_eq!(
        test.end_time,
        NaiveDate::from_ymd_opt(2022, 1, 13)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    );

    Ok(())
}
