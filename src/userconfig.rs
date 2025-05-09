use std::collections::HashMap;

use chrono::{NaiveDateTime, NaiveTime, TimeZone as _};
use chrono_tz::Europe::Berlin;
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventDetails {
    #[serde(default)]
    pub alert_minutes_before: Option<u16>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Userconfig {
    pub calendarfile_suffix: String,

    #[serde(default)]
    pub changes: Vec<Change>,

    pub events: HashMap<String, EventDetails>,

    #[serde(default)]
    pub removed_events: RemovedEvents,
}

#[derive(Deserialize, Debug)]
pub struct Change {
    pub name: String,

    #[serde(deserialize_with = "deserialize_change_date")]
    pub date: NaiveDateTime,

    #[serde(default)]
    pub add: bool,
    #[serde(default)]
    pub remove: bool,

    #[serde(default, deserialize_with = "deserialize_change_time")]
    /// Used when adapting events
    pub starttime: Option<NaiveTime>,
    #[serde(default, deserialize_with = "deserialize_change_time")]
    /// Used for adapting and creating new events
    pub endtime: Option<NaiveTime>,

    pub namesuffix: Option<String>,
    pub room: Option<String>,
}

fn deserialize_change_time<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    let time = NaiveTime::parse_from_str(&str, "%H:%M").map_err(serde::de::Error::custom)?;
    Ok(Some(time))
}

fn deserialize_change_date<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    parse_change_date(&raw).map_err(serde::de::Error::custom)
}

fn parse_change_date(raw: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    let tless = raw.replace('T', " ");
    let utc = NaiveDateTime::parse_from_str(&tless, "%Y-%m-%d %H:%M")?;
    let date_time = Berlin.from_utc_datetime(&utc);
    let naive = date_time.naive_local();
    Ok(naive)
}

#[test]
fn can_parse_change_date_from_utc_to_local() {
    let actual = parse_change_date("2020-07-01T06:30").unwrap();
    let string = actual.and_local_timezone(Berlin).unwrap().to_rfc3339();
    assert_eq!(string, "2020-07-01T08:30:00+02:00");
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
    let test: Result<Userconfig, serde_json::Error> =
        serde_json::from_str(r#"{"changes": [], "events": []}"#);

    let error = test.expect_err("parsing should fail");
    assert!(error.is_data());
}

#[test]
fn can_deserialize_minimal_userconfig() -> Result<(), serde_json::Error> {
    let test: Userconfig =
        serde_json::from_str(r#"{"calendarfileSuffix": "123qwe", "changes": [], "events": {}}"#)?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.changes.len(), 0);
    assert_eq!(test.events.len(), 0);
    assert_eq!(test.removed_events, RemovedEvents::Cancelled);

    Ok(())
}

#[test]
fn can_deserialize_userconfig_with_event_map() -> Result<(), serde_json::Error> {
    let test: Userconfig = serde_json::from_str(
        r#"{"calendarfileSuffix": "123qwe", "changes": [], "events": {"BTI1-TI": {}, "BTI5-VS": {}}, "removedEvents": "removed"}"#,
    )?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.changes.len(), 0);
    assert_eq!(test.removed_events, RemovedEvents::Removed);

    let mut events = test.events.keys().collect::<Vec<_>>();
    events.sort();
    assert_eq!(events, ["BTI1-TI", "BTI5-VS"]);

    Ok(())
}

#[test]
fn can_deserialize_minimal_change() -> Result<(), serde_json::Error> {
    let test: Change = serde_json::from_str(r#"{"name": "Tree", "date": "2020-12-20T22:04"}"#)?;
    assert_eq!(test.name, "Tree");
    assert_eq!(
        test.date,
        chrono::NaiveDate::from_ymd_opt(2020, 12, 20)
            .unwrap()
            .and_hms_opt(23, 4, 0)
            .unwrap()
    );
    assert!(!test.add);
    assert!(!test.remove);
    assert_eq!(test.starttime, None);
    assert_eq!(test.endtime, None);
    assert_eq!(test.namesuffix, None);
    assert_eq!(test.room, None);
    Ok(())
}

#[test]
fn can_deserialize_change_remove() -> Result<(), serde_json::Error> {
    let test: Change =
        serde_json::from_str(r#"{"name": "Tree", "date": "2020-12-20T22:04", "remove": true}"#)?;
    assert_eq!(test.name, "Tree");
    assert_eq!(
        test.date,
        chrono::NaiveDate::from_ymd_opt(2020, 12, 20)
            .unwrap()
            .and_hms_opt(23, 4, 0)
            .unwrap()
    );
    assert!(!test.add);
    assert!(test.remove);
    assert_eq!(test.starttime, None);
    assert_eq!(test.endtime, None);
    assert_eq!(test.namesuffix, None);
    assert_eq!(test.room, None);
    Ok(())
}

#[test]
fn can_deserialize_change_add() -> Result<(), serde_json::Error> {
    let test: Change = serde_json::from_str(
        r#"{"name": "Tree", "date": "2020-12-20T22:04", "add": true, "endtime": "23:42"}"#,
    )?;
    assert_eq!(test.name, "Tree");
    assert_eq!(
        test.date,
        chrono::NaiveDate::from_ymd_opt(2020, 12, 20)
            .unwrap()
            .and_hms_opt(23, 4, 0)
            .unwrap()
    );
    assert!(test.add);
    assert!(!test.remove);
    assert_eq!(test.starttime, None);
    assert_eq!(
        test.endtime,
        Some(NaiveTime::from_hms_opt(23, 42, 0).unwrap())
    );
    assert_eq!(test.namesuffix, None);
    assert_eq!(test.room, None);
    Ok(())
}
