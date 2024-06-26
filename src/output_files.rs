use std::fs;
use std::path::Path;

use crate::apply_changes::apply_changes;
use crate::apply_details::apply_details;
use crate::changestatus::{Changestatus, Changetype};
use crate::events;
use crate::generate_ics::{generate_ics, SoonToBeIcsEvent};
use crate::userconfig::UserconfigFile;

pub struct Buildresult {
    pub changestatus: Changestatus,
    pub filename: String,
}

pub const FOLDER: &str = "calendars";

pub fn ensure_directory() -> Result<(), std::io::Error> {
    fs::create_dir_all(FOLDER)
}

pub fn one(content: UserconfigFile) -> Result<Changestatus, String> {
    Ok(one_internal(content)?.changestatus)
}

fn one_internal(content: UserconfigFile) -> Result<Buildresult, String> {
    let user_id = content.chat.id;
    let first_name = content.chat.first_name;
    let ics_filename = format!("{user_id}-{}.ics", content.config.calendarfile_suffix);
    let path = Path::new(FOLDER).join(&ics_filename);

    let mut changetype = Changetype::Same;

    let existing = get_existing_files(&format!("{user_id}-"))
        .map_err(|err| format!("failed to read existing calendars of user {err}"))?;

    match existing.len() {
        1 => {
            if existing[0] != ics_filename {
                let existing_path = Path::new(FOLDER).join(&existing[0]);

                fs::rename(existing_path, &path)
                    .map_err(|err| format!("failed to rename calendars of user {err}"))?;

                changetype = Changetype::Moved;
            }
        }
        0 => {}
        _ => {
            for filename in existing {
                let existing_path = Path::new(FOLDER).join(filename);
                fs::remove_file(existing_path).map_err(|err| {
                    format!("failed to remove superfluous calendars of user {user_id} {first_name} Error: {err}")
                })?;
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
            Err(err) => println!("skip event {name:32} {err}"),
        }
    }

    if user_events.is_empty() {
        if path.exists() {
            fs::remove_file(&path).map_err(|err| {
                format!("failed to remove calendar with now 0 events {user_id} {first_name} Error: {err}")
            })?;
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

    apply_changes(
        &mut user_events,
        content.config.changes,
        content.config.removed_events,
    )
    .map_err(|err| {
        format!("failed to apply changes for user {user_id} {first_name} Error: {err}")
    })?;

    for event in &mut user_events {
        if let Some(details) = content.config.events.get(&event.name) {
            apply_details(event, details);
        } else {
            // IE changed from IE2-IC/01 to IE2-IC-01 for ics files
            // the old name was used and is not yet updated in the TelegramBot
            // skip here and wait for the user to update them in the TelegramBot
        }
    }

    user_events.sort_by_cached_key(|event| event.start_time);
    let ics_content = generate_ics(&first_name, &user_events);

    if let Ok(current_content) = fs::read_to_string(&path) {
        if ics_content != current_content {
            changetype = Changetype::Changed;
        }
    } else {
        changetype = Changetype::Added;
    };

    if matches!(changetype, Changetype::Changed | Changetype::Added) {
        fs::write(&path, &ics_content).map_err(|err| {
            format!("failed to write ics file content for user {user_id} {first_name} Error: {err}")
        })?;
    }

    Ok(Buildresult {
        filename: ics_filename,
        changestatus: Changestatus {
            name: first_name,
            changetype,
        },
    })
}

fn load_and_parse_events(name: &str) -> Result<Vec<SoonToBeIcsEvent>, String> {
    let mut result = Vec::new();
    for event in events::read(name)? {
        result.push(event.try_into()?);
    }
    Ok(result)
}

pub fn all_remove_rest(list: Vec<UserconfigFile>) -> Result<Vec<Changestatus>, String> {
    let mut changestati: Vec<Changestatus> = Vec::new();
    let mut created_files: Vec<String> = Vec::new();

    for content in list {
        let chat_id = content.chat.id;
        match one_internal(content) {
            Ok(filechange) => {
                changestati.push(filechange.changestatus);
                created_files.push(filechange.filename);
            }
            Err(error) => println!("build for {chat_id} failed. Error: {error}"),
        }
    }

    let existing = get_existing_files("")
        .map_err(|err| format!("failed to read calendars dir for cleanup {err}"))?;

    for filename in existing {
        if created_files.contains(&filename) {
            continue;
        }

        let path = Path::new(FOLDER).join(&filename);
        fs::remove_file(path).map_err(|err| {
            format!("failed to remove superfluous calendar file {filename} Error: {err}")
        })?;

        changestati.push(Changestatus {
            name: filename,
            changetype: Changetype::Removed,
        });
    }

    Ok(changestati)
}

fn get_existing_files(starts_with: &str) -> Result<Vec<String>, std::io::Error> {
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
