# Workflows
<!-- metadata: type=workflows, audience=ai-agents, scope=user-flows-build-processes -->

## Application Startup

```mermaid
sequenceDiagram
    participant main as main()
    participant app as TeclazoApplication
    participant window as TeclazoWindow

    main->>main: Set locale (gettext)
    main->>main: Register GResource bundle
    main->>app: TeclazoApplication::new()
    app->>app: startup(): Set resource base path
    app->>app: Load CSS provider
    app->>app: Set keyboard shortcuts
    app->>window: activate(): TeclazoWindow::new()
    window->>window: Load window state from GSettings
    window->>window: present()
```

## Lesson Flow

```mermaid
sequenceDiagram
    participant user as User
    participant window as TeclazoWindow
    participant lesson as LessonView
    participant typing as TypingRow
    participant keyboard as KeyboardWidget
    participant hand as HandWidget

    user->>window: Click "Lessons" row
    window->>lesson: push_by_tag("lessons")
    lesson->>lesson: load_course() from locale
    lesson->>lesson: load_lesson_from_settings()
    lesson->>typing: set_target_text(step.text)
    lesson->>keyboard: set_visible_keys(step keys)
    lesson->>keyboard: set_current_key(first char)
    lesson->>hand: set_current_finger(finger for key)

    loop Each keystroke
        user->>typing: Type character
        typing->>typing: Validate against target
        alt Correct
            typing-->>lesson: next-char-changed signal
            lesson->>keyboard: set_current_key(next char)
            lesson->>hand: set_current_finger(next finger)
        else Mistake
            typing-->>lesson: mistake-made signal
            lesson->>typing: show_error()
        end
    end

    typing-->>lesson: step-completed signal
    lesson->>lesson: Increment repetition count
    alt More repetitions needed
        lesson->>typing: clear() and restart step
    else Step complete
        lesson->>lesson: advance_to_next_step()
        alt More steps in lesson
            lesson->>typing: set_target_text(next step)
        else Lesson complete
            lesson->>lesson: Load next lesson
            alt More lessons
                lesson->>typing: set_target_text(first step)
            else Course complete
                lesson->>lesson: show_completion_view()
            end
        end
    end
```

## Speed Test Flow

```mermaid
sequenceDiagram
    participant user as User
    participant view as SpeedTestView
    participant stv as SpeedTestTextView
    participant textgen as text_generation
    participant results as SpeedTestResultsView

    user->>view: Navigate to Speed Test
    view->>textgen: Generate random text
    textgen-->>view: Text chunk
    view->>stv: set_original_text(text)

    user->>stv: Start typing (first keystroke)
    stv->>stv: Start timer

    loop Each keystroke
        user->>stv: Type character
        stv->>stv: Validate grapheme
        stv->>stv: Update colors, caret, scroll
        stv-->>view: typed-text-changed signal
        view->>view: Update progress

        alt Near end of text
            stv-->>view: push-original-text signal
            view->>textgen: Generate more text
            view->>stv: push_original_text(more)
        end
    end

    alt Timer expires or text complete
        view->>view: Calculate TestSummary (WPM, accuracy)
        view->>results: set_summary(summary)
        view->>results: Show results page
    end

    user->>results: Click "Retry"
    results-->>view: retry-clicked signal
    view->>view: reset_test()
```

## Game Flow (Falling Keys / Scrolling Lanes)

```mermaid
sequenceDiagram
    participant user as User
    participant game as FallingKeysGame / ScrollingLanesGame
    participant keyboard as KeyboardWidget
    participant loop as Game Loop (glib::timeout)

    user->>game: Navigate to game
    game->>game: setup_game()
    game->>game: start_game_loop()

    loop Every frame (~16ms)
        loop->>game: update_game()
        game->>game: Move existing items
        game->>game: Check for missed items
        alt Spawn interval reached
            game->>game: spawn_key() / spawn_text()
        end
        game->>game: queue_draw()
    end

    user->>game: Press key
    game->>game: handle_key_press()
    alt Key matches falling item
        game->>game: Remove item, increment score
        game->>keyboard: Update highlight
    else No match
        game->>game: Decrement lives
    end

    alt Lives reach 0
        game->>game: show_game_over()
        user->>game: Click restart
        game->>game: restart_game()
    end
```

## Build Workflows

### Development Build (Cargo)

```mermaid
graph LR
    A["cargo run"] --> B["build.rs"]
    B --> C["Generate config.rs<br/>from config.rs.in"]
    B --> D["Compile GResource<br/>from resources.gresource.xml"]
    C --> E["cargo build"]
    D --> E
    E --> F["Run binary"]
```

### Production Build (Meson)

```mermaid
graph LR
    A["meson setup builddir"] --> B["meson compile -C builddir"]
    B --> C["Set env vars<br/>(VERSION, APPLICATION_ID, etc.)"]
    C --> D["cargo build --release"]
    D --> E["Copy binary to builddir"]
    B --> F["Compile GSettings schemas"]
    B --> G["Process i18n (gettext)"]
    B --> H["Install icons"]
```

## CI Pipeline

```mermaid
graph LR
    A["Push to main / PR"] --> B["Install deps<br/>(GTK4, libadwaita, meson)"]
    B --> C["Setup Rust stable"]
    C --> D["cargo fmt --check"]
    D --> E["cargo clippy -- -D warnings"]
    E --> F["cargo build --verbose"]
    F --> G["cargo test --verbose"]
    G --> H["meson setup + compile"]
```

## Adding a New Language

1. Add language code to `po/LINGUAS`
2. Create/update `.po` file in `po/`
3. Create lesson file: `data/lessons/{lang_code}.json`
4. Create keyboard layout: `data/keyboard_layouts/{lang_code}.json`
5. Add `include_str!` match arm in `Course::new_with_language()`
6. Add locale match in `utils::language_from_locale()`
7. Optionally add word list: `data/word_lists/{lang_code}.txt`

## Release Process

1. Update version in `Cargo.toml` and `meson.build`
2. Add release entry in `data/io.github.nacho.teclazo.metainfo.xml`
3. Run `cargo fmt` and `cargo update -p teclazo`
4. Commit as "Release X.Y.Z"
5. Tag `vX.Y.Z`
6. Push changes and tags
