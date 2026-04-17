use chrono::DateTime;
use clap::Parser;
use fmodeparser::PermStrParser;
use owo_colors::OwoColorize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Parser)]
#[command(version, about = "Best ll command")]
struct Cli {
    path: Option<PathBuf>,

    #[arg(short = 'l')]
    all: bool,
}

#[derive(Debug, Display)]
enum EntryType {
    File,
    Dir,
}

impl PartialEq for EntryType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (EntryType::File, EntryType::File) | (EntryType::Dir, EntryType::Dir)
        )
    }
}

#[derive(Debug, Tabled)]
struct FileEntry {
    #[tabled(display("format_name", self))]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Size(b)"}]
    len_bytes: u64,
    #[tabled{rename="Modified"}]
    modified: String,
    #[tabled(rename = "Permissions")]
    permissions: String,
}

/// Formats the name of a file or directory entry with appropriate icons and colors.
fn format_name(name: &str, record: &FileEntry) -> String {
    match record.e_type {
        EntryType::File => "\u{ea7b} ".magenta().bold().to_string() + &name.to_string(),
        EntryType::Dir => "\u{f114} ".yellow().to_string() + &name.to_string(),
    }
}

/// Sorts a vector of FileEntry items.
/// Directories come first, then files.
/// Within each type, items with dots in names come first, then alphabetically.
fn sort_entries(entries: &mut Vec<FileEntry>) {
    entries.sort_by(|a, b| {
        // 1. Приоритет типа: Dir (0) перед File (1)
        // Если EntryType не реализует Ord в нужном порядке,
        // можно сопоставить вручную: Dir => 0, File => 1
        let type_order_a = if a.e_type == EntryType::Dir { 0 } else { 1 };
        let type_order_b = if b.e_type == EntryType::Dir { 0 } else { 1 };

        // 2. Приоритет точки в имени: есть (0) перед нет (1)
        let has_dot_a = if a.name.contains('.') { 0 } else { 1 };
        let has_dot_b = if b.name.contains('.') { 0 } else { 1 };

        // Сравниваем последовательно: Тип -> Наличие точки -> Имя (алфавит)
        type_order_a
            .cmp(&type_order_b)
            .then(has_dot_a.cmp(&has_dot_b))
            .then(a.name.cmp(&b.name))
    });
}

fn main() {
    let cli = Cli::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            let mut get_files = get_files(&path, cli.all);
            sort_entries(&mut get_files);

            let mut table = Table::new(get_files);
            stylise_table(&mut table);

            println!("{}", table);
        } else {
            println!("{}", "Path does not exists".red());
        }
    } else {
        println!("{}", "error reading directory".red());
    }

    println!("{}", path.display());
}

/// Applies styling to the table
fn stylise_table(table: &mut Table) {
    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(1), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
}

/// Retrieves a list of FileEntry from the given path.
/// If 'all' is false, skips entries starting with '.'.
fn get_files(path: &Path, all: bool) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if !all && entry.file_name().to_string_lossy().starts_with(".") {
                    continue;
                }
                map_data(&mut data, entry);
            } else {
                eprintln!("Error reading an entry");
            }
        }
    } else {
        eprintln!("Could not open directory: {}", path.display().red());
    }
    data
}

/// Maps a DirEntry to a FileEntry, extracting metadata like type, size, modification time, and permissions.
fn map_data(data: &mut Vec<FileEntry>, entry: fs::DirEntry) {
    if let Ok(meta) = fs::metadata(entry.path()) {
        data.push(FileEntry {
            name: entry
                .file_name()
                .into_string()
                .unwrap_or("unknown name".into()),
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            len_bytes: meta.len(),
            modified: if let Ok(path) = meta.modified() {
                format_date(path)
            } else {
                "unknown".into()
            },

            permissions: meta
                .convert_permission_to_string()
                .unwrap_or("unknown".into()),
        });
    }
}

/// Formats a SystemTime into a human-readable date string.
/// Returns "unknown" if conversion fails.
fn format_date(datetime: std::time::SystemTime) -> String {
    if let Ok(datetime) = datetime.duration_since(std::time::UNIX_EPOCH) {
        let datetime = DateTime::from_timestamp(
            datetime.as_secs() as i64,
            datetime.subsec_nanos(),
        );
        if let Some(datetime) = datetime {
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "unknown".into()
        }
    } else {
        "unknown".into()
    }
}
