use std::time::Duration;

use eframe::egui;

use crate::{config::{AppConfig, ScreenConfig, Rgba8, TimeInterval, IntervalMode, CycleStep, TimeOfDay}, timer::{format_duration_hhmmss, TimerScheduler, format_time_until_transition, validate_intervals, get_daily_transitions}, utils::{tr, set_language}};

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
    pub timer_scheduler: TimerScheduler,
}

impl AppState {
    pub fn update_ui(&mut self, ctx: &egui::Context) {
        // Настраиваем глобальный стиль приложения
        self.setup_custom_style(ctx);
        
        // Обрабатываем горячие клавиши
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.timer_scheduler.toggle_pause();
            }
            if i.key_pressed(egui::Key::F5) || (i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
                self.timer_scheduler.force_update(&self.config);
            }
            if i.key_pressed(egui::Key::F1) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Comma)) {
                self.show_settings = !self.show_settings;
            }
        });
        
        // Обновляем планировщик таймера
        let screen_changed = self.timer_scheduler.update(&self.config);
        
        // Если экран изменился и включены звуковые уведомления, можно добавить звук
        if screen_changed && self.config.system_settings.sound_notifications {
            // TODO: Добавить воспроизведение звука при смене экранов
        }
        
        ctx.request_repaint_after(Duration::from_secs(1));
        self.top_bar(ctx);
        self.main_panel(ctx);
        self.settings_window(ctx);
    }

    fn setup_custom_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Настройки шрифтов
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(24.0, egui::FontFamily::Proportional)
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(16.0, egui::FontFamily::Proportional)
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(16.0, egui::FontFamily::Proportional)
        );
        
        // Настройки отступов и размеров
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.indent = 16.0;
        
        // Настройки визуального стиля
        style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(40, 40, 40, 240);
        
        ctx.set_style(style);
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        // Невидимая область для перетаскивания и контролов
        let title_bar_height = 30.0;
        
        egui::TopBottomPanel::top("title_bar")
            .exact_height(title_bar_height)
            .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Делаем заголовок перетаскиваемым
                let title_bar_rect = ui.max_rect();
                let title_bar_response = ui.allocate_rect(title_bar_rect, egui::Sense::click());
                
                if title_bar_response.is_pointer_button_down_on() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    
                    // Левая сторона - выбор языка (маленький и прозрачный)
                    let mut selected = self.config.language.to_string();
                    ui.style_mut().spacing.combo_width = 70.0;
                    
                    ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20);
                        ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                        ui.visuals_mut().widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60);
                        
                        egui::ComboBox::from_id_salt("lang_combo")
                            .selected_text(match selected.as_str() {
                                "ru-RU" | "ru" => "RU",
                                _ => "EN",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected, "en-US".to_owned(), "English");
                                ui.selectable_value(&mut selected, "ru-RU".to_owned(), "Русский");
                            });
                    });

                    if selected != self.config.language.to_string() {
                        set_language(self, &selected);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        
                        // Кнопки управления - белые и полупрозрачные
                        let button_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                        
                        // Кнопка закрытия
                        if ui.add(egui::Button::new("✕")
                            .fill(button_color)
                            .stroke(egui::Stroke::NONE))
                            .on_hover_text("Закрыть")
                            .clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // Кнопка сворачивания  
                        if ui.add(egui::Button::new("−")
                            .fill(button_color)
                            .stroke(egui::Stroke::NONE))
                            .on_hover_text("Свернуть")
                            .clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }

                        // Кнопка настроек
                        if ui.add(egui::Button::new("⚙")
                            .fill(button_color)
                            .stroke(egui::Stroke::NONE))
                            .on_hover_text("Настройки")
                            .clicked() {
                            self.show_settings = true;
                        }
                    });
                });
            });
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        // Клонируем информацию о текущем экране, чтобы избежать проблем с заимствованием
        let current_screen = self.timer_scheduler.state.current_screen.clone();
        let next_transition = self.timer_scheduler.state.next_transition;
        let is_running = self.timer_scheduler.state.is_running;
        
        if let Some(active) = current_screen {
            let bg = active.color.to_egui();
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(bg))
                .show(ctx, |ui| {
                    let remaining_text = format_duration_hhmmss(active.remaining_seconds);
                    
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        
                        // 1. ТАЙМЕР ПЕРВЫЙ - большой белый таймер по центру
                        let timer_text = egui::RichText::new(remaining_text)
                            .size(64.0)
                            .strong()
                            .color(egui::Color32::WHITE);
                        ui.label(timer_text);
                        
                        ui.add_space(15.0);
                        
                        // 2. ЗАГОЛОВОК ВТОРОЙ
                        let title_text = egui::RichText::new(&active.title)
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::WHITE);
                        ui.label(title_text);
                        
                        // 3. ПОДЗАГОЛОВОК ТРЕТИЙ
                        if !active.subtitle.is_empty() {
                            ui.add_space(3.0);
                            let subtitle_text = egui::RichText::new(&active.subtitle)
                                .size(16.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 220));
                            ui.label(subtitle_text);
                        }
                        
                        ui.add_space(20.0);
                        
                        // Информация о следующем переходе
                        if let Some(next_transition) = next_transition {
                            let next_text = format_time_until_transition(Some(next_transition));
                            let next_text_styled = egui::RichText::new(&format!("Смена интервала через: {}", next_text))
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200));
                            ui.label(next_text_styled);
                            ui.add_space(12.0);
                        }
                        
                        // Кнопки управления - правильно отцентрированные
                        ui.horizontal_centered(|ui| {
                            let button_size = egui::vec2(100.0, 35.0);
                            let button_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                            
                            if is_running {
                                if ui.add_sized(button_size, egui::Button::new("⏸ Пауза")
                                    .fill(button_color))
                                    .clicked() {
                                    self.timer_scheduler.toggle_pause();
                                }
                            } else {
                                if ui.add_sized(button_size, egui::Button::new("▶ Продолжить")
                                    .fill(button_color))
                                    .clicked() {
                                    self.timer_scheduler.toggle_pause();
                                }
                            }
                            
                            ui.add_space(8.0);
                            
                            if ui.add_sized(button_size, egui::Button::new("🔄 Обновить")
                                .fill(button_color))
                                .clicked() {
                                self.timer_scheduler.force_update(&self.config);
                            }
                        });
                        
                        // Статус паузы
                        if !is_running {
                            ui.add_space(10.0);
                            let pause_text = egui::RichText::new("⏸ Таймер приостановлен")
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200));
                            ui.label(pause_text);
                        }
                    });
                });
        } else {
            // Состояние когда нет экранов
            egui::CentralPanel::default()
                .frame(egui::Frame::default()
                    .fill(egui::Color32::from_rgb(60, 60, 60)))
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            let title = egui::RichText::new("Нет настроенных экранов")
                                .size(24.0)
                                .color(egui::Color32::WHITE);
                            ui.label(title);
                            
                            ui.add_space(8.0);
                            
                            let hint = egui::RichText::new("Откройте настройки для создания экранов и интервалов")
                                .size(14.0)
                                .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180));
                            ui.label(hint);
                        });
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
        
        // Валидация и предупреждения
        ui.separator();
        ui.heading("Валидация настроек");
        
        let validation_errors = validate_intervals(&self.config.intervals);
        if !validation_errors.is_empty() {
            ui.group(|ui| {
                ui.strong("⚠ Обнаружены проблемы в настройках:");
                for error in &validation_errors {
                    ui.small(error);
                }
            });
        } else {
            ui.group(|ui| {
                ui.strong("✓ Настройки корректны");
                ui.small("Все интервалы настроены правильно");
            });
        }
        
        // Показываем расписание переходов на день
        ui.separator();
        ui.heading("Расписание переходов");
        
        let transitions = get_daily_transitions(&self.config);
        if transitions.is_empty() {
            ui.small("Нет настроенных переходов");
        } else {
            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (time_min, description, transition_type) in transitions {
                            let hour = time_min / 60;
                            let minute = time_min % 60;
                            let icon = match transition_type.as_str() {
                                "start" => "▶",
                                "end" => "⏸",
                                "step" => "🔄",
                                _ => "•",
                            };
                            ui.horizontal(|ui| {
                                ui.monospace(format!("{:02}:{:02}", hour, minute));
                                ui.label(icon);
                                ui.small(description);
                            });
                        }
                    });
            });
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
        let mut settings_changed = false;
        
        // Язык интерфейса
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "system-language"));
            ui.small(tr(&self.bundle, "system-language-desc"));
            
            let mut selected = self.config.language.to_string();
            let current_text = match selected.as_str() {
                "ru-RU" | "ru" => "Русский",
                _ => "English",
            };
            
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("system_language")
                    .selected_text(current_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "en-US".to_owned(), "English");
                        ui.selectable_value(&mut selected, "ru-RU".to_owned(), "Русский");
                    });
            });
            
            if selected != self.config.language.to_string() {
                set_language(self, &selected);
                settings_changed = true;
            }
        });
        
        ui.separator();
        
        // Автозапуск с системой
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "system-autostart"));
            ui.small(tr(&self.bundle, "system-autostart-desc"));
            
            if ui.checkbox(&mut self.config.system_settings.autostart, "").changed() {
                settings_changed = true;
            }
        });
        
        ui.separator();
        
        // Звуковые уведомления
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "system-sounds"));
            ui.small(tr(&self.bundle, "system-sounds-desc"));
            
            if ui.checkbox(&mut self.config.system_settings.sound_notifications, "").changed() {
                settings_changed = true;
            }
        });
        
        ui.separator();
        
        // Положение окна на экране
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "system-window-pos"));
            ui.small(tr(&self.bundle, "system-window-pos-desc"));
            
            let has_position = self.config.system_settings.window_position.is_some();
            let mut remember_position = has_position;
            
            if ui.checkbox(&mut remember_position, "").changed() {
                if remember_position && !has_position {
                    // Запомнить текущую позицию (пока что заглушка)
                    self.config.system_settings.window_position = Some(crate::config::WindowPosition { x: 100.0, y: 100.0 });
                } else if !remember_position && has_position {
                    // Забыть позицию
                    self.config.system_settings.window_position = None;
                }
                settings_changed = true;
            }
            
            if let Some(pos) = &mut self.config.system_settings.window_position {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut pos.x).range(0.0..=2000.0)).changed() {
                        settings_changed = true;
                    }
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut pos.y).range(0.0..=2000.0)).changed() {
                        settings_changed = true;
                    }
                });
            }
        });
        
        ui.separator();
        
        // Информация о версии и разработчике
        ui.group(|ui| {
            ui.strong("О приложении");
            ui.label("FlowTimer v0.1.0");
            ui.small("Приложение для визуального отображения временных интервалов");
            ui.small("© 2024 Pet Projects");
            
            ui.separator();
            ui.strong("Горячие клавиши:");
            ui.small("Space - пауза/продолжить таймер");
            ui.small("F5 или Ctrl+R - принудительно обновить");
            ui.small("F1 или Ctrl+, - открыть/закрыть настройки");
        });
        
        // Автосохранение при изменениях
        if settings_changed {
            let _ = crate::config::save_config(&self.config_path, &self.config);
        }
    }
}


