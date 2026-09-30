# Teclazo

A typing tutor application built with GTK4, Rust, and Adwaita.

Based on [Mecawin](https://archive.org/details/mecawin), a classic typing tutor for Windows.

## Installation

Available on [Flathub](https://flathub.org/apps/io.github.nacho.teclazo)

### Building from Source

```bash
# Development
cargo run

# Production build
meson setup builddir
meson compile -C builddir
./builddir/teclazo
```

## Dependencies

- GTK4 4.10+
- libadwaita 1.5+
- Rust toolchain
- Meson build system

## Architecture

Uses GTK Builder with XML UI templates and GResource embedding for a modern GNOME application structure.

## Contributing

Contributions are welcome! Please note:

- This project follows the [GNOME Code of Conduct](https://conduct.gnome.org).

## Translation

### UI Translation

The application uses standard gettext for UI translation. To add a new language:

1. Add the language code to `po/LINGUAS`
2. Create or update the `.po` file in the `po/` directory
3. Translate the strings using your preferred translation tool

### Lesson Content and Keyboard Layout

**Important for translators:** Apart from translating the UI strings, you also need to create localized content:

1. **Lesson Content**: Create a new lesson file in `data/lessons/[language_code].json` based on the existing `us.json` or `es.json` files. This includes:
   - Lesson titles and descriptions
   - Step instructions and practice text
   - Introduction messages

2. **Keyboard Layout**: Create a keyboard layout file in `data/keyboard_layouts/[language_code].json` that matches your language's keyboard layout. This defines:
   - Key positions and labels
   - Finger mapping for proper touch typing guidance

The application automatically detects the system language and loads the appropriate lesson content and keyboard layout. If your language files don't exist, it falls back to the US English versions.
