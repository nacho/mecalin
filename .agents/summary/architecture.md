# Architecture
<!-- metadata: type=architecture, audience=ai-agents, scope=system-design -->

## Overview

Teclazo follows the standard GNOME application architecture using GTK4 with the Adwaita design system. It uses the GTK subclassing pattern where each UI component is a GObject subclass with a corresponding XML composite template.

## High-Level Architecture

```mermaid
graph TB
    subgraph Entry["Application Entry"]
        main["main.rs"]
        app["TeclazoApplication"]
    end

    subgraph Navigation["Navigation Layer"]
        window["TeclazoWindow<br/>(NavigationView hub)"]
    end

    subgraph Views["Feature Views"]
        lesson["LessonView<br/>(Structured lessons)"]
        speed["SpeedTestView<br/>(Timed typing tests)"]
        falling["FallingKeysGame<br/>(Falling keys game)"]
        scrolling["ScrollingLanesGame<br/>(Scrolling lanes game)"]
        prefs["PreferencesView"]
        about["AboutView"]
        completion["CourseCompletionView"]
    end

    subgraph Widgets["Reusable Widgets"]
        typing["TypingRow<br/>(Text input)"]
        keyboard["KeyboardWidget<br/>(Visual keyboard)"]
        hand["HandWidget<br/>(Hand position guide)"]
        stv["SpeedTestTextView<br/>(Rich text display)"]
        results["SpeedTestResultsView"]
    end

    subgraph Data["Data & Utilities"]
        course["Course / Lesson / LessonStep"]
        textgen["text_generation<br/>(Random text)"]
        textutil["text_utils<br/>(Validation, WPM)"]
        testutil["typing_test_utils<br/>(Test config/summary)"]
        utils["utils<br/>(Locale, decomposition)"]
    end

    subgraph Resources["Embedded Resources"]
        ui["11 XML UI templates"]
        css["style.css"]
        lessons["7 lesson JSON files"]
        layouts["7 keyboard layout JSONs"]
        words["40+ word list files"]
        icons["SVG icons"]
    end

    main --> app
    app --> window
    window --> lesson & speed & falling & scrolling & prefs & about
    lesson --> typing & keyboard & hand & completion
    lesson --> course
    speed --> stv & results
    falling --> keyboard
    scrolling --> keyboard
    stv --> textutil
    speed --> testutil
    textgen --> words
    course --> lessons
    keyboard --> layouts
    typing --> textutil
    utils --> course & keyboard & lesson
```

## Design Patterns

### GTK4 Subclassing Pattern

Every UI component follows this structure:

```mermaid
classDiagram
    class Component {
        +mod imp (private implementation)
        +glib::wrapper! macro
        +public API methods
    }
    class imp_Module {
        +struct ComponentName (fields)
        +ObjectSubclass impl
        +ObjectImpl::constructed()
        +WidgetImpl overrides
        +CompositeTemplate derive
    }
    class XML_Template {
        +Widget hierarchy
        +template_child bindings
        +Signal connections
    }
    Component --> imp_Module : contains
    imp_Module --> XML_Template : loads via GResource
```

Each component:
1. Defines a private `imp` module with the actual struct and trait implementations
2. Uses `#[derive(gtk::CompositeTemplate)]` to bind to an XML UI template
3. Exposes a public wrapper type via `glib::wrapper!`
4. Initializes in `ObjectImpl::constructed()` — setting up signals, loading data, binding settings

### Navigation Architecture

`TeclazoWindow` uses `adw::NavigationView` as a stack-based navigation hub. Each feature is an `adw::NavigationPage` pushed by tag:

```mermaid
graph LR
    Home["Home Menu"] -->|"push by tag"| lessons["lessons"]
    Home -->|"push by tag"| speed_test["speed_test"]
    Home -->|"push by tag"| game["game (Falling Keys)"]
    Home -->|"push by tag"| lanes_game["lanes_game"]
    Home -->|"push by tag"| preferences["preferences"]
    Home -->|"push by tag"| about["about"]
```

### State Management

- **GSettings** (`io.github.nacho.teclazo`): Persists user preferences (current lesson/step, widget visibility, finger colors, test duration)
- **Window state** (`io.github.nacho.teclazo.state.window`): Persists window size and maximized state
- **In-memory state**: Component-local `Cell`/`RefCell` fields in `imp` structs

### Resource Embedding

All UI templates, CSS, and icons are compiled into the binary via GResource (`resources.gresource.xml`). Lesson data, keyboard layouts, and word lists are embedded via `include_str!` and `include_dir!` macros at compile time.

### Build-Time Code Generation

`build.rs` performs two tasks:
1. **Config generation**: Processes `src/config.rs.in` template, replacing `@VARIABLE@` placeholders with environment variables (VERSION, APPLICATION_ID, GETTEXT_PACKAGE, DATADIR)
2. **Resource compilation**: Compiles `resources.gresource.xml` into a binary resource bundle

### Internationalization

- UI strings: gettext via `gettext-rs` and `i18n-format` crates
- Lesson content: Separate JSON files per language, selected by locale detection (`utils::language_from_locale()`)
- Keyboard layouts: Separate JSON files per language
- Word lists: Separate text files per language, embedded at compile time
