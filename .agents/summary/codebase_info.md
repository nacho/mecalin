# Codebase Information
<!-- metadata: type=overview, audience=ai-agents, scope=project-wide -->

## Project Identity

- **Name**: Teclazo
- **Application ID**: `io.github.nacho.teclazo`
- **Version**: 1.0.2
- **License**: GPL-3.0-or-later
- **Category**: Education (Typing Tutor)
- **Distribution**: [Flathub](https://flathub.org/apps/io.github.nacho.teclazo)
- **Heritage**: Based on [Mecawin](https://archive.org/details/mecawin), a classic Windows typing tutor

## Technology Stack

| Layer | Technology | Version Requirement |
|-------|-----------|-------------------|
| Language | Rust | Edition 2024 |
| UI Framework | GTK4 | ≥ 4.14 (Cargo), ≥ 4.10 (Meson) |
| Design System | libadwaita | ≥ 1.5 |
| Build (dev) | Cargo | stable toolchain |
| Build (prod) | Meson | ≥ 0.59.0 |
| Packaging | Flatpak | GNOME Platform 46 |
| i18n | gettext | via `gettext-rs` crate |
| CI | GitHub Actions | ubuntu-latest |

## Language Breakdown

| Language | Files | Purpose |
|----------|-------|---------|
| Rust | 20 `.rs` files | Application logic |
| XML | 11 `.ui` files | GTK Builder UI templates |
| JSON | 7 lesson + 7 keyboard layout files | Localized content |
| Text | 40+ `.txt` files | Word lists for text generation |
| CSS | 1 `style.css` | Custom styling |
| XML | 1 `.gresource.xml` | Resource manifest |
| XML | 1 `.gschema.xml` | GSettings schema |
| YAML | 1 `ci.yml` + 1 Flatpak manifest | CI/CD and packaging |

## Supported Languages (UI Translation)

Spanish (es), French (fr), Galician (gl), Italian (it), Polish (pl), Portuguese (pt)

## Supported Keyboard Layouts

US, Spanish, French, Galician, Italian, Polish, Portuguese

## Word Lists for Text Generation

40+ languages including: Arabic, Bengali, Bulgarian, Catalan, Czech, Danish, Dutch, English, Estonian, Finnish, French, Galician, German, Greek, Hebrew, Hindi, Hungarian, Indonesian, Italian, Kabyle, Kinyarwanda, Korean, Nepali, Norwegian (Bokmål/Nynorsk), Occitan, Persian, Polish, Portuguese, Romanian, Russian, Slovak, Swahili, Swedish, Turkish, Ukrainian, Vietnamese
