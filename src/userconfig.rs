use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UserconfigFile {
    pub chat: Chat,
    pub config: Userconfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chat {
    pub id: i64,
    pub first_name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Userconfig {
    pub calendarfile_suffix: String,

    #[serde(default)]
    pub changes: Vec<Change>,

    pub events: Vec<String>,

    #[serde(default)]
    pub removed_events: RemovedEvents,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Change {
    pub add: Option<bool>,
    pub name: String,
    pub date: String,
    pub remove: Option<bool>,
    pub namesuffix: Option<String>,
    pub starttime: Option<String>,
    pub endtime: Option<String>,
    pub room: Option<String>,
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
    assert_eq!(error.is_data(), true);
}

#[test]
fn can_deserialize_minimal_userconfig() -> Result<(), serde_json::Error> {
    let test: Userconfig =
        serde_json::from_str(r#"{"calendarfileSuffix": "123qwe", "changes": [], "events": []}"#)?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.changes.len(), 0);
    assert_eq!(test.events.len(), 0);
    assert_eq!(test.removed_events, RemovedEvents::Cancelled);

    Ok(())
}

#[test]
fn can_deserialize_userconfig_with_events() -> Result<(), serde_json::Error> {
    let test: Userconfig = serde_json::from_str(
        r#"{"calendarfileSuffix": "123qwe", "changes": [], "events": ["BTI1-TI", "BTI5-VS"], "removedEvents": "removed"}"#,
    )?;

    assert_eq!(test.calendarfile_suffix, "123qwe");
    assert_eq!(test.changes.len(), 0);
    assert_eq!(test.events, ["BTI1-TI", "BTI5-VS"]);
    assert_eq!(test.removed_events, RemovedEvents::Removed);

    Ok(())
}

#[test]
fn can_serialize_minimal_userconfig() -> Result<(), serde_json::Error> {
    let test = Userconfig {
        calendarfile_suffix: "123qwe".to_owned(),
        changes: vec![],
        events: vec!["BTI1-TI".to_owned(), "BTI5-VS".to_owned()],
        removed_events: RemovedEvents::Removed,
    };

    assert_eq!(
        serde_json::to_string(&test)?,
        r#"{"calendarfileSuffix":"123qwe","changes":[],"events":["BTI1-TI","BTI5-VS"],"removedEvents":"removed"}"#
    );

    Ok(())
}

#[test]
fn can_deserialize_minimal_change() -> Result<(), serde_json::Error> {
    let test: Change = serde_json::from_str(r#"{"name": "Tree", "date": "2020-12-20T22:04"}"#)?;
    assert_eq!(test.add, None);
    assert_eq!(test.name, "Tree");
    assert_eq!(test.date, "2020-12-20T22:04");
    assert_eq!(test.remove, None);
    assert_eq!(test.namesuffix, None);
    assert_eq!(test.starttime, None);
    assert_eq!(test.endtime, None);
    assert_eq!(test.room, None);

    Ok(())
}
