# Interfaces
<!-- metadata: type=interfaces, audience=ai-agents, scope=signals-properties-apis -->

## GObject Signals

Signals are the primary inter-component communication mechanism in this GTK4 application.

### TypingRow Signals

| Signal | Parameters | Emitted When |
|--------|-----------|-------------|
| `mistake-made` | `bool` (is_mistake) | User types a wrong character |
| `step-completed` | — | User finishes typing the target text |
| `next-char-changed` | `String` (next_char) | The next expected character changes |
| `dead-key-started` | — | A dead key composition sequence begins |

**Consumers**: `LessonView` connects to all four signals to update keyboard highlighting, hand position, repetition tracking, and step advancement.

### SpeedTestTextView Signals

| Signal | Parameters | Emitted When |
|--------|-----------|-------------|
| `typed-text-changed` | — | User modifies typed text |
| `push-original-text` | — | More original text needs to be appended |
| `set-original-text` | — | Original text is fully replaced |

**Consumers**: `SpeedTestView` connects to track progress and trigger text generation.

### SpeedTestResultsView Signals

| Signal | Parameters | Emitted When |
|--------|-----------|-------------|
| `retry-clicked` | — | User clicks the retry button |

**Consumers**: `SpeedTestView` connects to reset the test.

## GObject Properties

### LessonView Properties

| Property | Type | Description |
|----------|------|-------------|
| `current-lesson` | `BoxedAnyObject` (nullable) | Currently active Lesson object |
| `current-step-index` | `u32` | Index of current step within the lesson |

### SpeedTestTextView Properties

| Property | Type | Description |
|----------|------|-------------|
| `caret-x` | `f64` | Caret X position for animation |
| `caret-y` | `f64` | Caret Y position for animation |
| `caret-height` | `f64` | Caret height |
| `running` | `bool` | Whether the test is actively running |
| `accepts-input` | `bool` | Whether input is currently accepted |

## GSettings Schema

Schema ID: `io.github.nacho.teclazo`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `current-lesson` | `u` | 0 | Current lesson number |
| `current-step` | `u` | 0 | Current step within lesson |
| `show-hand-widget` | `b` | true | Show hand position guide |
| `show-keyboard-widget` | `b` | true | Show on-screen keyboard |
| `use-finger-colors` | `b` | false | Color-code keys by finger |
| `speed-test-duration` | `u` | 1 | Speed test duration index |

Schema ID: `io.github.nacho.teclazo.state.window`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `maximized` | `b` | false | Window maximized state |
| `size` | `(ii)` | (1024, 768) | Window dimensions |

## Public API Patterns

### KeyboardWidget

```rust
// Set the currently highlighted key
fn set_current_key(&self, key: &str)
// Set which keys are visible/relevant for current lesson step
fn set_visible_keys(&self, keys: &HashSet<char>)
// Advance dead key sequence (accent → base character)
fn advance_sequence(&self)
// Get finger assignment for a character
fn get_finger_for_char(&self, ch: char) -> Option<Finger>
```

### HandWidget

```rust
// Set which finger to highlight
fn set_current_finger(&self, finger: Option<Finger>)
```

### TypingRow

```rust
// Set the target text to type
fn set_target_text(&self, text: &str)
// Set repetition indicator text
fn set_repetition_text(&self, text: &str)
// Clear input and reset state
fn clear(&self)
// Show error animation
fn show_error(&self)
```

### Course

```rust
// Load course for a language (falls back to US English)
fn new_with_language(language: &str) -> Result<Self, Box<dyn Error>>
// Get all lessons
fn get_lessons(&self) -> &Vec<Lesson>
// Get lesson by ID
fn get_lesson(&self, id: u32) -> Option<&Lesson>
// Get next lesson after current
fn get_next_lesson(&self, current_id: u32) -> Option<&Lesson>
```

### SpeedTestTextView

```rust
// Set/get original and typed text
fn set_original_text(&self, text: &str)
fn push_original_text(&self, text: &str)
fn set_typed_text(&self, text: &str)
fn original_text(&self) -> String
fn typed_text(&self) -> String
// Query state
fn progress(&self) -> f64
fn keystrokes(&self) -> Vec<(Instant, bool)>
fn last_grapheme_state(&self) -> Option<GraphemeState>
fn reset(&self)
```
