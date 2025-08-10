use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;
use crate::ui::AppState;

pub fn make_bundle(lang: &LanguageIdentifier) -> FluentBundle<FluentResource> {
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

pub fn tr(bundle: &FluentBundle<FluentResource>, id: &str) -> String {
    if let Some(msg) = bundle.get_message(id) {
        if let Some(pattern) = msg.value() {
            let mut errors = vec![];
            let value = bundle.format_pattern(pattern, None, &mut errors);
            return value.into_owned();
        }
    }
    id.to_string()
}

pub fn set_language(app: &mut AppState, lang_tag: &str) {
    if let Ok(parsed) = lang_tag.parse::<LanguageIdentifier>() {
        if parsed != app.config.language {
            app.config.language = parsed;
            app.bundle = make_bundle(&app.config.language);
            let _ = crate::config::save_config(&app.config_path, &app.config);
        }
    }
}
