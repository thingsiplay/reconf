#![allow(dead_code)]

use std::fmt;
use std::fs::File;
//use std::io::{self, BufRead, Write};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

// use core::iter::Rev;

// https://docs.rs/indexmap/latest/indexmap/
use indexmap::IndexMap;

// https://docs.rs/regex/latest/regex/
use regex::Regex;

// https://docs.rs/rev_lines/latest/rev_lines/
use rev_lines::RevLines;

// https://crates.io/crates/compact_str/
use compact_str::CompactString;
use compact_str::ToCompactString;

// https://docs.rs/colored/latest/colored/
use colored::Colorize;

#[cfg(windows)]
const NL: &str = "\r\n";

#[cfg(not(windows))]
const NL: &str = "\n";

#[derive(Debug, Default)]
pub struct Config {
    // Used for reading from and writing to .cfg files.
    pub path: Option<PathBuf>,
    // A code for coloring and styling when formatting keys and values as string.
    pub style: u8,
    //
    pub lineending: &'static str,
    // Controls on values if surrounding quotation marks should be automatically removed and added
    // when reading or writing. (experimental)
    pub data: IndexMap<CompactString, CompactString>,
}

// Convert internal data to String representation.
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data_as_string: String = self
            .data
            .iter()
            .rev()
            .map(|(key, value)| {
                format!(
                    "{} = \"{}\"",
                    format_key_string(key, self.style),
                    format_value_string(value, self.style),
                )
            })
            .collect::<Vec<_>>()
            .join(self.lineending);
        write!(f, "{}", data_as_string)?;
        Ok(())
    }
}

impl Config {
    pub fn new() -> Config {
        Config {
            path: None,
            style: 0,
            lineending: NL,
            data: IndexMap::new(),
        }
    }

    // Set path by slice and read file into data.
    pub fn load(&mut self, filename: &str) {
        self.set_path(filename);
        self.read_file();
    }

    // Overwrite file at path with current data converted to cfg text format.
    pub fn write(&self) -> io::Result<()> {
        match &self.path {
            Some(path) if path.is_dir() => {
                eprintln!("Error! Path is directory: {}", path.display());
            }
            Some(path) => {
                let mut file = File::create(path.as_os_str())?;
                file.write_all(self.to_string().as_bytes())?;
            }
            None => {
                eprintln!("Error! Can't write file, no filename given.");
            }
        }
        Ok(())
    }

    // Read value of given key.
    pub fn get(&self, key: &str) -> Option<CompactString> {
        self.data.get(key).map(ToCompactString::to_compact_string)
    }

    // Update existing or add missing key value pair to internal data.
    pub fn set(&mut self, key: &str, value: &str) -> Option<CompactString> {
        self.data
            .insert(key.to_compact_string(), value.to_compact_string())
    }

    // Search in value of existing key exact search string and change matching part with replace string.
    pub fn replace(
        &mut self,
        key: &str,
        search: &str,
        replace: &str,
    ) -> Option<CompactString> {
        match self.data.get(key).map(ToCompactString::to_compact_string) {
            Some(value) => {
                let new_value = value.replace(search, replace);
                self.data.insert(
                    key.to_compact_string(),
                    new_value.to_compact_string(),
                )
            }
            None => None,
        }
    }

    // Add missing key value pair or if existing, get its value.
    pub fn add(&mut self, key: &str, value: &str) -> Option<CompactString> {
        if self.data.contains_key(key) {
            self.data.get(key).map(ToCompactString::to_compact_string)
        } else {
            self.data
                .insert(key.to_compact_string(), value.to_compact_string())
        }
    }

    // Replace and move existing or add missing key value pair to first position in data.
    pub fn prepend(
        &mut self,
        key: &str,
        value: &str,
    ) -> Option<CompactString> {
        self.data.remove(key);
        self.data.reverse();
        let pair = self
            .data
            .insert(key.to_compact_string(), value.to_compact_string());
        self.data.reverse();
        pair
    }

    // Remove key value pair from internal data by name of key.
    pub fn remove(&mut self, key: &str) -> Option<CompactString> {
        self.data.remove(key)
    }

    // Sort with standard algorithm the key value pairs in data.
    pub fn sort(&mut self) {
        self.data.sort_keys();
        self.data.reverse();
    }

    // Update internal data by parsing a slice in cfg text data format.
    pub fn insert_from_string(&mut self, text: &str) {
        for line in text.lines() {
            self.insert_line(line);
        }
    }

    // Update or add key value pairs provided by another Config, without changing path.
    pub fn insert_from_config(&mut self, config: &Config) {
        for (key, value) in &config.data {
            self.data
                .insert(key.to_compact_string(), value.to_compact_string());
        }
    }

    // Update internal data by providing an indexmap with pair of strings.
    pub fn insert_from_map(
        &mut self,
        map: IndexMap<CompactString, CompactString>,
    ) {
        for (key, value) in map {
            self.data.insert(key, value);
        }
    }

    // Simple check if key is found in data.
    pub fn has_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    // Get a list of all key names only.
    pub fn list_keys(&self) -> Vec<CompactString> {
        self.data
            .keys()
            .map(CompactString::to_compact_string)
            .rev()
            .collect()
    }

    // Get a list of all values only.
    pub fn list_values(&self) -> Vec<CompactString> {
        self.data
            .values()
            .map(CompactString::to_compact_string)
            .rev()
            .collect()
    }

    // Search and find all key value pairs by matching regex pattern to key names and values.
    // An empty pattern will match keys with empty values only.
    pub fn find(
        &self,
        key: &str,
        value: &str,
    ) -> Vec<(CompactString, CompactString)> {
        let re_key = create_regex(key);
        let re_value = if value.is_empty() {
            create_regex("^$")
        } else {
            create_regex(value)
        };
        let mut result = self.data.clone();
        result.retain(|k, v| re_key.is_match(k) && re_value.is_match(v));
        as_rev_list(&result)
    }

