use std::time::Duration;

use eframe::egui;

use crate::{
    config::{AppConfig, CycleStep, IntervalMode, Rgba8, ScreenConfig, TimeInterval, TimeOfDay},
    timer::{TimerScheduler, format_duration_hhmmss, get_daily_transitions, validate_intervals},
    utils::{set_language, tr, tr_with_args},
};

use fluent_bundle::{FluentBundle, FluentResource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Timers,
    System,
}

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
            if i.key_pressed(egui::Key::F1) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Comma))
            {
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
        self.main_panel(ctx);
        self.settings_window(ctx);
    }

    fn setup_custom_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Настройки шрифтов
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
        );

        // Настройки отступов и размеров
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.indent = 16.0;

        // Настройки визуального стиля
        style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(40, 40, 40, 240);

        ctx.set_style(style);
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        // Клонируем информацию о текущем экране, чтобы избежать проблем с заимствованием
        let current_screen = self.timer_scheduler.state.current_screen.clone();
        let _next_transition = self.timer_scheduler.state.next_transition;
        let _is_running = self.timer_scheduler.state.is_running;

        if let Some(active) = current_screen {
            let bg = active.color.to_egui();
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(bg))
                .show(ctx, |ui| {
                    // ПАНЕЛЬ УПРАВЛЕНИЯ СВЕРХУ
                    ui.horizontal(|ui| {
                        // Делаем область для перетаскивания окна
                        let drag_area = ui.allocate_response(
                            egui::vec2(ui.available_width() - 120.0, 30.0),
                            egui::Sense::click(),
                        );
                        if drag_area.is_pointer_button_down_on() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Кнопки управления - белые и полупрозрачные
                            let button_color =
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);

                            // Кнопка закрытия
                            if ui
                                .add(
                                    egui::Button::new("✕")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-close"))
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }

                            // Кнопка сворачивания
                            if ui
                                .add(
                                    egui::Button::new("−")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-minimize"))
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }

                            // Кнопка настроек
                            if ui
                                .add(
                                    egui::Button::new("⚙")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-settings"))
                                .clicked()
                            {
                                self.show_settings = true;
                            }
                        });
                    });

                    // ОСНОВНОЕ СОДЕРЖИМОЕ
                    let remaining_text = format_duration_hhmmss(active.remaining_seconds);

                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

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
                    });
                });
        } else {
            // Состояние когда нет экранов
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(egui::Color32::from_rgb(60, 60, 60)))
                .show(ctx, |ui| {
                    // ПАНЕЛЬ УПРАВЛЕНИЯ СВЕРХУ (даже когда нет экранов)
                    ui.horizontal(|ui| {
                        // Делаем область для перетаскивания окна
                        let drag_area = ui.allocate_response(
                            egui::vec2(ui.available_width() - 120.0, 30.0),
                            egui::Sense::click(),
                        );
                        if drag_area.is_pointer_button_down_on() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Кнопки управления - белые и полупрозрачные
                            let button_color =
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);

                            // Кнопка закрытия
                            if ui
                                .add(
                                    egui::Button::new("✕")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-close"))
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }

                            // Кнопка сворачивания
                            if ui
                                .add(
                                    egui::Button::new("−")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-minimize"))
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }

                            // Кнопка настроек
                            if ui
                                .add(
                                    egui::Button::new("⚙")
                                        .fill(button_color)
                                        .stroke(egui::Stroke::NONE),
                                )
                                .on_hover_text(tr(&self.bundle, "btn-settings"))
                                .clicked()
                            {
                                self.show_settings = true;
                            }
                        });
                    });

                    // ОСНОВНОЕ СОДЕРЖИМОЕ
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            let title = egui::RichText::new(tr(&self.bundle, "main-no-screens"))
                                .size(24.0)
                                .color(egui::Color32::WHITE);
                            ui.label(title);

                            ui.add_space(8.0);

                            let hint =
                                egui::RichText::new(tr(&self.bundle, "main-no-screens-hint"))
                                    .size(14.0)
                                    .color(egui::Color32::from_rgba_unmultiplied(
                                        255, 255, 255, 180,
                                    ));
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
                            egui::FontId::new(16.0, egui::FontFamily::Proportional),
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Button,
                            egui::FontId::new(16.0, egui::FontFamily::Proportional),
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Heading,
                            egui::FontId::new(20.0, egui::FontFamily::Proportional),
                        );
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Small,
                            egui::FontId::new(14.0, egui::FontFamily::Proportional),
                        );

                        ui.horizontal_wrapped(|ui| {
                            let timers_tab = ui.selectable_label(
                                matches!(self.settings_tab, SettingsTab::Timers),
                                tr(&self.bundle, "tab-timers"),
                            );
                            let system_tab = ui.selectable_label(
                                matches!(self.settings_tab, SettingsTab::System),
                                tr(&self.bundle, "tab-system"),
                            );
                            if timers_tab.clicked() {
                                self.settings_tab = SettingsTab::Timers;
                            }
                            if system_tab.clicked() {
                                self.settings_tab = SettingsTab::System;
                            }
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| match self.settings_tab {
                            SettingsTab::Timers => self.ui_tab_timers(ui),
                            SettingsTab::System => self.ui_tab_system(ui),
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
        ui.heading(tr(&self.bundle, "screens-title"));
        ui.small(tr(&self.bundle, "screens-description"));

        let mut screen_changed = false;
        let mut to_delete_screen: Option<usize> = None;
        let mut to_edit_screen: Option<usize> = None;

        ui.group(|ui| {
            if self.config.screens.is_empty() {
                ui.label(tr(&self.bundle, "screens-none"));
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
                            if ui
                                .small_button("🗑")
                                .on_hover_text(tr(&self.bundle, "screen-delete-tooltip"))
                                .clicked()
                            {
                                to_delete_screen = Some(idx);
                            }
                            if ui
                                .small_button("✏")
                                .on_hover_text(tr(&self.bundle, "screen-edit-tooltip"))
                                .clicked()
                            {
                                to_edit_screen = Some(idx);
                            }
                        });
                    });
                }
            }
        });

        // Кнопка добавления нового экрана
        if ui.button(tr(&self.bundle, "screens-create")).clicked() {
            let new_screen = ScreenConfig {
                id: self.next_screen_id,
                title: tr(&self.bundle, "screen-default-title"),
                subtitle: String::new(),
                color: Rgba8 {
                    r: 100,
                    g: 150,
                    b: 200,
                    a: 255,
                },
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
        ui.heading(tr(&self.bundle, "intervals-title"));
        ui.small(tr(&self.bundle, "intervals-description"));

        let mut interval_changed = false;
        let mut to_delete_interval: Option<usize> = None;
        let mut to_edit_interval: Option<usize> = None;

        ui.group(|ui| {
            if self.config.intervals.is_empty() {
                ui.label(tr(&self.bundle, "intervals-none"));
            } else {
                for (idx, interval) in self.config.intervals.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&interval.name);
                            ui.label(format!(
                                "{}:{:02} — {}:{:02}",
                                interval.start.hour,
                                interval.start.minute,
                                interval.end.hour,
                                interval.end.minute
                            ));

                            // Показываем режим интервала
                            match &interval.mode {
                                IntervalMode::Static { screen_id } => {
                                    if let Some(screen) =
                                        self.config.screens.iter().find(|s| s.id == *screen_id)
                                    {
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
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("🗑")
                                        .on_hover_text(tr(&self.bundle, "interval-delete-tooltip"))
                                        .clicked()
                                    {
                                        to_delete_interval = Some(idx);
                                    }
                                    if ui
                                        .small_button("✏")
                                        .on_hover_text(tr(&self.bundle, "interval-edit-tooltip"))
                                        .clicked()
                                    {
                                        to_edit_interval = Some(idx);
                                    }
                                },
                            );
                        });
                    });
                }
            }
        });

        // Кнопка добавления нового интервала
        if ui.button(tr(&self.bundle, "intervals-create")).clicked() {
            let start_time = if let Some(last) = self.config.intervals.last() {
                last.end
            } else {
                TimeOfDay { hour: 9, minute: 0 }
            };

            let new_interval = TimeInterval {
                id: self.next_interval_id,
                name: tr(&self.bundle, "interval-new-title"),
                start: start_time,
                end: TimeOfDay {
                    hour: start_time.hour + 1,
                    minute: start_time.minute,
                },
                mode: IntervalMode::Static {
                    screen_id: self.config.screens.first().map(|s| s.id).unwrap_or(1),
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
        ui.heading(tr(&self.bundle, "validation-title"));

        let validation_errors = validate_intervals(&self.config.intervals);
        if !validation_errors.is_empty() {
            ui.group(|ui| {
                ui.strong(tr(&self.bundle, "validation-problems-found"));
                for error in &validation_errors {
                    ui.small(error);
                }
            });
        } else {
            ui.group(|ui| {
                ui.strong(tr(&self.bundle, "validation-ok"));
                ui.small(tr(&self.bundle, "validation-all-correct"));
            });
        }

        // Показываем расписание переходов на день
        ui.separator();
        ui.heading(tr(&self.bundle, "schedule-title"));

        let transitions = get_daily_transitions(&self.config);
        if transitions.is_empty() {
            ui.small(tr(&self.bundle, "schedule-none"));
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
                tr(&self.bundle, "interval-editor-new")
            } else {
                tr(&self.bundle, "interval-editor-edit")
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
                        egui::FontId::new(16.0, egui::FontFamily::Proportional),
                    );

                    // Название интервала
                    ui.horizontal(|ui| {
                        ui.label(tr(&self.bundle, "interval-name-field"));
                        ui.text_edit_singleline(&mut editing.interval.name);
                    });

                    // Время работы интервала
                    ui.group(|ui| {
                        ui.strong(tr(&self.bundle, "interval-time-work"));
                        ui.horizontal(|ui| {
                            ui.label(tr(&self.bundle, "interval-time-from"));
                            ui.add(
                                egui::DragValue::new(&mut editing.interval.start.hour)
                                    .range(0..=23)
                                    .speed(1.0),
                            );
                            ui.label(":");
                            ui.add(
                                egui::DragValue::new(&mut editing.interval.start.minute)
                                    .range(0..=59)
                                    .speed(1.0),
                            );

                            ui.label(tr(&self.bundle, "interval-time-to"));

                            ui.add(
                                egui::DragValue::new(&mut editing.interval.end.hour)
                                    .range(0..=23)
                                    .speed(1.0),
                            );
                            ui.label(":");
                            ui.add(
                                egui::DragValue::new(&mut editing.interval.end.minute)
                                    .range(0..=59)
                                    .speed(1.0),
                            );
                        });

                        // Показать длительность
                        let duration_minutes = if editing.interval.end.to_minutes()
                            > editing.interval.start.to_minutes()
                        {
                            editing.interval.end.to_minutes() - editing.interval.start.to_minutes()
                        } else {
                            0
                        };
                        let hours = duration_minutes / 60;
                        let minutes = duration_minutes % 60;
                        let mut args = fluent_bundle::FluentArgs::new();
                        args.set("hours", hours);
                        args.set("minutes", minutes);
                        ui.small(tr_with_args(
                            &self.bundle,
                            "interval-duration-format",
                            Some(&args),
                        ));
                    });

                    ui.separator();

                    // Режим интервала
                    ui.strong(tr(&self.bundle, "interval-mode-work"));

                    let is_static = matches!(editing.interval.mode, IntervalMode::Static { .. });

                    ui.horizontal(|ui| {
                        if ui
                            .radio(is_static, tr(&self.bundle, "interval-mode-static-radio"))
                            .on_hover_text(tr(&self.bundle, "interval-mode-static-tooltip"))
                            .clicked()
                            && !is_static
                        {
                            editing.interval.mode = IntervalMode::Static {
                                screen_id: self.config.screens.first().map(|s| s.id).unwrap_or(1),
                            };
                        }
                        if ui
                            .radio(!is_static, tr(&self.bundle, "interval-mode-cycle-radio"))
                            .on_hover_text(tr(&self.bundle, "interval-mode-cycle-tooltip"))
                            .clicked()
                            && is_static
                        {
                            editing.interval.mode = IntervalMode::Cycle { steps: vec![] };
                        }
                    });

                    ui.separator();

                    // Настройка в зависимости от режима
                    match &mut editing.interval.mode {
                        IntervalMode::Static { screen_id } => {
                            ui.label(tr(&self.bundle, "interval-screen-choose"));
                            let screen_name = self
                                .config
                                .screens
                                .iter()
                                .find(|s| s.id == *screen_id)
                                .map(|s| s.title.clone())
                                .unwrap_or_else(|| {
                                    tr(&self.bundle, "interval-screen-choose-placeholder")
                                });

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
                            ui.label(tr(&self.bundle, "interval-steps-configure"));

                            let mut to_remove: Option<usize> = None;

                            for (idx, step) in steps.iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let mut args = fluent_bundle::FluentArgs::new();
                                        args.set("number", idx + 1);
                                        ui.strong(tr_with_args(
                                            &self.bundle,
                                            "interval-step-number",
                                            Some(&args),
                                        ));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("🗑").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            },
                                        );
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(tr(&self.bundle, "interval-step-screen-label"));
                                        let screen_name = self
                                            .config
                                            .screens
                                            .iter()
                                            .find(|s| s.id == step.screen_id)
                                            .map(|s| s.title.clone())
                                            .unwrap_or_else(|| {
                                                tr(&self.bundle, "interval-step-choose")
                                            });

                                        egui::ComboBox::from_id_salt(format!(
                                            "cycle_screen_{}",
                                            idx
                                        ))
                                        .selected_text(&screen_name)
                                        .width(120.0)
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for screen in &self.config.screens {
                                                    ui.selectable_value(
                                                        &mut step.screen_id,
                                                        screen.id,
                                                        &screen.title,
                                                    );
                                                }
                                            },
                                        );

                                        ui.label(tr(&self.bundle, "interval-step-duration-label"));
                                        ui.add(
                                            egui::DragValue::new(&mut step.duration_minutes)
                                                .range(1..=480)
                                                .speed(1.0)
                                                .suffix(" мин"),
                                        );
                                    });
                                });
                            }

                            if let Some(idx) = to_remove {
                                steps.remove(idx);
                            }

                            if ui.button(tr(&self.bundle, "interval-step-add")).clicked() {
                                steps.push(CycleStep {
                                    screen_id: self
                                        .config
                                        .screens
                                        .first()
                                        .map(|s| s.id)
                                        .unwrap_or(1),
                                    duration_minutes: 25,
                                });
                            }

                            if !steps.is_empty() {
                                let total_duration: u32 =
                                    steps.iter().map(|s| s.duration_minutes).sum();
                                let mut args = fluent_bundle::FluentArgs::new();
                                args.set("minutes", total_duration);
                                ui.small(tr_with_args(
                                    &self.bundle,
                                    "interval-cycle-duration",
                                    Some(&args),
                                ));
                            }
                        }
                    }

                    ui.separator();

                    // Кнопки управления
                    ui.horizontal(|ui| {
                        if ui.button(tr(&self.bundle, "btn-save")).clicked() {
                            should_save = true;
                        }
                        if ui.button(tr(&self.bundle, "btn-cancel")).clicked() {
                            should_close = true;
                        }
                    });
                });

            if should_save {
                if editing.is_new {
                    self.config.intervals.push(editing.interval.clone());
                } else {
                    if let Some(idx) = self
                        .config
                        .intervals
                        .iter()
                        .position(|i| i.id == editing.interval.id)
                    {
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
                        if ui.button(tr(&self.bundle, "btn-save")).clicked() {
                            should_save = true;
                        }
                        if ui.button(tr(&self.bundle, "btn-cancel")).clicked() {
                            should_close = true;
                        }
                    });
                });

            if should_save {
                if editing.is_new {
                    self.config.screens.push(editing.screen.clone());
                } else {
                    if let Some(idx) = self
                        .config
                        .screens
                        .iter()
                        .position(|s| s.id == editing.screen.id)
                    {
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
                "ru-RU" | "ru" => tr(&self.bundle, "language-russian"),
                _ => "English".to_string(),
            };

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("system_language")
                    .selected_text(current_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "en-US".to_owned(), "English");
                        ui.selectable_value(
                            &mut selected,
                            "ru-RU".to_owned(),
                            tr(&self.bundle, "language-russian"),
                        );
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

            if ui
                .checkbox(&mut self.config.system_settings.autostart, "")
                .changed()
            {
                settings_changed = true;
            }
        });

        ui.separator();

        // Звуковые уведомления
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "system-sounds"));
            ui.small(tr(&self.bundle, "system-sounds-desc"));

            if ui
                .checkbox(&mut self.config.system_settings.sound_notifications, "")
                .changed()
            {
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
                    self.config.system_settings.window_position =
                        Some(crate::config::WindowPosition { x: 100.0, y: 100.0 });
                } else if !remember_position && has_position {
                    // Забыть позицию
                    self.config.system_settings.window_position = None;
                }
                settings_changed = true;
            }

            if let Some(pos) = &mut self.config.system_settings.window_position {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui
                        .add(egui::DragValue::new(&mut pos.x).range(0.0..=2000.0))
                        .changed()
                    {
                        settings_changed = true;
                    }
                    ui.label("Y:");
                    if ui
                        .add(egui::DragValue::new(&mut pos.y).range(0.0..=2000.0))
                        .changed()
                    {
                        settings_changed = true;
                    }
                });
            }
        });

        ui.separator();

        // Информация о версии и разработчике
        ui.group(|ui| {
            ui.strong(tr(&self.bundle, "app-title"));
            ui.label(tr(&self.bundle, "app-version"));
            ui.small(tr(&self.bundle, "app-description"));

            ui.separator();
            ui.strong(tr(&self.bundle, "hotkeys-title"));
            ui.small(tr(&self.bundle, "hotkey-settings"));
        });

        // Автосохранение при изменениях
        if settings_changed {
            let _ = crate::config::save_config(&self.config_path, &self.config);
        }
    }
}
