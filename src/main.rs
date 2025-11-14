use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::changestatus::{Changestatus, Changetype, write_change_summary};
use crate::watchcat::Watchcat;

mod apply_changes;
mod apply_details;
mod changestatus;
mod events;
mod generate_ics;
mod output_files;
mod userconfig;
mod userconfigs;
mod watchcat;

fn main() {
    output_files::ensure_directory().expect("should be able to create output directory");
    let mut stdout = std::io::stdout();

    println!("Pull eventfiles...");
    events::pull();
    println!("Begin build all configs...");

    let changes = output_files::all_remove_rest(userconfigs::load_all())
        .expect("should be able to build all initial userconfigs");
    _ = write_change_summary(&mut stdout, changes, Changetype::ALL);

    println!("Finished building all configs. Engage watchcats...\n");
    let mut last_eventfiles_pull = Instant::now();
    let userconfig_watcher = Watchcat::new(userconfigs::FOLDER);

    loop {
        if last_eventfiles_pull.elapsed() > Duration::from_mins(42) {
            println!("\nPull eventfiles...");
            events::pull();
            println!("Begin build all configs...");

            match output_files::all_remove_rest(userconfigs::load_all()) {
                Ok(changes) => {
                    _ = write_change_summary(&mut stdout, changes, Changetype::INTERESTING);
                }
                Err(err) => println!("failed to build all {err:#}"),
            }

            println!("Finished building all configs.\n");
            last_eventfiles_pull = Instant::now();
        }

        for filename in userconfig_watcher.get_changed_filenames() {
            println!("userconfig changed {filename:>16}... ");
            match do_specific(&filename) {
                Ok(change) => println!("{:?} {}", change.changetype, change.name),
                Err(err) => println!("{err:#}"),
            }
        }

        sleep(Duration::from_secs(5));
    }
}

fn do_specific(userconfig_filename: &str) -> anyhow::Result<Changestatus> {
    let config = userconfigs::load_specific(userconfig_filename)?;
    output_files::one(config)
}
