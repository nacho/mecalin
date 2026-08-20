use gtk::glib;
use gtk::glib::Unichar;

pub fn language_from_locale() -> &'static str {
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    let locale_lower = locale.to_lowercase();
    if locale_lower.starts_with("es") {
        "es"
    } else if locale_lower.starts_with("de") && !locale_lower.starts_with("de_ch") {
        // German (Germany/Austria). Swiss German (de_CH) uses a different layout
        // and orthography, so it intentionally falls through to the default.
        "de"
    } else if locale_lower.starts_with("fr") {
        "fr"
    } else if locale_lower.starts_with("gl") {
        "gl"
    } else if locale_lower.starts_with("it") {
        "it"
    } else if locale_lower.starts_with("pl") {
        "pl"
    } else if locale_lower.starts_with("pt_br") {
        "pt_br"
    } else if locale_lower.starts_with("pt") {
        "pt"
    } else {
        "us"
    }
}

/// Decompose a character and map combining accent to spacing accent
/// Returns (spacing_accent, base_char) for composed characters, None otherwise
pub fn decompose_with_spacing_accent(ch: char) -> Option<(char, char)> {
    if let glib::CharacterDecomposition::Pair(base, combining_accent) = ch.decompose() {
        let spacing_accent = match combining_accent {
            '\u{0301}' => '´',
            '\u{0300}' => '`',
            '\u{0302}' => '^',
            '\u{0303}' => '~',
            '\u{0308}' => '¨',
            _ => combining_accent,
        };
        Some((spacing_accent, base))
    } else {
        None
    }
}

/// Extract unique non-control characters from text
pub fn extract_keys(text: &str) -> std::collections::HashSet<char> {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_decompose_with_spacing_accent_acute() {
        assert_eq!(decompose_with_spacing_accent('á'), Some(('´', 'a')));
        assert_eq!(decompose_with_spacing_accent('é'), Some(('´', 'e')));
        assert_eq!(decompose_with_spacing_accent('ó'), Some(('´', 'o')));
    }

    #[test]
    fn test_decompose_with_spacing_accent_grave() {
        assert_eq!(decompose_with_spacing_accent('à'), Some(('`', 'a')));
        assert_eq!(decompose_with_spacing_accent('è'), Some(('`', 'e')));
    }

    #[test]
    fn test_decompose_with_spacing_accent_circumflex() {
        assert_eq!(decompose_with_spacing_accent('â'), Some(('^', 'a')));
        assert_eq!(decompose_with_spacing_accent('ê'), Some(('^', 'e')));
    }

    #[test]
    fn test_decompose_with_spacing_accent_tilde() {
        assert_eq!(decompose_with_spacing_accent('ã'), Some(('~', 'a')));
        assert_eq!(decompose_with_spacing_accent('ñ'), Some(('~', 'n')));
    }

    #[test]
    fn test_decompose_with_spacing_accent_diaeresis() {
        assert_eq!(decompose_with_spacing_accent('ä'), Some(('¨', 'a')));
        assert_eq!(decompose_with_spacing_accent('ü'), Some(('¨', 'u')));
    }

    #[test]
    fn test_decompose_with_spacing_accent_non_composed() {
        assert_eq!(decompose_with_spacing_accent('a'), None);
        assert_eq!(decompose_with_spacing_accent('z'), None);
    }

    #[test]
    fn test_extract_keys_basic() {
        let keys = extract_keys("hello");
        assert_eq!(keys.len(), 4);
        assert!(keys.contains(&'h'));
        assert!(keys.contains(&'e'));
        assert!(keys.contains(&'l'));
        assert!(keys.contains(&'o'));
    }

    #[test]
    fn test_extract_keys_accented() {
        let keys = extract_keys("café");
        assert_eq!(keys.len(), 4);
        assert!(keys.contains(&'c'));
        assert!(keys.contains(&'a'));
        assert!(keys.contains(&'f'));
        assert!(keys.contains(&'é'));
    }

    #[test]
    fn test_extract_keys_mixed_accents() {
        let keys = extract_keys("niño español");
        assert!(keys.contains(&'ñ'));
        assert!(keys.contains(&'a'));
        assert!(keys.contains(&' '));
    }

    #[test]
    fn test_extract_keys_control_chars() {
        let keys = extract_keys("hello\nworld\t");
        assert!(!keys.contains(&'\n'));
        assert!(!keys.contains(&'\t'));
        assert!(keys.contains(&'h'));
        assert!(keys.contains(&'w'));
    }

    #[test]
    fn test_language_from_locale_spanish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "es_ES.UTF-8") };
        assert_eq!(language_from_locale(), "es");
    }

    #[test]
    fn test_language_from_locale_italian() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "it_IT.UTF-8") };
        assert_eq!(language_from_locale(), "it");
    }

    #[test]
    fn test_language_from_locale_french() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "fr_FR.UTF-8") };
        assert_eq!(language_from_locale(), "fr");
    }

    #[test]
    fn test_language_from_locale_german() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_DE.UTF-8") };
        assert_eq!(language_from_locale(), "de");
    }

    #[test]
    fn test_language_from_locale_austrian_german() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_AT.UTF-8") };
        assert_eq!(language_from_locale(), "de");
    }

    #[test]
    fn test_language_from_locale_swiss_german_fallback() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "de_CH.UTF-8") };
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_english() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "en_US.UTF-8") };
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_default() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "xx_YY.UTF-8") };
        assert_eq!(language_from_locale(), "us");
    }

    #[test]
    fn test_language_from_locale_polish() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pl_PL.UTF-8") };
        assert_eq!(language_from_locale(), "pl");
    }

    #[test]
    fn test_language_from_locale_galician() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "gl_ES.UTF-8") };
        assert_eq!(language_from_locale(), "gl");
    }

    #[test]
    fn test_language_from_locale_portuguese() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_PT.UTF-8") };
        assert_eq!(language_from_locale(), "pt");
    }

    #[test]
    fn test_language_from_locale_brazilian_portuguese() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_BR.UTF-8") };
        assert_eq!(language_from_locale(), "pt_br");
    }

    #[test]
    fn test_language_from_locale_brazilian_portuguese_lowercase() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "pt_br.UTF-8") };
        assert_eq!(language_from_locale(), "pt_br");
    }

    #[test]
    fn test_language_from_locale_partial_match() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("LANG", "es") };
        assert_eq!(language_from_locale(), "es");
    }
}
