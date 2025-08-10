use std::time::Duration;

use eframe::egui;

use crate::{config::AppConfig, timer::{determine_active_screen, format_duration_hhmmss}, utils::{tr, set_language}};

use fluent_bundle::{FluentBundle, FluentResource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab { Timers, System }

pub struct AppState {
    pub config: AppConfig,
    pub config_path: std::path::PathBuf,
    pub bundle: FluentBundle<FluentResource>,
    pub show_settings: bool,
    pub settings_tab: SettingsTab,
}

impl AppState {
    pub fn update_ui(&mut self, ctx: &egui::Context) {
        ctx.request_repaint_after(Duration::from_secs(1));
        self.top_bar(ctx);
        self.main_panel(ctx);
        self.settings_window(ctx);
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = tr(&self.bundle, "menu-language");
                let mut selected = self.config.language.to_string();
                egui::ComboBox::from_label(label)
                    .selected_text(match selected.as_str() {
                        "ru-RU" | "ru" => "Русский",
                        _ => "English",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "en-US".to_owned(), "English");
                        ui.selectable_value(&mut selected, "ru-RU".to_owned(), "Русский");
                    });

                if selected != self.config.language.to_string() {
                    set_language(self, &selected);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙").on_hover_text("Settings").clicked() {
                        self.show_settings = true;
                    }
                });
            });
        });
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        let now = chrono::Local::now();
        if let Some(active) = determine_active_screen(&self.config, now) {
            let bg = active.color.to_egui();
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(bg))
                .show(ctx, |ui| {
                    let remaining_text = format_duration_hhmmss(active.remaining_seconds);
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.heading(&active.title);
                        if !active.subtitle.is_empty() {
                            ui.label(&active.subtitle);
                        }
                        ui.add_space(24.0);
                        let text = egui::RichText::new(remaining_text).size(64.0).strong();
                        ui.label(text);
                    });
                });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label("No screen configured for now");
                });
            });
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if self.show_settings {
            let title = tr(&self.bundle, "settings-title");
            let mut open = self.show_settings;
            egui::Window::new(title)
                .open(&mut open)
                .default_width(560.0)
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let timers_tab = ui.selectable_label(matches!(self.settings_tab, SettingsTab::Timers), tr(&self.bundle, "tab-timers"));
                        let system_tab = ui.selectable_label(matches!(self.settings_tab, SettingsTab::System), tr(&self.bundle, "tab-system"));
                        if timers_tab.clicked() {
                            self.settings_tab = SettingsTab::Timers;
                        }
                        if system_tab.clicked() {
                            self.settings_tab = SettingsTab::System;
                        }
                    });
                    ui.separator();

                    match self.settings_tab {
                        SettingsTab::Timers => self.ui_tab_timers(ui),
                        SettingsTab::System => self.ui_tab_system(ui),
                    }
                });
            self.show_settings = open;
        }
    }

    fn ui_tab_timers(&mut self, ui: &mut egui::Ui) {
        ui.label("Здесь будет управление экранами, расписаниями и режимами.");
        ui.small("(Шаг 4/6 по ТЗ)");
    }

    fn ui_tab_system(&mut self, ui: &mut egui::Ui) {
        ui.label("Системные настройки (язык, автозапуск и пр.) — позже.");
        ui.small("(Шаг 5 по ТЗ)");
    }
}


