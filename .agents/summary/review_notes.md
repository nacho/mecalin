# Review Notes
<!-- metadata: type=review, audience=ai-agents, scope=documentation-quality -->

## Consistency Check

### ✅ Consistent Across Documents

- **GTK version**: `codebase_info.md` correctly notes the dual requirement (≥ 4.14 for Cargo features, ≥ 4.10 for Meson dependency). All other docs reference GTK4 consistently.
- **Component names**: All component names match between `components.md`, `interfaces.md`, `architecture.md`, and `workflows.md`.
- **Signal definitions**: Signals listed in `components.md` match the detailed tables in `interfaces.md`.
- **GSettings keys**: Keys in `interfaces.md` match the schema in `data_models.md` and the actual `.gschema.xml` file.
- **Dependency versions**: All versions in `dependencies.md` match `Cargo.toml`.
- **Navigation tags**: Tags in `architecture.md` match the `push_by_tag()` calls in `window.rs`.
- **Language lists**: Supported languages are consistent between `codebase_info.md` and the actual `po/LINGUAS` and `data/lessons/` files.

### ⚠️ Minor Notes

- **SpeedTestTextView parent type**: `components.md` lists parent as `adw::Bin` but the actual code uses a composite template on a custom widget. The parent type should be verified — the `imp` struct derives `CompositeTemplate` and `Properties` but the actual parent type in the `glib::wrapper!` macro was not fully read. This is a low-risk documentation uncertainty.
- **TestConfig settings keys**: `typing_test_utils.rs` references `session-type`, `text-language`, and `session-duration` GSettings keys that are not present in the documented GSettings schema (`io.github.nacho.teclazo.gschema.xml`). This code appears to be adapted from another project (Keypunch, based on SPDX headers) and these settings keys may not be actively used, or may be defined elsewhere. This warrants verification.

## Completeness Check

### ✅ Well-Documented Areas

- Application architecture and component hierarchy
- GTK4 subclassing pattern and navigation model
- All major components with roles, parent types, and key behaviors
- GObject signals and properties
- Data models with JSON schemas
- Build workflows (Cargo, Meson, CI)
- Dependency inventory with purposes
- User-facing workflows (lessons, speed test, games)

### ⚠️ Areas Needing More Detail

1. **CSS theming details**: `style.css` defines semantic color variables (`@define-color`) for keyboard, hand, game, and speed test components. The color system (finger-based color coding using GNOME HIG palette) is not documented in detail. An agent modifying visual appearance should read `resources/style.css` directly.

2. **Dead key handling**: The dead key composition system spans `keyboard_widget.rs` (sequence tracking), `typing_row.rs` (detection), and `utils.rs` (decomposition). The interaction between these three is complex and only partially documented in component descriptions.

3. **Text alias system**: `text_utils.rs` has a sophisticated alias system (æ→ae, guillemets→quotes, non-breaking spaces) that affects validation. This is mentioned in `components.md` but the full alias table and its implications for validation are not exhaustively documented.

4. **Game difficulty progression**: Both `FallingKeysGame` and `ScrollingLanesGame` likely have difficulty scaling (speed increases, more items) but the specific mechanics are not documented.

5. **Accessibility**: `SpeedTestTextView` has an `accessibility.rs` sub-module with `update_accessible_state()`. The accessibility implementation details are not documented.

6. **Metainfo/AppStream**: `data/io.github.nacho.teclazo.metainfo.xml` contains release history and app metadata for Flathub/GNOME Software. Its structure and role in the release process could be more detailed.

7. **LOCALEDIR handling**: The `config.rs.in` template and Meson's `DATADIR`/`LOCALEDIR` configuration affect runtime locale file discovery. The interaction between Cargo dev builds (which use defaults) and Meson production builds (which set real paths) is not explicitly documented.

## Recommendations

1. **For agents modifying visual appearance**: Read `resources/style.css` directly — it contains the complete color system.
2. **For agents working on i18n**: Read `src/text_generation.rs` for the full `Language` enum and `src/course.rs` for the language match arms.
3. **For agents adding keyboard layouts**: Read an existing layout JSON (e.g., `data/keyboard_layouts/us.json`) and `src/keyboard_widget.rs` for the `KeyInfo`/`KeyboardLayout` deserialization.
4. **Verify `typing_test_utils.rs` settings**: The `session-type`, `text-language`, and `session-duration` keys referenced in `TestConfig::from_settings()` should be verified against the actual GSettings schema to determine if they are active or vestigial.
