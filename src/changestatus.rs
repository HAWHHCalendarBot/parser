use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Changetype {
    Added,
    Changed,
    Moved,
    Removed,
    Same,
    Skipped,
}

impl Changetype {
    pub const ALL: &'static [Self] = &[
        Self::Added,
        Self::Changed,
        Self::Moved,
        Self::Removed,
        Self::Same,
        Self::Skipped,
    ];
    pub const INTERESTING: &'static [Self] =
        &[Self::Added, Self::Changed, Self::Moved, Self::Removed];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Changed => "changed",
            Self::Moved => "moved",
            Self::Removed => "removed",
            Self::Same => "same",
            Self::Skipped => "skipped",
        }
    }
}

pub struct Changestatus {
    pub name: String,
    pub changetype: Changetype,
}

pub fn write_change_summary<W: std::io::Write>(
    target: &mut W,
    changes: Vec<Changestatus>,
    to_be_shown: &[Changetype],
) -> std::io::Result<()> {
    let mut map: HashMap<Changetype, Vec<String>> = HashMap::new();
    for change in changes {
        map.entry(change.changetype).or_default().push(change.name);
    }
    for key in to_be_shown {
        if let Some(val) = map.get_mut(key) {
            val.sort_by_key(|string| string.to_lowercase());
            let key = key.as_str();
            writeln!(target, "{key:7} ({:4}): {val:?}", val.len())?;
        }
    }
    Ok(())
}

#[cfg(test)]
fn generate_every_type_once() -> Vec<Changestatus> {
    vec![
        Changestatus {
            name: String::from("A"),
            changetype: Changetype::Added,
        },
        Changestatus {
            name: String::from("C"),
            changetype: Changetype::Changed,
        },
        Changestatus {
            name: String::from("M"),
            changetype: Changetype::Moved,
        },
        Changestatus {
            name: String::from("R"),
            changetype: Changetype::Removed,
        },
        Changestatus {
            name: String::from("Sa"),
            changetype: Changetype::Same,
        },
        Changestatus {
            name: String::from("Sk"),
            changetype: Changetype::Skipped,
        },
    ]
}

#[test]
fn summary_without_changes_is_empty() {
    let mut result = Vec::new();
    write_change_summary(&mut result, vec![], Changetype::ALL).unwrap();
    assert_eq!(result, b"");
}

#[test]
fn summary_shows_every_type_once() {
    let mut result = Vec::new();
    write_change_summary(&mut result, generate_every_type_once(), Changetype::ALL).unwrap();
    let result = String::from_utf8(result).unwrap();
    assert_eq!(
        result,
        r#"added   (   1): ["A"]
changed (   1): ["C"]
moved   (   1): ["M"]
removed (   1): ["R"]
same    (   1): ["Sa"]
skipped (   1): ["Sk"]
"#
    );
}

#[test]
fn summary_shows_interesting_types_once() {
    let mut result = Vec::new();
    write_change_summary(
        &mut result,
        generate_every_type_once(),
        Changetype::INTERESTING,
    )
    .unwrap();
    let result = String::from_utf8(result).unwrap();
    assert_eq!(
        result,
        r#"added   (   1): ["A"]
changed (   1): ["C"]
moved   (   1): ["M"]
removed (   1): ["R"]
"#
    );
}
