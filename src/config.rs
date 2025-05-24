use directories::ProjectDirs;
use ron::de::from_str;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct ColorsConfig {
    pub background: [f32; 3],
    pub text: [f32; 3],
    pub highlight: [f32; 3],
    pub font_size: f32,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            background: [0.1, 0.1, 0.1],
            text: [1.0, 1.0, 1.0],
            highlight: [0.3, 0.3, 0.7],
            font_size: 16.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub position: (f32, f32),
    pub font_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            position: (100.0, 100.0),
            font_name: "Ubuntu-M".to_string(),
        }
    }
}

pub fn get_config_paths() -> Option<(PathBuf, PathBuf)> {
    let proj_dirs = ProjectDirs::from("com", "example", "rmenu")?;
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).ok()?;
    let colors_path = config_dir.join("colors.ron");
    let app_path = config_dir.join("app.ron");
    Some((colors_path, app_path))
}

pub fn load_config<T: Default + for<'de> Deserialize<'de>>(path: &PathBuf) -> T {
    if let Ok(mut file) = fs::File::open(path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(config) = from_str(&content) {
                return config;
            }
        }
    }
    T::default()
}

pub fn save_config<T: Serialize>(path: &PathBuf, config: &T) {
    if let Ok(serialized) = to_string_pretty(config, PrettyConfig::default()) {
        if let Ok(mut file) = fs::File::create(path) {
            let _ = file.write_all(serialized.as_bytes());
        }
    }
}
