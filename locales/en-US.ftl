hello-world = Hello, world!
menu-language = Language

settings-title = Settings
tab-timers = Timers & Screens
tab-system = System Settings
timers-mode = Mode
mode-static = Static
mode-interval = Interval
timers-screens = Screens
btn-add = Add
btn-delete = Delete
screen-new = New screen
screen-edit = Screen editor
field-title = Title
field-subtitle = Subtitle
field-color = Color
timers-params = Time parameters
btn-add-interval = Add interval
col-screen = Screen
col-start = Start
col-end = End
select = Select
interval-range = Global range
btn-add-seq = Add step
col-duration = Duration (min)

# System Settings
system-language = Interface language
system-autostart = Autostart with system
system-sounds = Sound notifications
system-window-pos = Window position on screen
system-language-desc = Choose the application interface language
system-autostart-desc = Automatically start the application on system startup
system-sounds-desc = Play sounds when switching screens
system-window-pos-desc = Remember window position on screen
btn-save = Save
btn-cancel = Cancel
settings-saved = Settings saved

# Main interface
main-no-screens = No configured screens
main-no-screens-hint = Open settings to create screens and intervals
interval-label = Interval: { $name }
next-transition = Next transition in: { $time }
timer-pause = ⏸ Pause
timer-continue = ▶ Continue
timer-refresh = 🔄 Refresh

# Screen management
screens-title = Screens
screens-description = Screens define background color and text to be displayed
screens-none = No screens created
screens-create = ➕ Create new screen
screen-delete-tooltip = Delete screen
screen-edit-tooltip = Edit screen
screen-new-title = New screen

# Interval management
intervals-title = Time intervals
intervals-description = Each interval has its own working time and screen display mode
intervals-none = No intervals created
intervals-create = ➕ Create new interval
interval-delete-tooltip = Delete interval
interval-edit-tooltip = Edit interval
interval-new-title = New interval
interval-static-mode = (static)
interval-cycle-mode = (cycle of { $steps } steps)

# Validation and schedule
validation-title = Settings validation
validation-problems = ⚠ Issues found in settings:
validation-ok = ✓ Settings are correct
validation-ok-desc = All intervals are configured properly
schedule-title = Transition schedule
schedule-none = No configured transitions

# Interval editor
interval-editor-new = New interval
interval-editor-edit = Edit interval
interval-name-label = Name:
interval-time-title = Working time
interval-time-from = From
interval-time-to = to
interval-duration = Duration: { $hours } h { $minutes } min
interval-mode-title = Interval working mode
interval-mode-static = Static
interval-mode-static-desc = Shows one screen for the entire interval
interval-mode-cycle = Cyclic
interval-mode-cycle-desc = Cyclically switches screens
interval-screen-select = Select screen to display:
interval-screen-placeholder = Select screen
interval-steps-title = Configure screen sequence:
interval-step-title = Step { $number }
interval-step-screen = Screen:
interval-step-duration = Duration:
interval-step-select = Select
interval-step-add = ➕ Add step
interval-cycle-total = Total cycle duration: { $minutes } min

# Default screen
default-waiting = Waiting
default-fallback = Using default screen
screen-not-found = ⚠ Screen not found (ID: { $id })

# Screen modes
static-mode-suffix = (static mode)
cycle-step-info = Step { $current }/{ $total } (cycle)

# Validation errors
validation-time-order = Interval '{ $name }': start time ({ $start }) must be earlier than end time ({ $end })
validation-overlap = Intervals '{ $first }' and '{ $second }' overlap in time
validation-empty-cycle = Interval '{ $name }': cyclic mode must contain at least one step
validation-zero-duration = Interval '{ $name }': total duration of steps cannot be zero

# Transitions
transition-start = Start: { $name }
transition-end = End: { $name }
transition-step = Step { $step }/{ $total } in '{ $interval }'

# Application
app-title = About application
app-version = FlowTimer v0.1.0
app-description = Application for visual display of time intervals
app-copyright = © 2024 Pet Projects

# Default screens
default-screen-work = Work
default-screen-work-subtitle = Focus
default-screen-break = Break
default-screen-break-subtitle = Rest
default-screen-prep = Preparation
default-interval-morning = Morning work
default-interval-pomodoro = Pomodoro session