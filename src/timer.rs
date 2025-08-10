use chrono::{Local, Timelike};

use crate::config::{AppConfig, Rgba8, TimeInterval, IntervalMode};

pub struct ActiveScreenInfo {
    pub title: String,
    pub subtitle: String,
    pub color: Rgba8,
    pub remaining_seconds: u64,
}

pub fn determine_active_screen(cfg: &AppConfig, now: chrono::DateTime<Local>) -> Option<ActiveScreenInfo> {
    let now_min = (now.hour() as u32) * 60 + (now.minute() as u32);
    
    // Найдем активный интервал
    for interval in &cfg.intervals {
        let start_min = interval.start.to_minutes();
        let end_min = interval.end.to_minutes();
        
        if start_min <= now_min && now_min < end_min {
            // Этот интервал активен
            return determine_screen_in_interval(cfg, interval, now_min, now.second());
        }
    }
    
    // Вне всех интервалов - показываем экран по умолчанию
    let screen = cfg
        .default_screen_id
        .and_then(|id| cfg.screens.iter().find(|s| s.id == id))
        .or_else(|| cfg.screens.first())?;

    // Найдем ближайший будущий интервал
    let mut next_start: Option<u32> = None;
    for interval in &cfg.intervals {
        let start = interval.start.to_minutes();
        if start > now_min {
            next_start = Some(match next_start { Some(ns) => ns.min(start), None => start });
        }
    }
    
    let remaining_seconds = if let Some(ns) = next_start {
        ((ns - now_min) as u64) * 60 + (60 - now.second() as u64 % 60)
    } else {
        // до конца дня
        ((24 * 60 - now_min) as u64) * 60 + (60 - now.second() as u64 % 60)
    };
    
    Some(ActiveScreenInfo {
        title: screen.title.clone(),
        subtitle: screen.subtitle.clone(),
        color: screen.color,
        remaining_seconds,
    })
}

fn determine_screen_in_interval(cfg: &AppConfig, interval: &TimeInterval, now_min: u32, now_sec: u32) -> Option<ActiveScreenInfo> {
    let start_min = interval.start.to_minutes();
    let end_min = interval.end.to_minutes();
    
    match &interval.mode {
        IntervalMode::Static { screen_id } => {
            // Статичный режим - показываем один экран весь интервал
            if let Some(screen) = cfg.screens.iter().find(|s| s.id == *screen_id) {
                let remaining_min = end_min.saturating_sub(now_min);
                let seconds = remaining_min as u64 * 60 + (60 - now_sec as u64 % 60);
                Some(ActiveScreenInfo {
                    title: screen.title.clone(),
                    subtitle: screen.subtitle.clone(),
                    color: screen.color,
                    remaining_seconds: seconds,
                })
            } else {
                None
            }
        }
        IntervalMode::Cycle { steps } => {
            // Циклический режим - переключаем экраны по шагам
            if steps.is_empty() {
                return None;
            }
            
            let into_interval = now_min - start_min;
            let total_cycle: u32 = steps.iter().map(|s| s.duration_minutes).sum();
            
            if total_cycle == 0 {
                return None;
            }
            
            let pos_in_cycle = into_interval % total_cycle;
            let mut acc = 0;
            
            for step in steps {
                let next_acc = acc + step.duration_minutes;
                if pos_in_cycle < next_acc {
                    if let Some(screen) = cfg.screens.iter().find(|s| s.id == step.screen_id) {
                        let remaining_in_step = next_acc - pos_in_cycle;
                        // Не выходим за границу интервала
                        let remaining_to_interval_end = end_min - now_min;
                        let remaining_minutes = remaining_in_step.min(remaining_to_interval_end);
                        let seconds = remaining_minutes as u64 * 60 + (60 - now_sec as u64 % 60);
                        
                        return Some(ActiveScreenInfo {
                            title: screen.title.clone(),
                            subtitle: screen.subtitle.clone(),
                            color: screen.color,
                            remaining_seconds: seconds,
                        });
                    }
                    break;
                }
                acc = next_acc;
            }
            None
        }
    }
}

pub fn format_duration_hhmmss(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 { format!("{:02}:{:02}:{:02}", h, m, s) } else { format!("{:02}:{:02}", m, s) }
}


