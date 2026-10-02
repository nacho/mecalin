use gtk::glib;
use gtk::glib::Unichar;

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
}