    // Search and find all key value pairs by matching regex pattern to key names.
    pub fn find_by_key(
        &self,
        key: &str,
    ) -> Vec<(CompactString, CompactString)> {
        let re_key = create_regex(key);
        let mut result = self.data.clone();
        result.retain(|k, _| re_key.is_match(k));
        as_rev_list(&result)
    }

    // Search and find all key value pairs by matching regex pattern to values.
    // An empty pattern will match keys with empty values only.
    pub fn find_by_value(
        &self,
        value: &str,
    ) -> Vec<(CompactString, CompactString)> {
        let re_value = if value.is_empty() {
            create_regex("^$")
        } else {
            create_regex(value)
        };
        let mut result = self.data.clone();
        result.retain(|_, v| re_value.is_match(v));
        as_rev_list(&result)
    }

    // Update current path, if file exist.
    pub fn set_path(&mut self, path: &str) {
        self.path = Some(normalize_path(PathBuf::from(path).as_path()));
    }

    // Get a copy of current associated optional path.
    pub fn path(&self) -> Option<PathBuf> {
        self.path.as_ref().cloned()
    }

    // Get the current path as a string. If no path is set yet or is not valid os string, then an
    // empty string is returned.
    pub fn path_to_string(&self) -> CompactString {
        match self.path.as_ref() {
            Some(path) => match path.clone().into_os_string().into_string() {
                Ok(filename) => filename.to_compact_string(),
                Err(_) => "".to_compact_string(),
            },
            None => "".to_compact_string(),
        }
    }

    // Read file at current path and key and values to data. If key exists multiple times, the
    // value for first encounter of key have priority.
    pub fn read_file(&mut self) {
        match &self.path {
            Some(file) if file.is_file() => {
                for line in read_lines_reverse(file.as_os_str()) {
                    self.insert_line(line.as_str());
                }
            }
            Some(file) if file.is_dir() => {
                eprintln!(
                    "Warning! Cant read config data. Path is a directory: \"{}\"",
                    file.display()
                );
            }
            Some(file) => {
                eprintln!(
                    "Warning! Cant read config data. Path not a file: \"{}\"",
                    file.display()
                );
            }
            None => {
                eprintln!("Warning! Cant read config data. No path set.");
            }
        }
    }

    // Parse a slice of a line and add key value pair to data.
    pub fn insert_line(&mut self, line: &str) {
        if let Some((key, value)) = Config::parse_line(line) {
            self.data
                .insert(key.to_compact_string(), value.to_compact_string());
        }
    }

    // Parse a slice of a cfg formatted text with keys and values.
    pub fn parse_line(line: &str) -> Option<(CompactString, CompactString)> {
        line.split_once('=').map(|(key, value)| {
            (
                key.trim().to_compact_string(),
                value.trim().trim_matches('"').to_compact_string(),
            )
        })
    }

    pub fn print_key(&self, key: &str) {
        println!("{}", format_key_string(key, self.style));
    }

    pub fn print_value(&self, value: &str) {
        println!("{}", format_value_string(value, self.style));
    }

    pub fn print_pair(&self, key: &str, value: &str) {
        println!(
            "{} = \"{}\"",
            format_key_string(key, self.style),
            format_value_string(value, self.style)
        );
    }

    pub fn print_keys_list(&self) {
        println!(
            "{}",
            format_key_string(
                &self.list_keys().join(self.lineending),
                self.style
            )
        );
    }
}

// Get lines from a file. Lines are in reverse order for priority reasons. Don't forget to reverse
// the lines after work is done.
fn read_lines_reverse<P>(filename: P) -> RevLines<File>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Can't open file.");
    RevLines::new(io::BufReader::new(file))
        .expect("Could not read file line by line.")
}

fn create_regex<S: AsRef<str>>(pattern: S) -> Regex {
    match Regex::new(pattern.as_ref()) {
        Ok(regex) => regex,
        Err(error) => {
            eprintln!(
                "Error! Regex pattern for key is not correct: {}",
                pattern.as_ref()
            );
            panic!("{error}");
        }
    }
}

// https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret =
        if let Some(c @ Component::Prefix(..)) = components.peek().copied() {
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::new()
        };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub fn as_rev_list(
    data: &IndexMap<CompactString, CompactString>,
) -> Vec<(CompactString, CompactString)> {
    let mut rev_list: Vec<(CompactString, CompactString)> = Vec::new();
    for (key, value) in data.iter().rev() {
        rev_list.push((
            key.clone().to_compact_string(),
            value.clone().to_compact_string(),
        ));
    }
    rev_list
}

pub fn format_key_string(key: &str, style: u8) -> colored::ColoredString {
    match style {
        1 => key.trim().bold().blue(),
        2 => key.trim().cyan(),
        3 => key.trim().bright_black(),
        4 => key.trim().dimmed().italic(),
        5 => key.trim().bright_green(),
        6 => key.trim().bright_purple(),
        7 => key.trim().bold().yellow(),
        8 => key.trim().dimmed().blue(),
        9 => key.trim().magenta(),
        _ => key.trim().clear(),
    }
}

pub fn format_value_string(value: &str, style: u8) -> colored::ColoredString {
    match style {
        1 => value.trim().italic().yellow(),
        2 => value.trim().bold().green(),
        3 => value.trim().bright_magenta(),
        4 => value.trim().italic(),
        5 => value.trim().italic().blue(),
        6 => value.trim().dimmed().yellow(),
        7 => value.trim().bright_red(),
        8 => value.trim().bold().white(),
        9 => value.trim().dimmed().white(),
        _ => value.trim().clear(),
    }
}
