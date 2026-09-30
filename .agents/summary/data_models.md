# Data Models
<!-- metadata: type=data-models, audience=ai-agents, scope=structs-enums-json -->

## Lesson Data Model

```mermaid
classDiagram
    class Course {
        -lessons: Vec~Lesson~
        +new_with_language(language: &str) Course
        +get_lessons() &Vec~Lesson~
        +get_lesson(id: u32) Option~&Lesson~
        +get_next_lesson(current_id: u32) Option~&Lesson~
    }
    class Lesson {
        +id: u32
        +title: String
        +description: String
        +steps: Vec~LessonStep~
        +introduction: bool
    }
    class LessonStep {
        +id: u32
        +text: String
        +description: Option~String~
        +repetitions: u32
        +introduction: bool
    }
    Course "1" --> "*" Lesson
    Lesson "1" --> "*" LessonStep
```

### Lesson JSON Structure

File location: `data/lessons/{language_code}.json`

```json
{
  "lessons": [
    {
      "id": 1,
      "title": "Home Row",
      "description": "Learn the home row keys",
      "introduction": false,
      "steps": [
        {
          "id": 1,
          "text": "fff jjj fff jjj",
          "description": "Practice f and j keys",
          "repetitions": 3,
          "introduction": false
        }
      ]
    }
  ]
}
```

## Keyboard Layout Data Model

```mermaid
classDiagram
    class KeyboardLayout {
        +rows: Vec~Vec~KeyInfo~~
        +load_from_json(language: &str) KeyboardLayout
        +contains_character(ch: char) bool
        +get_finger_for_char(ch: char) Option~Finger~
    }
    class KeyInfo {
        +base: String
        +shift: Option~String~
        +altgr: Option~String~
        +finger: Finger
        +width: Option~f64~
    }
    class Finger {
        <<enumeration>>
        LeftPinky
        LeftRing
        LeftMiddle
        LeftIndex
        LeftThumb
        RightIndex
        RightMiddle
        RightRing
        RightPinky
        RightThumb
        BothThumbs
    }
    class ModifierKey {
        +label: String
        +finger: Finger
        +rows: Option~Vec~u8~~
        +spans_row(row: u8) bool
    }
    KeyboardLayout "1" --> "*" KeyInfo
    KeyInfo --> Finger
```

### Keyboard Layout JSON Structure

File location: `data/keyboard_layouts/{language_code}.json`

```json
{
  "name": "US QWERTY",
  "keys": [
    [
      {"base": "`", "shift": "~", "finger": "left_pinky"},
      {"base": "1", "shift": "!", "finger": "left_pinky"},
      {"base": "q", "shift": "Q", "finger": "left_pinky", "altgr": "ä"}
    ]
  ],
  "space": {"base": " ", "label": "SPACE", "finger": "both_thumbs"},
  "modifiers": {
    "tab": {"label": "Tab", "finger": "left_pinky"},
    "caps_lock": {"label": "Caps", "finger": "left_pinky"},
    "enter": {"label": "Enter", "finger": "right_pinky", "rows": [2]},
    "backspace": {"label": "Backspace", "finger": "right_pinky"}
  }
}
```

The `rows` field on the `enter` modifier specifies which keyboard rows the Enter key occupies:
- `[1, 2]` — ISO-style tall Enter spanning rows 1–2 (used by ES, GL, IT, PL, PT layouts)
- `[2]` — ANSI-style single-row Enter on row 2 only (used by US layout)

## Speed Test Data Models

```mermaid
classDiagram
    class TestConfig {
        <<enumeration>>
        Finite
        Generated(language, difficulty, duration)
    }
    class TestDuration {
        <<enumeration>>
        Sec15
        Sec30
        Min1
        Min5
        Min10
        +as_seconds() u64
        +ui_string() String
    }
    class GeneratedTestDifficulty {
        <<enumeration>>
        Simple
        Advanced
    }
    class TestSummary {
        +config: TestConfig
        +real_duration: Duration
        +wpm: f64
        +start_timestamp: SystemTime
        +accuracy: f64
    }
    TestConfig --> TestDuration
    TestConfig --> GeneratedTestDifficulty
    TestSummary --> TestConfig
```

## Text Validation Model

```mermaid
classDiagram
    class GraphemeState {
        <<enumeration>>
        Correct
        Unfinished
        Mistake
    }
```

`validate_with_replacements()` returns `Vec<(GraphemeState, line_num, start_byte, end_byte)>` — each tuple maps a grapheme's validation state to its exact position in the displayed text buffer.

## Text Generation Model

```mermaid
classDiagram
    class Language {
        <<enumeration>>
        Arabic
        Bangla
        Bulgarian
        Catalan
        Czech
        Danish
        Dutch
        English (default)
        Estonian
        Finnish
        French
        Galician
        German
        Greek
        Hebrew
        Hindi
        Hungarian
        Indonesian
        Italian
        Kabyle
        Kinyarwanda
        Korean
        ...30+ variants
    }
    class Punctuation {
        <<enumeration>>
        values for punctuation insertion
    }
```

Word lists are embedded at compile time from `data/word_lists/{lang_code}.txt` via `include_dir!`. The `Language` enum maps to file names using `strum` derive macros (`to_string` attribute).

## GSettings State

Persisted user state (not a Rust struct, but a runtime data model):

| Schema | Key | Type | Purpose |
|--------|-----|------|---------|
| `io.github.nacho.teclazo` | `current-lesson` | u32 | Lesson progress |
| `io.github.nacho.teclazo` | `current-step` | u32 | Step progress |
| `io.github.nacho.teclazo` | `show-hand-widget` | bool | Preference |
| `io.github.nacho.teclazo` | `show-keyboard-widget` | bool | Preference |
| `io.github.nacho.teclazo` | `use-finger-colors` | bool | Preference |
| `io.github.nacho.teclazo` | `speed-test-duration` | u32 | Preference |
| `io.github.nacho.teclazo.state.window` | `maximized` | bool | Window state |
| `io.github.nacho.teclazo.state.window` | `size` | (i32, i32) | Window state |
