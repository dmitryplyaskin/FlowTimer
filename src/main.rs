#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "FlowTimer",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    current_lang: LanguageIdentifier,
    bundle: FluentBundle<FluentResource>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // По умолчанию русский. При желании можно начать с en-US
        let current_lang: LanguageIdentifier = "ru-RU".parse().unwrap();
        let bundle = make_bundle(&current_lang);
        Self { current_lang, bundle }
    }

    fn set_language(&mut self, lang_tag: &str) {
        if let Ok(parsed) = lang_tag.parse::<LanguageIdentifier>() {
            if parsed != self.current_lang {
                self.current_lang = parsed;
                self.bundle = make_bundle(&self.current_lang);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let label = tr(&self.bundle, "menu-language");
                let mut selected = self.current_lang.to_string();
                egui::ComboBox::from_label(label)
                    .selected_text(match selected.as_str() {
                        "ru-RU" | "ru" => "Русский",
                        _ => "English",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "en-US".to_owned(), "English");
                        ui.selectable_value(&mut selected, "ru-RU".to_owned(), "Русский");
                    });

                if selected != self.current_lang.to_string() {
                    self.set_language(&selected);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(tr(&self.bundle, "hello-world"));
        });
    }
}

fn make_bundle(lang: &LanguageIdentifier) -> FluentBundle<FluentResource> {
    // Выбираем встроенную строку-ресурс по текущему языку
    let ftl: &str = match lang.to_string().as_str() {
        "ru-RU" | "ru" => include_str!("../locales/ru-RU.ftl"),
        _ => include_str!("../locales/en-US.ftl"),
    };

    let resource = FluentResource::try_new(ftl.to_string())
        .expect("Некорректный формат FTL ресурса");

    let mut bundle = FluentBundle::new(vec![lang.clone()]);
    bundle
        .add_resource(resource)
        .expect("Не удалось добавить FTL ресурс в bundle");
    bundle
}

fn tr(bundle: &FluentBundle<FluentResource>, id: &str) -> String {
    if let Some(msg) = bundle.get_message(id) {
        if let Some(pattern) = msg.value() {
            let mut errors = vec![];
            let value = bundle.format_pattern(pattern, None, &mut errors);
            return value.into_owned();
        }
    }
    // Fallback на ключ
    id.to_string()
}
