//! Handles Postgres migrations
//!

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

#[cfg(test)]
extern crate tempdir;

#[macro_use]
extern crate clap;
extern crate regex;
extern crate postgres;
extern crate ansi_term;

use std::path::Path;
use std::env;

mod files;
mod drivers;
mod errors;
mod cmd;



fn main() {
    let matches = clap_app!(myapp =>
        (@setting SubcommandRequiredElseHelp)
        (version: &crate_version!()[..])
        (author: "Vincent Prouillet <vincent@wearewizards.io>")
        (about: "
Handles migrations for databases.
Each call requires the database url and the path to the directory containing
the SQL migration files.
Those can be set using the DBMIGRATE_URL and DBMIGRATE_PATH environment variables
or the --url and --path arguments.
Using arguments will override the environment variables.
        ")
        (@arg url: -u --url +takes_value "Sets the URL of the database to use.")
        (@arg path: -p --path +takes_value "Sets the folder containing the migrations")
        (@subcommand create =>
            (about: "Creates two migration files (up and down) with the given slug")
            (@arg slug: +required "Sets the name of the migration")
        )
        (@subcommand sync =>
            (about: "Apply all non-applied migrations")
        )
        (@subcommand rollback =>
            (about: "Rollback the current migration")
        )
        (@subcommand status =>
            (about: "See list of migrations and which ones are applied")
        )
        (@subcommand goto =>
            (about: "Go to a specific migration, migration up/down on its way")
            (@arg migration_number: +required "Which migration to go to")
        )
    ).get_matches();

    // TODO: that's quite ugly, surely there's a better way
    let url = match matches.value_of("url") {
        Some(url) => url.to_owned(),
        None => {
            if env::var("DBMIGRATE_URL").is_ok() {
                env::var("DBMIGRATE_URL").unwrap()
            } else {
                errors::no_database_url().exit();
            }
        }
    };

    let path_value = match matches.value_of("path") {
        Some(path) => path.to_owned(),
        None => {
            if env::var("DBMIGRATE_PATH").is_ok() {
                env::var("DBMIGRATE_PATH").unwrap()
            } else {
                errors::no_migration_path().exit();
            }
        }
    };

    let path = Path::new(&path_value);

    let migration_files = match files::read_migrations_files(path) {
        Ok(files) => files,
        Err(e) => e.exit()
    };

    match matches.subcommand_name() {
        Some("status") => cmd::status(&url, &migration_files),
        Some("create") => {
            // Should be safe unwraps
            let slug = matches.subcommand_matches("create").unwrap().value_of("slug").unwrap();
            cmd::create(&migration_files, path, slug)
        },
        None        => println!("No subcommand was used"),
        _           => println!("Some other subcommand was used"),
    }
}