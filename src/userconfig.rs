use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use serde::de::Visitor;
use serde::Deserialize;

#[allow(clippy::module_name_repetitions)]
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

#[derive(Deserialize, Copy, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RemovedEvents {
    Cancelled,
    Removed,
    Emoji,
}

impl Default for RemovedEvents {
    fn default() -> Self {
        Self::Cancelled
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventDetails {
    pub alert_minutes_before: Option<u16>,
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Userconfig {
    pub calendarfile_suffix: String,

    #[serde(default)]
    pub changes: Vec<Change>,

    #[serde(deserialize_with = "deserialize_events")]
    pub events: HashMap<String, EventDetails>,

    #[serde(default)]
    pub removed_events: RemovedEvents,
}

#[derive(Deserialize, Debug)]
pub struct Change {
    pub name: String,
    pub date: String,

    #[serde(default)]
    pub add: bool,
    #[serde(default)]
    pub remove: bool,

    /// Used when adapting events
    pub starttime: Option<String>,
    /// Used for adapting and creating new events
    pub endtime: Option<String>,

    pub namesuffix: Option<String>,
    pub room: Option<String>,
}

fn deserialize_events<'de, D>(deserializer: D) -> Result<HashMap<String, EventDetails>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct EventArrayOrMap;
    impl<'de> Visitor<'de> for EventArrayOrMap {
        type Value = HashMap<String, EventDetails>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter
                .write_str("a string array or a map with string keys and event details as values")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some(key) = seq.next_element::<String>()? {
                result.insert(key, EventDetails::default());
            }

            Ok(result)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, EventDetails>()? {
                result.insert(key, value);
            }

            Ok(result)
        }
    }

    deserializer.deserialize_any(EventArrayOrMap)
}

pub fn parse_change_date(raw: &str) -> Result<DateTime<FixedOffset>, String> {
    let tless = raw.replace('T', " ");
    let naive = NaiveDateTime::parse_from_str(&tless, "%Y-%m-%d %H:%M")
        .map_err(|err| format!("parse_datetime failed naive {} Error: {}", raw, err))?;
    let date_time = Berlin.from_utc_datetime(&naive);
    let fixed_offset = DateTime::parse_from_rfc3339(&date_time.to_rfc3339())
        .map_err(|err| format!("parse_datetime failed fixed_offset {} Error: {}", raw, err))?;
    Ok(fixed_offset)
}

#[test]
fn can_parse_change_date_from_utc_to_local() {
    let actual = parse_change_date("2020-07-01T06:30").unwrap();
    let string = actual.to_rfc3339();
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
fn can_deserialize_userconfig_with_event_array() -> Result<(), serde_json::Error> {
    let test: Userconfig = serde_json::from_str(
        r#"{"calendarfileSuffix": "123qwe", "changes": [], "events": ["BTI1-TI", "BTI5-VS"], "removedEvents": "removed"}"#,
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
    assert_eq!(test.date, "2020-12-20T22:04");
    assert!(!test.add);
    assert!(!test.remove);
    assert_eq!(test.starttime, None);
    assert_eq!(test.endtime, None);
    assert_eq!(test.namesuffix, None);
    assert_eq!(test.room, None);

    Ok(())
}
