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
pub struct CycleStep {
    pub screen_id: u32,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum IntervalMode {
    Static {
        screen_id: u32,
    },
    Cycle {
        steps: Vec<CycleStep>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub id: u32,
    pub name: String,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub mode: IntervalMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettings {
    pub autostart: bool,
    pub sound_notifications: bool,
    pub window_position: Option<WindowPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            sound_notifications: false,
            window_position: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: LanguageIdentifier,
    pub screens: Vec<ScreenConfig>,
    pub intervals: Vec<TimeInterval>,
    pub default_screen_id: Option<u32>,
    pub system_settings: SystemSettings,
}

impl AppConfig {
    pub fn create_default_with_localization() -> Self {
        // Определяем язык системы для выбора локализованных названий по умолчанию
        let is_russian = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LANGUAGE"))
            .map(|lang| lang.starts_with("ru"))
            .unwrap_or(true); // По умолчанию русский, так как это российский проект
            
        let (work_title, work_subtitle, break_title, break_subtitle, prep_title) = if is_russian {
            ("Работа", "Фокус", "Перерыв", "Отдых", "Подготовка")
        } else {
            ("Work", "Focus", "Break", "Rest", "Preparation")
        };
        
        let (morning_name, pomodoro_name) = if is_russian {
            ("Утренняя работа", "Помодоро сессия")
        } else {
            ("Morning work", "Pomodoro session")
        };
        
        let screens = vec![
            ScreenConfig { 
                id: 1, 
                title: work_title.into(), 
                subtitle: work_subtitle.into(), 
                color: Rgba8 { r: 46, g: 204, b: 113, a: 255 } // зелёный
            },
            ScreenConfig { 
                id: 2, 
                title: break_title.into(), 
                subtitle: break_subtitle.into(), 
                color: Rgba8 { r: 231, g: 76, b: 60, a: 255 } // красный
            },
            ScreenConfig { 
                id: 3, 
                title: prep_title.into(), 
                subtitle: "".into(), 
                color: Rgba8 { r: 52, g: 152, b: 219, a: 255 } // синий
            },
        ];
        
        let intervals = vec![
            TimeInterval {
                id: 1,
                name: morning_name.into(),
                start: TimeOfDay { hour: 9, minute: 0 },
                end: TimeOfDay { hour: 12, minute: 0 },
                mode: IntervalMode::Static { screen_id: 1 },
            },
            TimeInterval {
                id: 2,
                name: pomodoro_name.into(),
                start: TimeOfDay { hour: 14, minute: 0 },
                end: TimeOfDay { hour: 18, minute: 0 },
                mode: IntervalMode::Cycle {
                    steps: vec![
                        CycleStep { screen_id: 1, duration_minutes: 25 },
                        CycleStep { screen_id: 2, duration_minutes: 5 },
                    ],
                },
            },
        ];

        Self {
            language: if is_russian { "ru-RU".parse().unwrap() } else { "en-US".parse().unwrap() },
            screens,
            intervals,
            default_screen_id: Some(1),
            system_settings: SystemSettings::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::create_default_with_localization()
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


