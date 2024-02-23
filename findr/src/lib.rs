use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_type: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("me")
        .about("Rust find")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .help("Search paths")
                .multiple(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("names")
                .value_name("NAME")
                .short("n")
                .long("name")
                .help("Name")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("type")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .help("Entry type")
                .multiple(true)
                .possible_values(&["f", "d", "l"]),
        )
        .get_matches();

    let paths = matches.values_of_lossy("paths").unwrap();
    let names = matches
        .values_of_lossy("names")
        .map(|vals| {
            vals.into_iter()
                .map(|name| Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let entry_type = matches
        .values_of_lossy("type")
        .map(|vals| {
            vals.iter()
                .map(|val| match val.as_str() {
                    "d" => Dir,
                    "f" => File,
                    "l" => Link,
                    _ => unreachable!("Invalid type"),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Config {
        paths,
        names,
        entry_type,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| {
        config.entry_type.is_empty() || {
            config.entry_type.iter().any(|entry_type| match entry_type {
                Link => entry.file_type().is_symlink(),
                Dir => entry.file_type().is_dir(),
                File => entry.file_type().is_file(),
            })
        }
    };
    let name_filter = |entry: &DirEntry| {
        config.names.is_empty() || {
            config
                .names
                .iter()
                .any(|name| name.is_match(&entry.file_name().to_string_lossy()))
        }
    };

    for path in config.paths {
        // for entry in WalkDir::new(path) {
        //     match entry {
        //         Err(e) => eprintln!("{}", e),
        //         Ok(entry) => {
        //             if type_filter(&entry) && name_filter(&entry) {
        //                 println!("{}", entry.path().display())
        //             }
        //         }
        //     }
        // }
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();
        println!("{}", entries.join("\n"));
    }

    Ok(())
}
