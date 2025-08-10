use chrono::{Local, Timelike, DateTime, Duration};
use std::time::SystemTime;

use crate::config::{AppConfig, Rgba8, TimeInterval, IntervalMode};

#[derive(Debug, Clone)]
pub struct ActiveScreenInfo {
    pub title: String,
    pub subtitle: String,
    pub color: Rgba8,
    pub remaining_seconds: u64,
    pub interval_name: String,
    pub screen_id: u32,
    pub is_default_screen: bool,
}

#[derive(Debug, Clone)]
pub struct TimerState {
    pub current_screen: Option<ActiveScreenInfo>,
    pub next_transition: Option<DateTime<Local>>,
    pub is_running: bool,
    pub last_update: SystemTime,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            current_screen: None,
            next_transition: None,
            is_running: true,
            last_update: SystemTime::now(),
        }
    }
}

/// Основной планировщик таймера
pub struct TimerScheduler {
    pub state: TimerState,
}

impl TimerScheduler {
    pub fn new() -> Self {
        Self {
            state: TimerState::default(),
        }
    }

    /// Обновляет состояние таймера и возвращает true, если произошли изменения
    pub fn update(&mut self, config: &AppConfig) -> bool {
        let now = Local::now();
        let prev_screen_id = self.state.current_screen.as_ref().map(|s| s.screen_id);
        
        self.state.current_screen = determine_active_screen(config, now);
        self.state.last_update = SystemTime::now();
        
        // Определяем, изменился ли экран
        let current_screen_id = self.state.current_screen.as_ref().map(|s| s.screen_id);
        let screen_changed = prev_screen_id != current_screen_id;
        
        // Вычисляем время следующего перехода
        self.state.next_transition = calculate_next_transition(config, now);
        
        screen_changed
    }

    /// Возвращает true, если таймер должен быть обновлен
    pub fn should_update(&self) -> bool {
        self.state.is_running && 
        self.state.last_update.elapsed().unwrap_or(std::time::Duration::from_secs(0)) >= std::time::Duration::from_secs(1)
    }

    /// Приостанавливает или возобновляет таймер
    pub fn toggle_pause(&mut self) {
        self.state.is_running = !self.state.is_running;
    }

    /// Принудительно обновляет таймер
    pub fn force_update(&mut self, config: &AppConfig) -> bool {
        self.update(config)
    }
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
        interval_name: "Ожидание".to_string(),
        screen_id: screen.id,
        is_default_screen: true,
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
                // Улучшенный расчет оставшегося времени с учетом секунд
                let seconds = if remaining_min > 0 {
                    (remaining_min - 1) as u64 * 60 + (60 - now_sec as u64)
                } else {
                    60 - now_sec as u64
                };
                
