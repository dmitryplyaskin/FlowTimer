use fluent_bundle::{FluentBundle, FluentResource, FluentArgs};
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

/// Локализация с параметрами
pub fn tr_with_args(bundle: &FluentBundle<FluentResource>, id: &str, args: Option<&FluentArgs>) -> String {
    if let Some(msg) = bundle.get_message(id) {
        if let Some(pattern) = msg.value() {
            let mut errors = vec![];
            let value = bundle.format_pattern(pattern, args, &mut errors);
            return value.into_owned();
        }
    }
    id.to_string()
}

/// Создает FluentArgs для одного параметра
pub fn make_args_1<'a>(key: &'a str, value: &'a str) -> FluentArgs<'a> {
    let mut args = FluentArgs::new();
    args.set(key, value);
    args
}

/// Создает FluentArgs для двух параметров
pub fn make_args_2<'a>(key1: &'a str, value1: &'a str, key2: &'a str, value2: &'a str) -> FluentArgs<'a> {
    let mut args = FluentArgs::new();
    args.set(key1, value1);
    args.set(key2, value2);
    args
}

/// Создает FluentArgs для числового параметра
pub fn make_args_num<'a>(key: &'a str, value: i64) -> FluentArgs<'a> {
    let mut args = FluentArgs::new();
    args.set(key, value);
    args
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
