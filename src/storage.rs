use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const MAX_TASKS: usize = 200;
pub const MAX_TASK_CHARS: usize = 240;

/// A stored task. Older files kept plain strings; those migrate to entries
/// with `done: false` on load (see [`deserialize_tasks`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEntry {
    pub title: String,
    #[serde(default)]
    pub done: bool,
}

impl TaskEntry {
    pub fn new(title: String) -> Self {
        Self { title, done: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub dark: bool,
    pub collapsed: bool,
    pub animations: bool,
    pub translucent: bool,
    #[serde(deserialize_with = "deserialize_tasks")]
    pub tasks: Vec<TaskEntry>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: false,
            collapsed: false,
            animations: true,
            translucent: true,
            tasks: Vec::new(),
        }
    }
}

/// Accepts both the current `[{ "title": .., "done": .. }]` format and the
/// legacy `[".."]` format so no user task is ever dropped by an upgrade.
fn deserialize_tasks<'de, D>(deserializer: D) -> Result<Vec<TaskEntry>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TaskValue {
        Entry {
            title: String,
            #[serde(default)]
            done: bool,
        },
        Title(String),
    }

    Vec::<TaskValue>::deserialize(deserializer).map(|values| {
        values
            .into_iter()
            .map(|value| match value {
                TaskValue::Entry { title, done } => TaskEntry { title, done },
                TaskValue::Title(title) => TaskEntry { title, done: false },
            })
            .collect()
    })
}

impl Settings {
    fn normalize(&mut self) {
        self.tasks = self
            .tasks
            .drain(..)
            .map(|mut task| {
                task.title = task.title.trim().chars().take(MAX_TASK_CHARS).collect();
                task
            })
            .filter(|task| !task.title.is_empty())
            .take(MAX_TASKS)
            .collect();
    }
}

pub fn settings_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "weepwood", "GPUI-Kit-Demo")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No system config directory"))?;
    Ok(dirs.config_dir().join("settings.json"))
}

/// Size of the settings file in bytes, or `None` before the first save. Used by
/// the Developer page.
pub fn settings_size() -> Option<u64> {
    let path = settings_path().ok()?;
    fs::metadata(path).ok().map(|meta| meta.len())
}

/// Restore from the platform config directory.
pub fn load() -> io::Result<Settings> {
    load_from(&settings_path()?)
}

/// Restore from an explicit file. Tests point this at a temporary directory so
/// they never touch the real user profile.
pub fn load_from(path: &Path) -> io::Result<Settings> {
    match fs::read(path) {
        Ok(bytes) => {
            let mut settings: Settings = serde_json::from_slice(&bytes)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
            settings.normalize();
            Ok(settings)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(err) => Err(err),
    }
}

/// Persist to the platform config directory.
pub fn save(settings: &Settings) -> io::Result<()> {
    save_to(&settings_path()?, settings)
}

/// Persist to an explicit file, creating parent directories as needed.
pub fn save_to(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_vec_pretty(settings).map_err(io::Error::other)?;
    fs::write(path, content)
}

/// Pretty-printed settings, for the Developer page's JSON preview.
pub fn to_pretty_json(settings: &Settings) -> String {
    serde_json::to_string_pretty(settings).unwrap_or_else(|err| format!("Unavailable: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_a_clean_task_list() {
        let default = Settings::default();
        assert!(default.tasks.is_empty());
        assert!(!default.dark);
    }

    #[test]
    fn backwards_compatible_json_defaults() {
        let parsed: Settings = serde_json::from_str(r#"{"dark":true}"#).unwrap();
        assert!(parsed.dark);
        assert!(!parsed.collapsed);
        assert!(parsed.tasks.is_empty());
    }

    #[test]
    fn legacy_string_tasks_migrate_to_entries() {
        let parsed: Settings =
            serde_json::from_str(r#"{"tasks":["Write release notes","Old todos"]}"#).unwrap();
        assert_eq!(parsed.tasks.len(), 2);
        assert_eq!(
            parsed.tasks[0],
            TaskEntry::new("Write release notes".into())
        );
        assert!(!parsed.tasks[1].done);
    }

    #[test]
    fn entries_round_trip_with_done_state() {
        let settings = Settings {
            dark: true,
            collapsed: true,
            animations: false,
            translucent: false,
            tasks: vec![
                TaskEntry::new("One".into()),
                TaskEntry {
                    title: "Two".into(),
                    done: true,
                },
            ],
        };
        let data = serde_json::to_string(&settings).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&data).unwrap(), settings);
    }

    #[test]
    fn sanitize_loaded_tasks() {
        let mut settings = Settings {
            dark: false,
            collapsed: false,
            animations: true,
            translucent: true,
            tasks: vec![
                TaskEntry::new(" ".into()),
                TaskEntry::new("  hello  ".into()),
                TaskEntry::new("x".repeat(300)),
            ],
        };
        settings.normalize();
        assert_eq!(settings.tasks[0].title, "hello");
        assert_eq!(settings.tasks[1].title.chars().count(), MAX_TASK_CHARS);
        assert_eq!(settings.tasks.len(), 2);
    }

    #[test]
    fn invalid_json_rejected() {
        assert!(serde_json::from_str::<Settings>("{oops").is_err());
    }
}
