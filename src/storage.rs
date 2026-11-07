use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

pub const MAX_TASKS: usize = 200;
pub const MAX_TASK_CHARS: usize = 240;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub dark: bool,
    pub collapsed: bool,
    pub tasks: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: false,
            collapsed: false,
            tasks: vec![
                "Explore GPUI Kit components".into(),
                "Customize the desktop shell".into(),
            ],
        }
    }
}

impl Settings {
    fn normalize(&mut self) {
        self.tasks = self.tasks.drain(..)
            .map(|task| task.trim().chars().take(MAX_TASK_CHARS).collect::<String>())
            .filter(|task| !task.is_empty())
            .take(MAX_TASKS)
            .collect();
    }
}

pub fn settings_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "weepwood", "GPUI-Kit-Demo")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No system config directory"))?;
    Ok(dirs.config_dir().join("settings.json"))
}

pub fn load() -> io::Result<Settings> {
    let path = settings_path()?;
    match fs::read(path) {
        Ok(bytes) => {
            let mut settings: Settings = serde_json::from_slice(&bytes)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
            settings.normalize();
            Ok(settings)
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(err) => Err(err),
    }
}

pub fn save(settings: &Settings) -> io::Result<()> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let content = serde_json::to_vec_pretty(settings).map_err(io::Error::other)?;
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_two_tasks() {
        let default = Settings::default();
        assert_eq!(default.tasks.len(), 2);
        assert!(!default.dark);
    }

    #[test]
    fn backwards_compatible_json_defaults() {
        let parsed: Settings = serde_json::from_str(r#"{"dark":true}"#).unwrap();
        assert!(parsed.dark);
        assert!(!parsed.collapsed);
        assert_eq!(parsed.tasks.len(), 2);
    }

    #[test]
    fn serialize_and_restore() {
        let settings = Settings { dark:true, collapsed:true, tasks:vec!["One".to_string()] };
        let data = serde_json::to_string(&settings).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&data).unwrap(), settings);
    }

    #[test]
    fn sanitize_loaded_tasks() {
        let mut settings = Settings {
            dark:false, collapsed:false,
            tasks:vec![" ".to_owned(), "  hello  ".to_owned(), "x".repeat(300)],
        };
        settings.normalize();
        assert_eq!(settings.tasks[0], "hello");
        assert_eq!(settings.tasks[1].chars().count(), MAX_TASK_CHARS);
        assert_eq!(settings.tasks.len(), 2);
    }

    #[test]
    fn invalid_json_rejected() {
        assert!(serde_json::from_str::<Settings>("{oops").is_err());
    }
}
