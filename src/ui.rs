use std::time::Duration;

use eframe::egui;

use crate::{config::{AppConfig, ScreenConfig, Rgba8, TimeInterval, IntervalMode, CycleStep, TimeOfDay}, timer::{determine_active_screen, format_duration_hhmmss}, utils::{tr, set_language}};

use fluent_bundle::{FluentBundle, FluentResource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab { Timers, System }

#[derive(Debug, Clone)]
pub struct EditingScreen {
    pub screen: ScreenConfig,
    pub is_new: bool,
}

#[derive(Debug, Clone)]
pub struct EditingInterval {
    pub interval: TimeInterval,
    pub is_new: bool,
}

pub struct AppState {
    pub config: AppConfig,
    pub config_path: std::path::PathBuf,
    pub bundle: FluentBundle<FluentResource>,
    pub show_settings: bool,
    pub settings_tab: SettingsTab,
    pub editing_screen: Option<EditingScreen>,
    pub editing_interval: Option<EditingInterval>,
    pub next_screen_id: u32,
    pub next_interval_id: u32,
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
            
            // Создаем отдельное окно настроек
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("settings_window"),
                egui::ViewportBuilder::default()
                    .with_title(&title)
                    .with_inner_size([700.0, 600.0])
                    .with_min_inner_size([600.0, 400.0])
                    .with_resizable(true)
                    .with_close_button(true),
                |ctx, class| {
                    assert!(class == egui::ViewportClass::Immediate);
                    let mut close_requested = false;
                    
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // Увеличиваем размер шрифта для всего интерфейса настроек
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(16.0, egui::FontFamily::Proportional)
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Button,
                            egui::FontId::new(16.0, egui::FontFamily::Proportional)
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Heading,
                            egui::FontId::new(20.0, egui::FontFamily::Proportional)
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Small,
                            egui::FontId::new(14.0, egui::FontFamily::Proportional)
                        );
                        
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

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            match self.settings_tab {
                                SettingsTab::Timers => self.ui_tab_timers(ui),
                                SettingsTab::System => self.ui_tab_system(ui),
                            }
                        });
                    });
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_requested = true;
                    }
                    
                    if close_requested {
                        self.show_settings = false;
                    }
                },
            );
        }
    }

    fn ui_tab_timers(&mut self, ui: &mut egui::Ui) {
        // Управление экранами
        ui.heading("Экраны");
        ui.small("Экраны определяют цвет фона и текст, которые будут показываться");
        
        let mut screen_changed = false;
        let mut to_delete_screen: Option<usize> = None;
        let mut to_edit_screen: Option<usize> = None;
        
        ui.group(|ui| {
            if self.config.screens.is_empty() {
                ui.label("Нет созданных экранов");
            } else {
                for (idx, screen) in self.config.screens.iter().enumerate() {
                    ui.horizontal(|ui| {
                        // Цветовой индикатор
                        let color = screen.color.to_egui();
                        ui.colored_label(color, "●");
                        
                        // Название экрана
                        ui.strong(&screen.title);
                        if !screen.subtitle.is_empty() {
                            ui.label(format!("— {}", screen.subtitle));
                        }
                        
                        // Кнопки управления
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🗑").on_hover_text("Удалить экран").clicked() {
                                to_delete_screen = Some(idx);
                            }
                            if ui.small_button("✏").on_hover_text("Редактировать экран").clicked() {
                                to_edit_screen = Some(idx);
                            }
                        });
                    });
                }
            }
        });
        
        // Кнопка добавления нового экрана
        if ui.button("➕ Создать новый экран").clicked() {
            let new_screen = ScreenConfig {
                id: self.next_screen_id,
                title: "Новый экран".to_string(),
                subtitle: String::new(),
                color: Rgba8 { r: 100, g: 150, b: 200, a: 255 },
            };
            self.editing_screen = Some(EditingScreen {
                screen: new_screen,
                is_new: true,
            });
            self.next_screen_id += 1;
        }
        
        // Обработка операций с экранами
        if let Some(idx) = to_delete_screen {
            if self.config.screens.len() > 1 {
                self.config.screens.remove(idx);
                screen_changed = true;
            }
        }
        
        if let Some(idx) = to_edit_screen {
            if let Some(screen) = self.config.screens.get(idx).cloned() {
                self.editing_screen = Some(EditingScreen {
                    screen,
                    is_new: false,
                });
            }
        }

        ui.separator();
        
        // Управление временными интервалами
        ui.heading("Временные интервалы");
        ui.small("Каждый интервал имеет свое время работы и режим отображения экранов");
        
        let mut interval_changed = false;
        let mut to_delete_interval: Option<usize> = None;
        let mut to_edit_interval: Option<usize> = None;
        
        ui.group(|ui| {
            if self.config.intervals.is_empty() {
                ui.label("Нет созданных интервалов");
            } else {
                for (idx, interval) in self.config.intervals.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&interval.name);
                            ui.label(format!("{}:{:02} — {}:{:02}", 
                                interval.start.hour, interval.start.minute,
                                interval.end.hour, interval.end.minute));
                            
                            // Показываем режим интервала
                            match &interval.mode {
                                IntervalMode::Static { screen_id } => {
                                    if let Some(screen) = self.config.screens.iter().find(|s| s.id == *screen_id) {
                                        ui.colored_label(screen.color.to_egui(), "●");
                                        ui.label(&screen.title);
                                    }
                                    ui.small("(статичный)");
                                }
                                IntervalMode::Cycle { steps } => {
                                    ui.small(format!("(цикл из {} шагов)", steps.len()));
                                }
                            }
                            
                            // Кнопки управления
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🗑").on_hover_text("Удалить интервал").clicked() {
                                    to_delete_interval = Some(idx);
                                }
                                if ui.small_button("✏").on_hover_text("Редактировать интервал").clicked() {
                                    to_edit_interval = Some(idx);
                                }
                            });
                        });
                    });
                }
            }
        });
        
        // Кнопка добавления нового интервала
        if ui.button("➕ Создать новый интервал").clicked() {
            let start_time = if let Some(last) = self.config.intervals.last() {
                last.end
            } else {
                TimeOfDay { hour: 9, minute: 0 }
            };
            
            let new_interval = TimeInterval {
                id: self.next_interval_id,
                name: "Новый интервал".to_string(),
                start: start_time,
                end: TimeOfDay { hour: start_time.hour + 1, minute: start_time.minute },
                mode: IntervalMode::Static { 
                    screen_id: self.config.screens.first().map(|s| s.id).unwrap_or(1) 
                },
            };
            self.editing_interval = Some(EditingInterval {
                interval: new_interval,
                is_new: true,
            });
            self.next_interval_id += 1;
        }
        
        // Обработка операций с интервалами
        if let Some(idx) = to_delete_interval {
            self.config.intervals.remove(idx);
            interval_changed = true;
        }
        
        if let Some(idx) = to_edit_interval {
            if let Some(interval) = self.config.intervals.get(idx).cloned() {
                self.editing_interval = Some(EditingInterval {
                    interval,
                    is_new: false,
                });
            }
        }
        
        // Сохранение изменений
        if screen_changed || interval_changed {
            let _ = crate::config::save_config(&self.config_path, &self.config);
        }
        
        // Окна редактирования
        self.ui_screen_editor(ui.ctx());
        self.ui_interval_editor(ui.ctx());
    }

    fn ui_interval_editor(&mut self, ctx: &egui::Context) {
        if let Some(editing) = &mut self.editing_interval {
            let title = if editing.is_new {
                "Новый интервал"
            } else {
                "Редактирование интервала"
            };
            
            let mut open = true;
            let mut should_close = false;
            let mut should_save = false;
            
            egui::Window::new(title)
                .open(&mut open)
                .default_width(500.0)
                .show(ctx, |ui| {
                    // Увеличиваем шрифт для редактора интервала
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::new(16.0, egui::FontFamily::Proportional)
                    );
                    
                    // Название интервала
                    ui.horizontal(|ui| {
                        ui.label("Название:");
                        ui.text_edit_singleline(&mut editing.interval.name);
                    });
                    
                    // Время работы интервала
                    ui.group(|ui| {
                        ui.strong("Время работы");
                        ui.horizontal(|ui| {
                            ui.label("С");
                            ui.add(egui::DragValue::new(&mut editing.interval.start.hour).range(0..=23).speed(1.0));
                            ui.label(":");
                            ui.add(egui::DragValue::new(&mut editing.interval.start.minute).range(0..=59).speed(1.0));
                            
                            ui.label("до");
                            
                            ui.add(egui::DragValue::new(&mut editing.interval.end.hour).range(0..=23).speed(1.0));
                            ui.label(":");
                            ui.add(egui::DragValue::new(&mut editing.interval.end.minute).range(0..=59).speed(1.0));
                        });
                        
                        // Показать длительность
                        let duration_minutes = if editing.interval.end.to_minutes() > editing.interval.start.to_minutes() {
                            editing.interval.end.to_minutes() - editing.interval.start.to_minutes()
                        } else {
                            0
                        };
                        ui.small(format!("Длительность: {} ч {} мин", duration_minutes / 60, duration_minutes % 60));
                    });
                    
                    ui.separator();
                    
                    // Режим интервала
                    ui.strong("Режим работы интервала");
                    
                    let is_static = matches!(editing.interval.mode, IntervalMode::Static { .. });
                    
                    ui.horizontal(|ui| {
                        if ui.radio(is_static, "Статичный").on_hover_text("Показывает один экран весь интервал").clicked() && !is_static {
                            editing.interval.mode = IntervalMode::Static { 
                                screen_id: self.config.screens.first().map(|s| s.id).unwrap_or(1) 
                            };
                        }
                        if ui.radio(!is_static, "Циклический").on_hover_text("Циклически переключает экраны").clicked() && is_static {
                            editing.interval.mode = IntervalMode::Cycle { steps: vec![] };
                        }
                    });
                    
                    ui.separator();
                    
                    // Настройка в зависимости от режима
                    match &mut editing.interval.mode {
                        IntervalMode::Static { screen_id } => {
                            ui.label("Выберите экран для отображения:");
                            let screen_name = self.config.screens.iter()
                                .find(|s| s.id == *screen_id)
                                .map(|s| s.title.clone())
                                .unwrap_or_else(|| "Выберите экран".to_string());
                            
                            egui::ComboBox::from_id_salt("static_screen_combo")
                                .selected_text(&screen_name)
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for screen in &self.config.screens {
                                        ui.selectable_value(screen_id, screen.id, &screen.title);
                                    }
                                });
                        }
                        IntervalMode::Cycle { steps } => {
                            ui.label("Настройте последовательность экранов:");
                            
                            let mut to_remove: Option<usize> = None;
                            
                            for (idx, step) in steps.iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(&format!("Шаг {}", idx + 1));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("🗑").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        });
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Экран:");
                                        let screen_name = self.config.screens.iter()
                                            .find(|s| s.id == step.screen_id)
                                            .map(|s| s.title.clone())
                                            .unwrap_or_else(|| "Выберите".to_string());
                                        
                                        egui::ComboBox::from_id_salt(format!("cycle_screen_{}", idx))
                                            .selected_text(&screen_name)
                                            .width(120.0)
                                            .show_ui(ui, |ui| {
                                                for screen in &self.config.screens {
                                                    ui.selectable_value(&mut step.screen_id, screen.id, &screen.title);
                                                }
                                            });
                                        
                                        ui.label("Длительность:");
                                        ui.add(egui::DragValue::new(&mut step.duration_minutes).range(1..=480).speed(1.0).suffix(" мин"));
                                    });
                                });
                            }
                            
                            if let Some(idx) = to_remove {
                                steps.remove(idx);
                            }
                            
                            if ui.button("➕ Добавить шаг").clicked() {
                                steps.push(CycleStep {
                                    screen_id: self.config.screens.first().map(|s| s.id).unwrap_or(1),
                                    duration_minutes: 25,
                                });
                            }
                            
                            if !steps.is_empty() {
                                let total_duration: u32 = steps.iter().map(|s| s.duration_minutes).sum();
                                ui.small(format!("Общая длительность цикла: {} мин", total_duration));
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    // Кнопки управления
                    ui.horizontal(|ui| {
                        if ui.button("Сохранить").clicked() {
                            should_save = true;
                        }
                        if ui.button("Отмена").clicked() {
                            should_close = true;
                        }
                    });
                });
            
            if should_save {
                if editing.is_new {
                    self.config.intervals.push(editing.interval.clone());
                } else {
                    if let Some(idx) = self.config.intervals.iter().position(|i| i.id == editing.interval.id) {
                        self.config.intervals[idx] = editing.interval.clone();
                    }
                }
                let _ = crate::config::save_config(&self.config_path, &self.config);
                self.editing_interval = None;
            } else if should_close || !open {
                self.editing_interval = None;
            }
        }
    }
    
    fn ui_screen_editor(&mut self, ctx: &egui::Context) {
        if let Some(editing) = &mut self.editing_screen {
            let title = if editing.is_new {
                tr(&self.bundle, "screen-new")
            } else {
                tr(&self.bundle, "screen-edit")
            };
            
            let mut open = true;
            let mut should_close = false;
            let mut should_save = false;
            
            egui::Window::new(title)
                .open(&mut open)
                .default_width(400.0)
                .show(ctx, |ui| {
                    // Заголовок
                    ui.horizontal(|ui| {
                        ui.label(tr(&self.bundle, "field-title"));
                        ui.text_edit_singleline(&mut editing.screen.title);
                    });
                    
                    // Подзаголовок
                    ui.horizontal(|ui| {
                        ui.label(tr(&self.bundle, "field-subtitle"));
                        ui.text_edit_singleline(&mut editing.screen.subtitle);
                    });
                    
                    // Выбор цвета
                    ui.horizontal(|ui| {
                        ui.label(tr(&self.bundle, "field-color"));
                        let mut color = [
                            editing.screen.color.r as f32 / 255.0,
                            editing.screen.color.g as f32 / 255.0,
                            editing.screen.color.b as f32 / 255.0,
                        ];
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            editing.screen.color = Rgba8 {
                                r: (color[0] * 255.0) as u8,
                                g: (color[1] * 255.0) as u8,
                                b: (color[2] * 255.0) as u8,
                                a: 255,
                            };
                        }
                    });
                    
                    ui.separator();
                    
                    // Предварительный просмотр
                    ui.group(|ui| {
                        ui.set_min_height(100.0);
                        let bg_color = editing.screen.color.to_egui();
                        ui.visuals_mut().widgets.noninteractive.bg_fill = bg_color;
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.heading(&editing.screen.title);
                            if !editing.screen.subtitle.is_empty() {
                                ui.label(&editing.screen.subtitle);
                            }
                            ui.label("12:34");
                            ui.add_space(10.0);
                        });
                    });
                    
                    ui.separator();
                    
                    // Кнопки управления
                    ui.horizontal(|ui| {
                        if ui.button("Сохранить").clicked() {
                            should_save = true;
                        }
                        if ui.button("Отмена").clicked() {
                            should_close = true;
                        }
                    });
                });
            
            if should_save {
                if editing.is_new {
                    self.config.screens.push(editing.screen.clone());
                } else {
                    if let Some(idx) = self.config.screens.iter().position(|s| s.id == editing.screen.id) {
                        self.config.screens[idx] = editing.screen.clone();
                    }
                }
                let _ = crate::config::save_config(&self.config_path, &self.config);
                self.editing_screen = None;
            } else if should_close || !open {
                self.editing_screen = None;
            }
        }
    }

    fn ui_tab_system(&mut self, ui: &mut egui::Ui) {
        ui.label("Системные настройки (язык, автозапуск и пр.) — позже.");
        ui.small("(Шаг 5 по ТЗ)");
    }
}


