use chrono::{Local, Timelike};

use crate::config::{AppConfig, Rgba8, ScreenConfig, StaticEntry, TimerMode};

pub struct ActiveScreenInfo {
    pub title: String,
    pub subtitle: String,
    pub color: Rgba8,
    pub remaining_seconds: u64,
}

pub fn determine_active_screen(cfg: &AppConfig, now: chrono::DateTime<Local>) -> Option<ActiveScreenInfo> {
    match &cfg.mode {
        TimerMode::Static { schedule } => {
            let now_min = (now.hour() as u32) * 60 + (now.minute() as u32);
            let mut active: Option<(&ScreenConfig, &StaticEntry)> = None;
            for entry in schedule {
                let start = entry.start.to_minutes();
                let end = entry.end.to_minutes();
                if start <= now_min && now_min < end {
                    if let Some(screen) = cfg.screens.iter().find(|s| s.id == entry.screen_id) {
                        active = Some((screen, entry));
                        break;
                    }
                }
            }
            if let Some((screen, entry)) = active {
                let end_min = entry.end.to_minutes();
                let remaining_min = end_min.saturating_sub(now_min);
                let seconds = remaining_min as u64 * 60 + (60 - now.second() as u64 % 60);
                Some(ActiveScreenInfo {
                    title: screen.title.clone(),
                    subtitle: screen.subtitle.clone(),
                    color: screen.color,
                    remaining_seconds: seconds,
                })
            } else {
                // Вне расписания: экран по умолчанию и до следующего старта
                let screen = cfg
                    .default_screen_id
                    .and_then(|id| cfg.screens.iter().find(|s| s.id == id))
                    .or_else(|| cfg.screens.first())?;

                // Найдём ближайший будущий старт
                let mut next_start: Option<u32> = None;
                for entry in schedule {
                    let start = entry.start.to_minutes();
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
        }
        TimerMode::Interval { range, sequence } => {
            let now_min = (now.hour() as u32) * 60 + (now.minute() as u32);
            let start_min = range.start.to_minutes();
            let end_min = range.end.to_minutes();
            if !(start_min <= now_min && now_min < end_min) {
                // Вне общего диапазона: экран по умолчанию
                let screen = cfg
                    .default_screen_id
                    .and_then(|id| cfg.screens.iter().find(|s| s.id == id))
                    .or_else(|| cfg.screens.first())?;
                let remaining_seconds = ((24 * 60 - now_min) as u64) * 60 + (60 - now.second() as u64 % 60);
                return Some(ActiveScreenInfo {
                    title: screen.title.clone(),
                    subtitle: screen.subtitle.clone(),
                    color: screen.color,
                    remaining_seconds,
                });
            }
            let into_range = now_min - start_min;
            let total_cycle: u32 = sequence.iter().map(|e| e.duration_minutes).sum();
            let pos_in_cycle = if total_cycle > 0 { into_range % total_cycle } else { 0 };
            let mut acc = 0;
            for entry in sequence {
                let next_acc = acc + entry.duration_minutes;
                if pos_in_cycle < next_acc {
                    if let Some(screen) = cfg.screens.iter().find(|s| s.id == entry.screen_id) {
                        let remaining_min_in_item = next_acc - pos_in_cycle;
                        // не выходим за границу общего диапазона
                        let remaining_to_range_end = end_min - now_min;
                        let remaining_minutes = remaining_min_in_item.min(remaining_to_range_end);
                        let seconds = remaining_minutes as u64 * 60 + (60 - now.second() as u64 % 60);
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


