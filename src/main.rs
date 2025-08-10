#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod config;
mod timer;
mod ui;
mod utils;

use config::load_or_default_config;
use ui::{AppState, SettingsTab};
use utils::make_bundle;
use timer::TimerScheduler;

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([400.0, 600.0])
        .with_min_inner_size([400.0, 600.0])
        .with_max_inner_size([400.0, 600.0])
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top();

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "FlowTimer",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

// AppConfig объявлен ниже вместе с остальными моделями

struct MyApp(AppState);

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (config_path, config) = load_or_default_config();
        let bundle = make_bundle(&config.language);
        let next_screen_id = config.screens.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let next_interval_id = config.intervals.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        Self(AppState {
            config,
            config_path,
            bundle,
            show_settings: false,
            settings_tab: SettingsTab::Timers,
            editing_screen: None,
            editing_interval: None,
            next_screen_id,
            next_interval_id,
            timer_scheduler: TimerScheduler::new(),
        })
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.update_ui(ctx);
    }
}










