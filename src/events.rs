use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Context as _;
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::generate_ics::{EventStatus, SoonToBeIcsEvent};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

pub const FOLDER: &str = "eventfiles";

pub fn pull() {
    if Path::new(FOLDER).join(".git").exists() {
        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(FOLDER)
            .status()
            .expect("git process should execute");
        assert!(status.success(), "git pull status code {status}");
    } else {
        let status = Command::new("git")
            .args([
                "clone",
                "-q",
                "--depth",
                "1",
                "https://github.com/HAWHHCalendarBot/eventfiles.git",
                FOLDER,
            ])
            .status()
            .expect("git process should execute");
        assert!(status.success(), "git clone status code {status}");
    }
}

pub fn read(filename: &str) -> anyhow::Result<Vec<EventEntry>> {
    let mut path = Path::new(FOLDER).join("events").join(filename);
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
        r#"{"name": "BTI1-TI", "location": "1060", "description": "Dozent: HTM", "startTime": "2022-01-13T11:40:00", "endTime": "2022-01-13T12:00:00"}"#,
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
