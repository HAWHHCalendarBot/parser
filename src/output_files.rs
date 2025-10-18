use std::fs;
use std::path::Path;

use anyhow::Context as _;

use crate::apply_changes::apply_changes;
use crate::apply_details::apply_details;
use crate::changestatus::{Changestatus, Changetype};
use crate::events;
use crate::generate_ics::{SoonToBeIcsEvent, generate_ics};
use crate::userconfig::UserconfigFile;

pub struct Buildresult {
    pub changestatus: Changestatus,
    pub filename: String,
}

pub const FOLDER: &str = "calendars";

pub fn ensure_directory() -> std::io::Result<()> {
    fs::create_dir_all(FOLDER)
}

pub fn one(content: UserconfigFile) -> anyhow::Result<Changestatus> {
    let user_id = content.chat.id;
    one_internal(content)
        .map(|buildresult| buildresult.changestatus)
        .with_context(|| format!("Failed to build calendar for {user_id}"))
}

fn one_internal(content: UserconfigFile) -> anyhow::Result<Buildresult> {
    let user_id = content.chat.id;
    let first_name = content.chat.first_name;
    let ics_filename = format!("{user_id}-{}.ics", content.config.calendarfile_suffix);
    let path = Path::new(FOLDER).join(&ics_filename);

    let mut changetype = Changetype::Same;

    let existing = get_existing_files(&format!("{user_id}-"))
        .context("failed to read existing calendars of user")?;

    match existing.len() {
        1 => {
            if existing[0] != ics_filename {
                let existing_path = Path::new(FOLDER).join(&existing[0]);
                fs::rename(existing_path, &path).context("failed to rename old calendar")?;
                changetype = Changetype::Moved;
            }
        }
        0 => {}
        _ => {
            for filename in existing {
                let existing_path = Path::new(FOLDER).join(filename);
                fs::remove_file(existing_path)
                    .context("failed to remove superfluous calendars of user")?;
                changetype = Changetype::Removed;
            }
        }
    }

    let mut user_events = Vec::new();
    let mut event_keys = content.config.events.keys().collect::<Vec<_>>();
    event_keys.sort();
    for name in event_keys {
        match load_and_parse_events(name) {
            Ok(mut events) => user_events.append(&mut events),
            Err(err) => println!("skip event {name:32} {err:#}"),
        }
    }

    if user_events.is_empty() {
        if path.exists() {
            fs::remove_file(&path).context("failed to remove calendar with now 0 events")?;
            changetype = Changetype::Removed;
        } else {
            changetype = Changetype::Skipped;
        }

        return Ok(Buildresult {
            filename: ics_filename,
            changestatus: Changestatus {
                name: first_name,
                changetype,
            },
        });
    }

    for event in &mut user_events {
        if let Some(details) = content.config.events.get(&event.name) {
            apply_details(event, details);
        } else {
            // IE changed from IE2-IC/01 to IE2-IC-01 for ics files
            // the old name was used and is not yet updated in the TelegramBot
            // skip here and wait for the user to update them in the TelegramBot
        }
    }

    apply_changes(&mut user_events, content.config);

    user_events.sort_by_cached_key(|event| event.start_time);
    let ics_content = generate_ics(&first_name, &user_events);

    if let Ok(current_content) = fs::read_to_string(&path) {
        if ics_content != current_content {
            changetype = Changetype::Changed;
        }
    } else {
        changetype = Changetype::Added;
    }

    if matches!(changetype, Changetype::Changed | Changetype::Added) {
        fs::write(&path, &ics_content).context("failed to write ics file content")?;
    }

    Ok(Buildresult {
        filename: ics_filename,
        changestatus: Changestatus {
            name: first_name,
            changetype,
        },
    })
}

fn load_and_parse_events(name: &str) -> anyhow::Result<Vec<SoonToBeIcsEvent>> {
    let mut result = Vec::new();
    for event in events::read(name)? {
        result.push(event.into());
    }
    Ok(result)
}

pub fn all_remove_rest(list: Vec<UserconfigFile>) -> anyhow::Result<Vec<Changestatus>> {
    let mut changestati: Vec<Changestatus> = Vec::new();
    let mut created_files: Vec<String> = Vec::new();

    for content in list {
        let chat_id = content.chat.id;
        match one_internal(content) {
            Ok(filechange) => {
                changestati.push(filechange.changestatus);
                created_files.push(filechange.filename);
            }
            Err(error) => println!("Failed to build calendar for {chat_id}: {error:#}"),
        }
    }

    let existing = get_existing_files("").context("failed to read calendars dir for cleanup")?;

    for filename in existing {
        if created_files.contains(&filename) {
            continue;
        }

        let path = Path::new(FOLDER).join(&filename);
        fs::remove_file(path)
            .with_context(|| format!("failed to remove superfluous calendar file {filename}"))?;

        changestati.push(Changestatus {
            name: filename,
            changetype: Changetype::Removed,
        });
    }

    Ok(changestati)
}

fn get_existing_files(starts_with: &str) -> std::io::Result<Vec<String>> {
    let mut list: Vec<String> = Vec::new();
    for maybe_entry in fs::read_dir(FOLDER)? {
        let filename = maybe_entry?
            .file_name()
            .into_string()
            .expect("filename should be UTF8");

        if filename.starts_with(starts_with) {
            list.push(filename);
        }
    }

    Ok(list)
}
