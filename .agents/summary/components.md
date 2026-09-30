# Components
<!-- metadata: type=components, audience=ai-agents, scope=all-modules -->

## Application Core

### TeclazoApplication (`src/application.rs`)
- **Parent**: `adw::Application`
- **Role**: Application entry point. Registers GResource base path, loads CSS, sets keyboard shortcuts (`Ctrl+Q` quit, `Ctrl+W` close), creates the main window on activation.
- **Key behavior**: `startup()` loads global CSS provider; `activate()` creates and presents `TeclazoWindow`.

### TeclazoWindow (`src/window.rs`)
- **Parent**: `adw::ApplicationWindow`
- **Role**: Main window and navigation hub. Contains `adw::NavigationView` with `ActionRow` entries for each feature.
- **Template children**: `header_bar`, `window_title`, `navigation_view`, plus rows for lessons, speed test, falling keys, scrolling lanes, preferences, about.
- **Key behavior**: Each row's `activated` signal pushes the corresponding navigation page by tag. Persists window size/maximized state via GSettings.

## Feature Views

### LessonView (`src/lesson_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Structured typing lessons with step-by-step progression. The most complex view.
- **Contains**: `TypingRow`, `KeyboardWidget`, `HandWidget`
- **Key behavior**: Loads a `Course` based on locale, tracks current lesson/step/repetition via GSettings, highlights relevant keys on the keyboard, shows hand position guidance, advances through steps on completion, shows `CourseCompletionView` when all lessons are done.
- **Properties**: `current_lesson` (boxed), `current_step_index` (u32)

### SpeedTestView (`src/speed_test_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Timed typing speed tests with configurable duration.
- **Contains**: `SpeedTestTextView`, `SpeedTestResultsView`
- **Key behavior**: Generates random text via `text_generation`, starts a timer, tracks progress, shows results (WPM, accuracy, duration) on completion or timeout.

### FallingKeysGame (`src/falling_keys_game.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Gamified typing practice where keys fall from the top of the screen.
- **Contains**: `FallingKeysWidget` (custom painted widget), `KeyboardWidget`
- **Key behavior**: Game loop spawns falling key characters, player must type them before they reach the bottom. Tracks score and lives. Uses `glib::timeout_add_local` for the game loop.
- **Inner types**: `FallingKey` (position, character, speed), `FallingKeysWidget` (custom snapshot rendering)

### ScrollingLanesGame (`src/scrolling_lanes_game.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Gamified typing practice with text scrolling across lanes.
- **Contains**: `LaneWidget` (custom painted widget), `KeyboardWidget`
- **Key behavior**: Multiple lanes with scrolling text that must be typed. Uses `glib::timeout_add_local` for animation. Tracks score and lives.
- **Inner types**: `ScrollingText` (text, position, lane), `LaneWidget` (custom snapshot rendering)

### PreferencesView (`src/preferences_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: User settings for hand widget visibility, keyboard widget visibility, finger colors, and lesson selection.
- **Key behavior**: Binds `adw::SwitchRow` widgets directly to GSettings keys. Populates lesson combo from `Course` data.

### AboutView (`src/about_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Application information, credits, and links.

### CourseCompletionView (`src/course_completion_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Congratulatory view shown when all lessons in a course are completed.

## Reusable Widgets

### TypingRow (`src/typing_row.rs`)
- **Parent**: `adw::PreferencesRow`
- **Role**: Core text input widget used in `LessonView`. Shows target text, captures typed input, validates character-by-character.
- **Signals**: `mistake-made(bool)`, `step-completed`, `next-char-changed(String)`, `dead-key-started`
- **Key behavior**: Locks cursor to end position, validates each keystroke against target text, draws custom cursor overlay, detects dead key input (for accented characters).

### KeyboardWidget (`src/keyboard_widget.rs`)
- **Parent**: `gtk::Widget`
- **Role**: Visual on-screen keyboard that highlights the current key to press and shows finger assignments.
- **Key behavior**: Loads keyboard layout from JSON, custom `snapshot()` rendering of keys with color-coded fingers, supports dead key sequences (accent → base char), handles modifier keys (Shift, AltGr).
- **Inner types**: `KeyboardLayout`, `KeyInfo`, `Finger`, `ModifierKey`

### HandWidget (`src/hand_widget.rs`)
- **Parent**: `gtk::Widget`
- **Role**: Visual hand position guide showing which finger to use.
- **Key behavior**: Custom `snapshot()` rendering of left/right hands with highlighted current finger. Caches theme colors and responds to dark/light mode changes.

### SpeedTestTextView (`src/speed_test_text_view.rs`)
- **Parent**: `adw::Bin` (composite template)
- **Role**: Rich text display for speed tests with color-coded correct/incorrect characters, animated caret, and scrolling.
- **Sub-modules**: `accessibility.rs`, `caret.rs`, `colors.rs`, `input.rs`, `scrolling.rs`
- **Key behavior**: Manages original vs typed text comparison, renders colored text via GTK TextBuffer tags, animates caret position, handles IME input, auto-scrolls as user types.
- **Signals**: `typed-text-changed`, `push-original-text`, `set-original-text`

### SpeedTestResultsView (`src/speed_test_results_view.rs`)
- **Parent**: `adw::NavigationPage`
- **Role**: Displays speed test results (WPM, accuracy, duration).
- **Signals**: `retry-clicked`

## Data & Utility Modules

### Course (`src/course.rs`)
- **Role**: Data model for structured typing lessons. Loads lesson JSON files based on language.
- **Types**: `Course`, `Lesson`, `LessonStep`, `LessonsData`
- **Key behavior**: `new_with_language()` loads embedded JSON via `include_str!`. Falls back to US English for unknown languages.

### text_generation (`src/text_generation.rs`)
- **Role**: Generates random typing text from embedded word lists.
- **Types**: `Language` (enum with 30+ variants), `Punctuation`
- **Key behavior**: Loads word lists via `include_dir!`, generates text with configurable difficulty (simple/advanced), supports punctuation insertion, uppercase, and wrapping.

### text_utils (`src/text_utils.rs`)
- **Role**: Text validation, WPM calculation, and character comparison utilities.
- **Types**: `GraphemeState` (Correct/Unfinished/Mistake)
- **Key behavior**: Grapheme-level validation with Unicode support, handles character aliases (æ→ae, guillemets→quotes, non-breaking spaces), calculates WPM from correct graphemes.

### typing_test_utils (`src/typing_test_utils.rs`)
- **Role**: Speed test configuration and result summary types.
- **Types**: `TestConfig`, `TestDuration`, `TestSummary`, `GeneratedTestDifficulty`

### utils (`src/utils.rs`)
- **Role**: Locale detection, Unicode decomposition, and key extraction utilities.
- **Key functions**: `language_from_locale()` (maps LANG env var to language code), `decompose_with_spacing_accent()` (for dead key handling), `extract_keys()` (unique characters from text).
