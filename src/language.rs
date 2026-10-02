use gtk::glib;
use gtk::prelude::*;
use gtk::{gdk, glib::GString};

/// Log domain for glib logging macros in this module.
const G_LOG_DOMAIN: &str = "mecalin";

/// A supported content/layout language.
///
/// Replaces stringly-typed language codes throughout the app: lesson JSON and
/// keyboard-layout JSON are selected by matching on the variant. Registered as
/// a glib enum type so it can be passed directly as a GObject property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, glib::Enum)]
#[enum_type(name = "MecalinLanguage")]
pub enum Language {
    #[default]
    Us,
    Es,
    De,
    Fr,
    Gl,
    It,
    Pl,
    Pt,
    PtBr,
}

/// The language for the system locale (`LANG`), falling back to US English.
pub fn language_from_locale() -> Language {
    supported_language_from_locale().unwrap_or(Language::Us)
}

/// Like [`language_from_locale`], but `None` when the system locale is not
/// explicitly supported (English locales count as supported → US).
pub fn supported_language_from_locale() -> Option<Language> {
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    let locale_lower = locale.to_lowercase();
    if locale_lower.starts_with("es") {
        Some(Language::Es)
    } else if locale_lower.starts_with("de") && !locale_lower.starts_with("de_ch") {
        // Swiss German (de_CH) uses a different layout/orthography; it falls
        // through to the default.
        Some(Language::De)
    } else if locale_lower.starts_with("fr") {
        Some(Language::Fr)
    } else if locale_lower.starts_with("gl") {
        Some(Language::Gl)
    } else if locale_lower.starts_with("it") {
        Some(Language::It)
    } else if locale_lower.starts_with("pl") {
        Some(Language::Pl)
    } else if locale_lower.starts_with("pt_br") {
        Some(Language::PtBr)
    } else if locale_lower.starts_with("pt") {
        Some(Language::Pt)
    } else if locale_lower.starts_with("en") {
        Some(Language::Us)
    } else {
        None
    }
}

/// Map a GDK keyboard layout name to a supported *layout* language, or `None`.
///
/// Layout names are backend-dependent human-readable strings (e.g.
/// "English (US)", "Spanish"), so matching is case-insensitive/keyword-based.
/// Returns layout languages only — "Spanish" maps to [`Language::Es`] (Galician
/// shares the Spanish layout); the lesson-vs-layout split is in
/// [`resolve_languages`].
pub fn language_from_layout_name(name: &str) -> Option<Language> {
    let n = name.to_lowercase();

    // Brazilian Portuguese must be checked before generic Portuguese.
    if n.contains("portuguese") || n.contains("português") || n.contains("portugues") {
        if n.contains("brazil")
            || n.contains("brasil")
            || n.split(|c: char| !c.is_alphanumeric())
                .any(|tok| tok == "br")
        {
            return Some(Language::PtBr);
        }
        return Some(Language::Pt);
    }

    // Spanish layout (also used for Galician).
    if n.contains("spanish")
        || n.contains("español")
        || n.contains("espanol")
        || n.contains("castil")
    {
        return Some(Language::Es);
    }

    // German, excluding Swiss German (different layout/orthography).
    if n.contains("german") || n.contains("deutsch") {
        if n.contains("swiss")
            || n.contains("switzerland")
            || n.contains("schweiz")
            || n.contains("suisse")
        {
            return None;
        }
        return Some(Language::De);
    }

    if n.contains("french") || n.contains("français") || n.contains("francais") {
        return Some(Language::Fr);
    }

    if n.contains("italian") || n.contains("italiano") {
        return Some(Language::It);
    }

    if n.contains("polish") || n.contains("polski") {
        return Some(Language::Pl);
    }

    // English/US layouts. Match "english" or a standalone "us"/"usa" token
    // (avoid substring false positives like "russian"/"belarusian").
    if n.contains("english")
        || n.split(|c: char| !c.is_alphanumeric())
            .any(|tok| tok == "us" || tok == "usa")
    {
        return Some(Language::Us);
    }

    None
}

