use std::collections::HashMap;

use chrono::{NaiveDateTime, NaiveTime};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserconfigFile {
    pub chat: Chat,
    pub config: Userconfig,
}

#[derive(Deserialize, Debug)]
pub struct Chat {
    pub id: i64,
    pub first_name: String,
}

#[derive(Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemovedEvents {
    #[default]
    Cancelled,
    Removed,
    Emoji,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDetails {
    #[serde(default)]
    pub alert_minutes_before: Option<u16>,
    #[serde(default)]
    pub changes: HashMap<NaiveDateTime, Change>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Userconfig {
    pub calendarfile_suffix: String,

    pub events: HashMap<String, EventDetails>,

    #[serde(default)]
    pub removed_events: RemovedEvents,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    #[serde(default)]
    pub remove: bool,

    #[serde(default)]
    pub starttime: Option<NaiveTime>,
    #[serde(default)]
    pub endtime: Option<NaiveTime>,

    pub namesuffix: Option<String>,
    pub room: Option<String>,
}

#[test]
fn can_deserialize_chat() -> Result<(), serde_json::Error> {
    let test: Chat = serde_json::from_str(
        r#"{"id": 1337666, "is_bot": false, "first_name": "Peter", "last_name": "Parker", "username": "Spiderman", "language_code": "en"}"#,
    )?;

    assert_eq!(test.id, 1_337_666);
    assert_eq!(test.first_name, "Peter");

    Ok(())
}

#[test]
fn error_on_userconfig_without_calendarfile_suffix() {
    let test: Result<Userconfig, serde_json::Error> = serde_json::from_str(r#"{"events": []}"#);

    let error = test.expect_err("parsing should fail");
    assert!(error.is_data());
}

#[test]
fn can_deserialize_minimal_userconfig() -> Result<(), serde_json::Error> {
    let test: Userconfig =
        serde_json::from_str(r#"{"calendarfileSuffix": "123qwe", "events": {}}"#)?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.events.len(), 0);
    assert_eq!(test.removed_events, RemovedEvents::Cancelled);

    Ok(())
}

#[test]
fn can_deserialize_userconfig_with_event_map() -> Result<(), serde_json::Error> {
    let test: Userconfig = serde_json::from_str(
        r#"{"calendarfileSuffix": "123qwe", "events": {"BTI1-TI": {}, "BTI5-VS": {}}, "removedEvents": "removed"}"#,
    )?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.removed_events, RemovedEvents::Removed);

    let mut events = test.events.keys().collect::<Vec<_>>();
    events.sort();
    assert_eq!(events, ["BTI1-TI", "BTI5-VS"]);

    Ok(())
}

#[cfg(test)]
#[track_caller]
fn deserialize_change(json: &str, expected: &Change) {
    let expected_date = chrono::NaiveDate::from_ymd_opt(2020, 12, 20)
        .unwrap()
        .and_hms_opt(22, 4, 0)
        .unwrap();
    let config = serde_json::from_str::<Userconfig>(
       & r#"{"calendarfileSuffix": "123qwe", "events": {"Fancy Event Name": {"changes": {"2020-12-20T22:04:00": {}}}}}"#.replace("{}", json),
    ).expect("should be able to parse json to userconfig");
    dbg!(&config);
    let actual = config
        .events
        .get("Fancy Event Name")
        .expect("event should exist")
        .changes
        .get(&expected_date)
        .expect("date should exist");
    assert_eq!(actual, expected);
}

#[test]
fn can_deserialize_minimal_change() {
    deserialize_change(
        "{}",
        &Change {
            remove: false,
            starttime: None,
            endtime: None,
            namesuffix: None,
            room: None,
        },
    );
}

#[test]
fn can_deserialize_change_remove() {
    deserialize_change(
        r#"{"remove": true}"#,
        &Change {
            remove: true,
            starttime: None,
            endtime: None,
            namesuffix: None,
            room: None,
        },
    );
}

#[test]
fn can_deserialize_change_endtime() {
    deserialize_change(
        r#"{"endtime": "23:42:00"}"#,
        &Change {
            remove: false,
            starttime: None,
            endtime: Some(NaiveTime::from_hms_opt(23, 42, 0).unwrap()),
            namesuffix: None,
            room: None,
        },
    );
}