                Some(ActiveScreenInfo {
                    title: screen.title.clone(),
                    subtitle: format!("{} (статичный режим)", screen.subtitle),
                    color: screen.color,
                    remaining_seconds: seconds,
                    interval_name: interval.name.clone(),
                    screen_id: screen.id,
                    is_default_screen: false,
                })
            } else {
                // Если экран не найден, показываем экран по умолчанию
                let default_screen = cfg.screens.first()?;
                let remaining_min = end_min.saturating_sub(now_min);
                let seconds = if remaining_min > 0 {
                    (remaining_min - 1) as u64 * 60 + (60 - now_sec as u64)
                } else {
                    60 - now_sec as u64
                };
                
                Some(ActiveScreenInfo {
                    title: format!("⚠ Экран не найден (ID: {})", screen_id),
                    subtitle: "Используется экран по умолчанию".to_string(),
                    color: default_screen.color,
                    remaining_seconds: seconds,
                    interval_name: interval.name.clone(),
                    screen_id: default_screen.id,
                    is_default_screen: false,
                })
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
                        
                        // Улучшенный расчет времени с учетом секунд
                        let seconds = if remaining_minutes > 0 {
                            (remaining_minutes - 1) as u64 * 60 + (60 - now_sec as u64)
                        } else {
                            60 - now_sec as u64
                        };
                        
                        // Показываем информацию о шаге в подзаголовке
                        let step_info = format!("Шаг {}/{} (цикл)", 
                            steps.iter().position(|s| s.screen_id == step.screen_id).unwrap_or(0) + 1,
                            steps.len()
                        );
                        let subtitle = if screen.subtitle.is_empty() {
                            step_info
                        } else {
                            format!("{} — {}", screen.subtitle, step_info)
                        };
                        
                        return Some(ActiveScreenInfo {
                            title: screen.title.clone(),
                            subtitle,
                            color: screen.color,
                            remaining_seconds: seconds,
                            interval_name: interval.name.clone(),
                            screen_id: screen.id,
                            is_default_screen: false,
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

/// Вычисляет время следующего перехода между экранами
pub fn calculate_next_transition(cfg: &AppConfig, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let now_min = (now.hour() as u32) * 60 + (now.minute() as u32);
    
    // Проверяем, находимся ли мы в активном интервале
    for interval in &cfg.intervals {
        let start_min = interval.start.to_minutes();
        let end_min = interval.end.to_minutes();
        
        if start_min <= now_min && now_min < end_min {
            // Мы в активном интервале
            match &interval.mode {
                IntervalMode::Static { .. } => {
                    // В статичном режиме следующий переход - конец интервала
                    let end_time = now.date_naive().and_hms_opt(
                        (interval.end.hour) as u32, 
                        interval.end.minute as u32, 
                        0
                    )?;
                    return Some(end_time.and_local_timezone(Local).single()?);
                }
                IntervalMode::Cycle { steps } => {
                    // В циклическом режиме находим следующий переход
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
                            // Следующий переход - конец текущего шага
                            let minutes_to_next = next_acc - pos_in_cycle;
                            let next_time = now + Duration::minutes(minutes_to_next as i64);
                            
                            // Но не позже конца интервала
                            let end_time = now.date_naive().and_hms_opt(
                                interval.end.hour as u32, 
                                interval.end.minute as u32, 
                                0
                            )?;
                            let end_datetime = end_time.and_local_timezone(Local).single()?;
                            
                            return Some(next_time.min(end_datetime));
                        }
                        acc = next_acc;
                    }
                }
            }
        }
    }
    
    // Мы вне всех интервалов - следующий переход это начало ближайшего интервала
    let mut next_start: Option<(u32, &TimeInterval)> = None;
    for interval in &cfg.intervals {
        let start = interval.start.to_minutes();
        if start > now_min {
            next_start = Some(match next_start { 
                Some((ns, _)) if ns < start => next_start.unwrap(),
                _ => (start, interval)
            });
        }
    }
    
    if let Some((_, interval)) = next_start {
        let start_time = now.date_naive().and_hms_opt(
            interval.start.hour as u32, 
            interval.start.minute as u32, 
            0
        )?;
        return Some(start_time.and_local_timezone(Local).single()?);
    }
    
    None
}

pub fn format_duration_hhmmss(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 { format!("{:02}:{:02}:{:02}", h, m, s) } else { format!("{:02}:{:02}", m, s) }
}

/// Форматирует время до следующего перехода
pub fn format_time_until_transition(next_transition: Option<DateTime<Local>>) -> String {
    if let Some(next) = next_transition {
        let now = Local::now();
        if next > now {
            let duration = next - now;
            let total_seconds = duration.num_seconds() as u64;
            return format_duration_hhmmss(total_seconds);
        }
    }
    "—".to_string()
}

/// Валидирует интервалы на предмет пересечений и корректности
pub fn validate_intervals(intervals: &[TimeInterval]) -> Vec<String> {
    let mut errors = Vec::new();
    
    // Проверяем каждый интервал на корректность
    for (idx, interval) in intervals.iter().enumerate() {
        let start_min = interval.start.to_minutes();
        let end_min = interval.end.to_minutes();
        
        // Проверяем, что начало раньше конца
        if start_min >= end_min {
            errors.push(format!(
                "Интервал '{}': время начала ({:02}:{:02}) должно быть раньше времени окончания ({:02}:{:02})",
                interval.name,
                interval.start.hour, interval.start.minute,
                interval.end.hour, interval.end.minute
            ));
        }
        
        // Проверяем пересечения с другими интервалами
        for (other_idx, other_interval) in intervals.iter().enumerate() {
            if idx != other_idx {
                let other_start = other_interval.start.to_minutes();
                let other_end = other_interval.end.to_minutes();
                
                // Проверяем пересечение
                if start_min < other_end && end_min > other_start {
                    errors.push(format!(
                        "Интервалы '{}' и '{}' пересекаются по времени",
                        interval.name, other_interval.name
                    ));
                }
            }
        }
        
        // Проверяем корректность режимов
        match &interval.mode {
            IntervalMode::Cycle { steps } => {
                if steps.is_empty() {
                    errors.push(format!(
                        "Интервал '{}': циклический режим должен содержать хотя бы один шаг",
                        interval.name
                    ));
                } else {
                    let total_duration: u32 = steps.iter().map(|s| s.duration_minutes).sum();
                    if total_duration == 0 {
                        errors.push(format!(
                            "Интервал '{}': общая длительность шагов не может быть нулевой",
                            interval.name
                        ));
                    }
                }
            }
            IntervalMode::Static { .. } => {
                // Для статичного режима дополнительных проверок пока не требуется
            }
        }
    }
    
    errors
}

/// Получает список всех переходов в течение дня
pub fn get_daily_transitions(cfg: &AppConfig) -> Vec<(u32, String, String)> {
    let mut transitions = Vec::new();
    
    // Добавляем начала и концы интервалов
    for interval in &cfg.intervals {
        transitions.push((
            interval.start.to_minutes(),
            format!("Начало: {}", interval.name),
            "start".to_string()
        ));
        
        transitions.push((
            interval.end.to_minutes(),
            format!("Конец: {}", interval.name),
            "end".to_string()
        ));
        
        // Для циклических режимов добавляем переходы между шагами
        if let IntervalMode::Cycle { steps } = &interval.mode {
            let mut acc_minutes = 0;
            for (step_idx, step) in steps.iter().enumerate() {
                acc_minutes += step.duration_minutes;
                let transition_time = interval.start.to_minutes() + acc_minutes;
                
                if transition_time < interval.end.to_minutes() {
                    transitions.push((
                        transition_time,
                        format!("Шаг {}/{} в '{}'", step_idx + 2, steps.len(), interval.name),
                        "step".to_string()
                    ));
                }
            }
        }
    }
    
    // Сортируем по времени
    transitions.sort_by_key(|t| t.0);
    transitions
}


