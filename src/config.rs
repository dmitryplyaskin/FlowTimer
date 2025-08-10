use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use eframe::egui;
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;



#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub fn to_egui(self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(self.r, self.g, self.b, self.a)
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    pub id: u32,
    pub title: String,
    pub subtitle: String,
    pub color: Rgba8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,  // 0..=23
    pub minute: u8, // 0..=59
}

impl TimeOfDay {

    pub fn to_minutes(self) -> u32 {
        self.hour as u32 * 60 + self.minute as u32
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: TimeOfDay,
    pub end: TimeOfDay, // предполагаем start < end в рамках одного дня
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticEntry {
    pub screen_id: u32,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalEntry {
    pub screen_id: u32,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimerMode {
    Static {
        schedule: Vec<StaticEntry>,
    },
    Interval {
        range: TimeRange,
        sequence: Vec<IntervalEntry>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: LanguageIdentifier,
    pub screens: Vec<ScreenConfig>,
    pub mode: TimerMode,
    pub default_screen_id: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let screens = vec![
            ScreenConfig { id: 1, title: "Работа".into(), subtitle: "Фокус".into(), color: Rgba8 { r: 46, g: 204, b: 113, a: 255 } }, // зелёный
            ScreenConfig { id: 2, title: "Перерыв".into(), subtitle: "Отдых".into(), color: Rgba8 { r: 231, g: 76, b: 60, a: 255 } }, // красный
            ScreenConfig { id: 3, title: "Подготовка".into(), subtitle: "".into(), color: Rgba8 { r: 52, g: 152, b: 219, a: 255 } }, // синий
        ];
        let mode = TimerMode::Static {
            schedule: vec![
                StaticEntry { screen_id: 2, start: TimeOfDay { hour: 8, minute: 0 }, end: TimeOfDay { hour: 11, minute: 0 } },
                StaticEntry { screen_id: 1, start: TimeOfDay { hour: 11, minute: 0 }, end: TimeOfDay { hour: 14, minute: 0 } },
                StaticEntry { screen_id: 3, start: TimeOfDay { hour: 14, minute: 0 }, end: TimeOfDay { hour: 18, minute: 0 } },
            ],
        };

        Self {
            language: "ru-RU".parse().unwrap(),
            screens,
            mode,
            default_screen_id: Some(1),
        }
    }
}

pub fn load_or_default_config() -> (PathBuf, AppConfig) {
    let dirs = ProjectDirs::from("dev", "pet_projects", "FlowTimer").expect("no valid home directory");
    let config_dir = dirs.config_dir();
    let _ = fs::create_dir_all(config_dir);
    let config_path = config_dir.join("config.json");

    if let Ok(bytes) = fs::read(&config_path) {
        if let Ok(cfg) = serde_json::from_slice::<AppConfig>(&bytes) {
            return (config_path, cfg);
        }
    }
    let cfg = AppConfig::default();
    let _ = save_config(&config_path, &cfg);
    (config_path, cfg)
}

pub fn save_config(path: &PathBuf, cfg: &AppConfig) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(cfg).expect("serialize config");
    fs::write(path, json)
}