/// Resolve the `(layout_language, lesson_language)` pair from an optional active
/// layout name and the system locale.
///
/// Layout precedence: active layout → locale → US. The lesson language equals
/// the layout language, except a Spanish layout with a Galician locale gives
/// Galician lessons (Galician has no keyboard layout, so the keyboard stays
/// Spanish).
pub fn resolve_languages(layout_name: Option<&str>, locale: Language) -> (Language, Language) {
    // Layout: active layout wins; otherwise the locale, but "gl" has no layout
    // of its own, so it maps to Spanish.
    let layout = layout_name.and_then(language_from_layout_name).unwrap_or({
        if locale == Language::Gl {
            Language::Es
        } else {
            locale
        }
    });

    // Lesson: usually the layout language, but prefer Galician lessons when the
    // layout is Spanish and the locale is Galician.
    let lesson = if layout == Language::Es && locale == Language::Gl {
        Language::Gl
    } else {
        layout
    };

    (layout, lesson)
}

/// Return the name of the currently active keyboard layout for `display`
/// (GTK ≥ 4.18), or `None` if it cannot be determined. The format is
/// backend-dependent; feed it to [`language_from_layout_name`], don't compare it.
pub fn active_layout_name(display: &gdk::Display) -> Option<GString> {
    let keyboard = display.default_seat()?.keyboard()?;
    let names = keyboard.layout_names();
    if names.is_empty() {
        return None;
    }
    let index = keyboard.active_layout_index();
    if index < 0 || index as usize >= names.len() {
        return None;
    }
    Some(names[index as usize].clone())
}

/// Resolve the `(layout_language, lesson_language)` pair for `display`,
/// combining the active keyboard layout (preferred) with the system locale.
pub fn resolve_languages_for_display(display: &gdk::Display) -> (Language, Language) {
    let layout_name = active_layout_name(display);
    // Log the raw layout name so the backend-specific format can be observed.
    if let Some(name) = &layout_name {
        glib::debug!("active keyboard layout name = {:?}", name.as_str());
    } else {
        glib::debug!("no active keyboard layout name available");
    }
    resolve_languages(layout_name.as_deref(), language_from_locale())
}

