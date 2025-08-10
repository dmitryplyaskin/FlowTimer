hello-world = Привет, мир!
menu-language = Язык

settings-title = Настройки
tab-timers = Таймеры и экраны
tab-system = Системные настройки
timers-mode = Режим работы
mode-static = Статичный
mode-interval = Интервальный
timers-screens = Экраны
btn-add = Добавить
btn-delete = Удалить
screen-new = Новый экран
screen-edit = Редактор экрана
field-title = Заголовок
field-subtitle = Подзаголовок
field-color = Цвет
timers-params = Временные параметры
btn-add-interval = Добавить интервал
col-screen = Экран
col-start = Начало
col-end = Конец
select = Выбрать
interval-range = Общий диапазон
btn-add-seq = Добавить шаг
col-duration = Длительность (мин)

# Системные настройки
system-language = Язык интерфейса
system-autostart = Автозапуск с системой
system-sounds = Звуковые уведомления
system-window-pos = Положение окна на экране
system-language-desc = Выберите язык интерфейса приложения
system-autostart-desc = Автоматически запускать приложение при старте системы
system-sounds-desc = Воспроизводить звуки при смене экранов
system-window-pos-desc = Запомнить положение окна на экране
btn-save = Сохранить
btn-cancel = Отмена
settings-saved = Настройки сохранены

# Основной интерфейс
main-no-screens = Нет настроенных экранов
main-no-screens-hint = Откройте настройки для создания экранов и интервалов
interval-label = Интервал: { $name }
next-transition = Следующий переход через: { $time }
timer-pause = ⏸ Пауза
timer-continue = ▶ Продолжить
timer-refresh = 🔄 Обновить

# Управление экранами
screens-title = Экраны
screens-description = Экраны определяют цвет фона и текст, которые будут показываться
screens-none = Нет созданных экранов
screens-create = ➕ Создать новый экран
screen-delete-tooltip = Удалить экран
screen-edit-tooltip = Редактировать экран
screen-new-title = Новый экран

# Управление интервалами
intervals-title = Временные интервалы
intervals-description = Каждый интервал имеет свое время работы и режим отображения экранов
intervals-none = Нет созданных интервалов
intervals-create = ➕ Создать новый интервал
interval-delete-tooltip = Удалить интервал
interval-edit-tooltip = Редактировать интервал
interval-new-title = Новый интервал
interval-static-mode = (статичный)
interval-cycle-mode = (цикл из { $steps } шагов)

# Валидация и расписание
validation-title = Валидация настроек
validation-problems = ⚠ Обнаружены проблемы в настройках:
validation-ok = ✓ Настройки корректны
validation-ok-desc = Все интервалы настроены правильно
schedule-title = Расписание переходов
schedule-none = Нет настроенных переходов

# Редактор интервалов
interval-editor-new = Новый интервал
interval-editor-edit = Редактирование интервала
interval-name-label = Название:
interval-time-title = Время работы
interval-time-from = С
interval-time-to = до
interval-duration = Длительность: { $hours } ч { $minutes } мин
interval-mode-title = Режим работы интервала
interval-mode-static = Статичный
interval-mode-static-desc = Показывает один экран весь интервал
interval-mode-cycle = Циклический
interval-mode-cycle-desc = Циклически переключает экраны
interval-screen-select = Выберите экран для отображения:
interval-screen-placeholder = Выберите экран
interval-steps-title = Настройте последовательность экранов:
interval-step-title = Шаг { $number }
interval-step-screen = Экран:
interval-step-duration = Длительность:
interval-step-select = Выберите
interval-step-add = ➕ Добавить шаг
interval-cycle-total = Общая длительность цикла: { $minutes } мин

# Экран по умолчанию
default-waiting = Ожидание
default-fallback = Используется экран по умолчанию
screen-not-found = ⚠ Экран не найден (ID: { $id })

# Режимы экранов
static-mode-suffix = (статичный режим)
cycle-step-info = Шаг { $current }/{ $total } (цикл)

# Валидация ошибок
validation-time-order = Интервал '{ $name }': время начала ({ $start }) должно быть раньше времени окончания ({ $end })
validation-overlap = Интервалы '{ $first }' и '{ $second }' пересекаются по времени
validation-empty-cycle = Интервал '{ $name }': циклический режим должен содержать хотя бы один шаг
validation-zero-duration = Интервал '{ $name }': общая длительность шагов не может быть нулевой

# Переходы
transition-start = Начало: { $name }
transition-end = Конец: { $name }
transition-step = Шаг { $step }/{ $total } в '{ $interval }'

# Приложение
app-title = О приложении
app-version = FlowTimer v0.1.0
app-description = Приложение для визуального отображения временных интервалов
app-copyright = © 2024 Pet Projects

# Экраны по умолчанию
default-screen-work = Работа
default-screen-work-subtitle = Фокус
default-screen-break = Перерыв
default-screen-break-subtitle = Отдых
default-screen-prep = Подготовка
default-interval-morning = Утренняя работа
default-interval-pomodoro = Помодоро сессия