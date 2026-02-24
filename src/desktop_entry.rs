use serde::{Deserialize, Serialize};
use serde_ini::from_str;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesktopEntry {
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: Option<String>,
    pub icon: Option<String>,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub no_display: bool,
    pub hidden: bool,
    pub entry_type: String,
    pub path: Option<String>,
    pub mime_type: Vec<String>,
}

fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "1")
}

fn parse_semicolon_list(s: &str) -> Vec<String> {
    s.split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                None
            } else {
                Some(part.to_string())
            }
        })
        .collect()
}

fn get_localized_value(entries: &BTreeMap<String, String>, key: &str) -> Option<String> {
    if let Some(value) = entries.get(key) {
        return Some(value.clone());
    }

    for (k, v) in entries {
        if k.starts_with(&format!("{}[", key)) && k.ends_with(']') {
            return Some(v.clone());
        }
    }
    None
}

pub fn load_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let content = fs::read_to_string(path).ok()?;
    let ini: BTreeMap<String, BTreeMap<String, String>> = from_str(&content).ok()?;

    let desktop_entry_section = ini.get("Desktop Entry")?;

    let entry_type = desktop_entry_section.get("Type")?.clone();
    if entry_type != "Application" {
        return None;
    }

    let name = get_localized_value(desktop_entry_section, "Name")?;

    let no_display = desktop_entry_section
        .get("NoDisplay")
        .map(|s| parse_bool(s))
        .unwrap_or(false);

    let hidden = desktop_entry_section
        .get("Hidden")
        .map(|s| parse_bool(s))
        .unwrap_or(false);

    if no_display || hidden {
        return None;
    }

    let terminal = desktop_entry_section
        .get("Terminal")
        .map(|s| parse_bool(s))
        .unwrap_or(false);

    let categories = desktop_entry_section
        .get("Categories")
        .map(|s| parse_semicolon_list(s))
        .unwrap_or_default();

    let mime_type = desktop_entry_section
        .get("MimeType")
        .map(|s| parse_semicolon_list(s))
        .unwrap_or_default();

    Some(DesktopEntry {
        name,
        generic_name: get_localized_value(desktop_entry_section, "GenericName"),
        comment: get_localized_value(desktop_entry_section, "Comment"),
        exec: desktop_entry_section.get("Exec").cloned(),
        icon: get_localized_value(desktop_entry_section, "Icon"),
        terminal,
        categories,
        no_display,
        hidden,
        entry_type,
        path: desktop_entry_section.get("Path").cloned(),
        mime_type,
    })
}

pub fn get_applications_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        dirs.push(data_dir.join("applications"));
    }

    dirs.push(std::path::PathBuf::from("/usr/share/applications"));
    dirs.push(std::path::PathBuf::from("/usr/local/share/applications"));

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }

    dirs
}

pub fn get_all_applications() -> Vec<DesktopEntry> {
    let mut apps = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();

    for dir in get_applications_dirs() {
        if !dir.exists() {
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "desktop")
                && let Some(desktop_entry) = load_desktop_file(&path)
            {
                if desktop_entry.name.is_empty() {
                    continue;
                }

                if seen.contains_key(&desktop_entry.name) {
                    continue;
                }
                seen.insert(desktop_entry.name.clone(), true);

                apps.push(desktop_entry);
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    apps
}