/// Whether the active layout / locale maps to supported content (else the
/// "unsupported layout" banner is shown).
pub fn layout_is_supported_for_display(display: &gdk::Display) -> bool {
    if active_layout_name(display)
        .as_deref()
        .and_then(language_from_layout_name)
        .is_some()
    {
        return true;
    }
    supported_language_from_locale().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_language_from_locale_spanish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "es_ES.UTF-8") };
        assert_eq!(language_from_locale(), Language::Es);
    }

    #[test]
    fn test_language_from_locale_italian() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "it_IT.UTF-8") };
        assert_eq!(language_from_locale(), Language::It);
    }

    #[test]
    fn test_language_from_locale_french() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "fr_FR.UTF-8") };
        assert_eq!(language_from_locale(), Language::Fr);
    }

    #[test]
    fn test_language_from_locale_german() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_DE.UTF-8") };
        assert_eq!(language_from_locale(), Language::De);
    }

    #[test]
    fn test_language_from_locale_austrian_german() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_AT.UTF-8") };
        assert_eq!(language_from_locale(), Language::De);
    }

    #[test]
    fn test_language_from_locale_swiss_german_fallback() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_CH.UTF-8") };
        assert_eq!(language_from_locale(), Language::Us);
    }

    #[test]
    fn test_language_from_locale_english() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "en_US.UTF-8") };
        assert_eq!(language_from_locale(), Language::Us);
    }

    #[test]
    fn test_language_from_locale_default() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "xx_YY.UTF-8") };
        assert_eq!(language_from_locale(), Language::Us);
    }

    #[test]
    fn test_language_from_locale_polish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pl_PL.UTF-8") };
        assert_eq!(language_from_locale(), Language::Pl);
    }

    #[test]
    fn test_language_from_locale_galician() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "gl_ES.UTF-8") };
        assert_eq!(language_from_locale(), Language::Gl);
    }

    #[test]
    fn test_language_from_locale_portuguese() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_PT.UTF-8") };
        assert_eq!(language_from_locale(), Language::Pt);
    }

    #[test]
    fn test_language_from_locale_brazilian_portuguese() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_BR.UTF-8") };
        assert_eq!(language_from_locale(), Language::PtBr);
    }

    #[test]
    fn test_language_from_locale_brazilian_portuguese_lowercase() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_br.UTF-8") };
        assert_eq!(language_from_locale(), Language::PtBr);
    }

    #[test]
    fn test_language_from_locale_partial_match() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "es") };
        assert_eq!(language_from_locale(), Language::Es);
    }

    #[test]
    fn test_supported_language_spanish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "es_ES.UTF-8") };
        assert_eq!(supported_language_from_locale(), Some(Language::Es));
    }

    #[test]
    fn test_supported_language_brazilian_portuguese() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_BR.UTF-8") };
        assert_eq!(supported_language_from_locale(), Some(Language::PtBr));
    }

    #[test]
    fn test_supported_language_english_is_supported() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "en_US.UTF-8") };
        assert_eq!(supported_language_from_locale(), Some(Language::Us));
    }

    #[test]
    fn test_supported_language_unknown_is_none() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "xx_YY.UTF-8") };
        assert_eq!(supported_language_from_locale(), None);
    }

    #[test]
    fn test_supported_language_swiss_german_is_none() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_CH.UTF-8") };
        assert_eq!(supported_language_from_locale(), None);
    }

    #[test]
    fn test_supported_language_german_is_supported() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_DE.UTF-8") };
        assert_eq!(supported_language_from_locale(), Some(Language::De));
    }

    #[test]
    fn test_language_from_layout_name_table() {
        let cases: &[(&str, Option<Language>)] = &[
            ("English (US)", Some(Language::Us)),
            ("English (UK)", Some(Language::Us)),
            ("us", Some(Language::Us)),
            ("USA", Some(Language::Us)),
            ("Spanish", Some(Language::Es)),
            ("Spanish (Latin American)", Some(Language::Es)),
            ("Español", Some(Language::Es)),
            ("Castilian", Some(Language::Es)),
            ("German", Some(Language::De)),
            ("Deutsch", Some(Language::De)),
            ("Swiss German", None),
            ("German (Switzerland)", None),
            ("French", Some(Language::Fr)),
            ("Français", Some(Language::Fr)),
            ("Italian", Some(Language::It)),
            ("Polish", Some(Language::Pl)),
            ("Portuguese", Some(Language::Pt)),
            ("Portuguese (Brazil)", Some(Language::PtBr)),
            ("Brazilian Portuguese", Some(Language::PtBr)),
            ("Português (Brasil)", Some(Language::PtBr)),
            // False-positive guards for the bare "us" substring.
            ("Russian", None),
            ("Belarusian", None),
            // Unknown layouts.
            ("Japanese", None),
            ("", None),
        ];
        for (input, expected) in cases {
            assert_eq!(
                language_from_layout_name(input),
                *expected,
                "layout name {input:?} should map to {expected:?}"
            );
        }
    }

    #[test]
    fn test_resolve_languages_layout_wins_over_locale() {
        // Spanish keyboard with an English locale → Spanish lessons + layout.
        assert_eq!(
            resolve_languages(Some("Spanish"), Language::Us),
            (Language::Es, Language::Es)
        );
        // French keyboard with a German locale → French both.
        assert_eq!(
            resolve_languages(Some("French"), Language::De),
            (Language::Fr, Language::Fr)
        );
    }

    #[test]
    fn test_resolve_languages_fallback_to_locale_then_us() {
        // Unknown layout → fall back to locale.
        assert_eq!(
            resolve_languages(Some("Japanese"), Language::It),
            (Language::It, Language::It)
        );
        // No layout at all → fall back to locale.
        assert_eq!(
            resolve_languages(None, Language::Pl),
            (Language::Pl, Language::Pl)
        );
        // No layout and unsupported locale → us.
        assert_eq!(
            resolve_languages(None, Language::Us),
            (Language::Us, Language::Us)
        );
    }

    #[test]
    fn test_resolve_languages_galician_prefers_locale_for_lessons() {
        // Spanish layout + Galician locale → Spanish keyboard, Galician lessons.
        assert_eq!(
            resolve_languages(Some("Spanish"), Language::Gl),
            (Language::Es, Language::Gl)
        );
        // Galician locale with no detectable layout → Spanish keyboard (gl has
        // no layout), Galician lessons.
        assert_eq!(
            resolve_languages(None, Language::Gl),
            (Language::Es, Language::Gl)
        );
    }

    #[test]
    fn test_resolve_languages_galician_layout_detected_non_spanish() {
        // If the detected layout is explicitly something else (e.g. French)
        // while locale is Galician, the layout wins and lessons follow it.
        assert_eq!(
            resolve_languages(Some("French"), Language::Gl),
            (Language::Fr, Language::Fr)
        );
    }
}
