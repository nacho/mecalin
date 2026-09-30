# Teclazo Documentation Index
<!-- metadata: type=index, audience=ai-agents, scope=knowledge-base-root -->

> **For AI Assistants**: This file is the primary entry point for understanding the Teclazo codebase. Read this file first to determine which detailed documentation files to consult for specific questions.

## Project Summary

Teclazo is a GTK4/Rust/Adwaita typing tutor application for GNOME. It provides structured lessons, timed speed tests, and two gamified practice modes (falling keys and scrolling lanes). It supports multiple languages for both UI and lesson content, with visual aids including an on-screen keyboard and hand position guide.

## Documentation Map

| File | Purpose | Consult When... |
|------|---------|-----------------|
| [codebase_info.md](codebase_info.md) | Project identity, tech stack, language support | You need version info, supported languages, or technology choices |
| [architecture.md](architecture.md) | System design, patterns, navigation, state management, build pipeline | You need to understand how components connect, the GTK subclassing pattern, resource embedding, or build processes |
| [components.md](components.md) | Detailed component descriptions with parent types, roles, signals, and key behaviors | You need to understand what a specific component does, its responsibilities, or how to modify it |
| [interfaces.md](interfaces.md) | GObject signals, properties, GSettings schema, public APIs | You need signal names, property types, settings keys, or method signatures |
| [data_models.md](data_models.md) | Structs, enums, JSON schemas, state persistence | You need to understand data structures, JSON file formats, or how state is stored |
| [workflows.md](workflows.md) | User flows, build processes, CI pipeline, release process, adding languages | You need to understand end-to-end flows, how to build/release, or how to add features |
| [dependencies.md](dependencies.md) | Crate dependencies, system libraries, Flatpak runtime | You need to understand what libraries are used and why |
| [review_notes.md](review_notes.md) | Documentation gaps, inconsistencies, improvement suggestions | You need to know what's missing or uncertain in the documentation |

## Quick Reference

### Key Entry Points

| What | Where |
|------|-------|
| Application entry | `src/main.rs` → `src/application.rs` |
| Main window | `src/window.rs` + `resources/ui/window.ui` |
| Lesson system | `src/lesson_view.rs` + `src/course.rs` |
| Speed test | `src/speed_test_view.rs` + `src/speed_test_text_view.rs` |
| Games | `src/falling_keys_game.rs`, `src/scrolling_lanes_game.rs` |
| Text validation | `src/text_utils.rs` |
| Text generation | `src/text_generation.rs` |
| Keyboard rendering | `src/keyboard_widget.rs` |
| Settings | `data/io.github.nacho.teclazo.gschema.xml` |
| Build config | `build.rs`, `Cargo.toml`, `meson.build` |

### Architecture at a Glance

- **Pattern**: GTK4 GObject subclassing with composite XML templates
- **Navigation**: `adw::NavigationView` stack-based, push by tag
- **State**: GSettings for persistence, `Cell`/`RefCell` for runtime
- **Resources**: All UI/CSS/icons compiled into binary via GResource; lessons/word lists embedded via `include_str!`/`include_dir!`
- **i18n**: gettext for UI strings, separate JSON/TXT files per language for content
- **Build**: Cargo for development, Meson for production/Flatpak

### Common Tasks

| Task | Key Files to Read |
|------|------------------|
| Add a new view/page | `src/window.rs`, any existing view (e.g., `src/about_view.rs`), `resources/ui/window.ui` |
| Modify lesson behavior | `src/lesson_view.rs`, `src/course.rs`, `src/typing_row.rs` |
| Change keyboard rendering | `src/keyboard_widget.rs`, `data/keyboard_layouts/*.json` |
| Add a new language | `src/course.rs`, `src/utils.rs`, `src/text_generation.rs`, `po/LINGUAS`, see [workflows.md](workflows.md) |
| Modify speed test | `src/speed_test_view.rs`, `src/speed_test_text_view.rs`, `src/typing_test_utils.rs` |
| Change styling | `resources/style.css` |
| Update settings | `data/io.github.nacho.teclazo.gschema.xml`, consuming component |
| Modify build | `build.rs`, `Cargo.toml`, `meson.build` |

## Cross-References

- Components → Interfaces: Each component in [components.md](components.md) lists its signals; full signal details are in [interfaces.md](interfaces.md)
- Components → Data Models: Components that use data structures reference types detailed in [data_models.md](data_models.md)
- Architecture → Workflows: Design patterns in [architecture.md](architecture.md) are demonstrated in action in [workflows.md](workflows.md)
- Dependencies → Architecture: The dependency graph in [dependencies.md](dependencies.md) maps to the architectural layers in [architecture.md](architecture.md)
