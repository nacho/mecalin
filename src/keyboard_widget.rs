use crate::utils::decompose_with_spacing_accent;
use gtk::gdk;
use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{graphene, gsk};
use libadwaita as adw;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    LeftThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    RightThumb,
    BothThumbs,
}

impl Finger {
    pub fn hand(&self) -> Option<&str> {
        match self {
            Finger::LeftPinky
            | Finger::LeftRing
            | Finger::LeftMiddle
            | Finger::LeftIndex
            | Finger::LeftThumb => Some("left"),
            Finger::RightIndex
            | Finger::RightMiddle
            | Finger::RightRing
            | Finger::RightPinky
            | Finger::RightThumb => Some("right"),
            Finger::BothThumbs => None,
        }
    }

    pub fn opposite_thumb(&self) -> Finger {
        match self.hand() {
            Some("left") => Finger::RightThumb,
            Some("right") => Finger::LeftThumb,
            _ => Finger::RightThumb,
        }
    }
}

impl FromStr for Finger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left_pinky" => Ok(Finger::LeftPinky),
            "left_ring" => Ok(Finger::LeftRing),
            "left_middle" => Ok(Finger::LeftMiddle),
            "left_index" => Ok(Finger::LeftIndex),
            "left_thumb" => Ok(Finger::LeftThumb),
            "right_index" => Ok(Finger::RightIndex),
            "right_middle" => Ok(Finger::RightMiddle),
            "right_ring" => Ok(Finger::RightRing),
            "right_pinky" => Ok(Finger::RightPinky),
            "right_thumb" => Ok(Finger::RightThumb),
            "both_thumbs" => Ok(Finger::BothThumbs),
            _ => Err(format!("Unknown finger: {}", s)),
        }
    }
}

impl fmt::Display for Finger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Finger::LeftPinky => "left_pinky",
            Finger::LeftRing => "left_ring",
            Finger::LeftMiddle => "left_middle",
            Finger::LeftIndex => "left_index",
            Finger::LeftThumb => "left_thumb",
            Finger::RightIndex => "right_index",
            Finger::RightMiddle => "right_middle",
            Finger::RightRing => "right_ring",
            Finger::RightPinky => "right_pinky",
            Finger::RightThumb => "right_thumb",
            Finger::BothThumbs => "both_thumbs",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub base: String,
    #[serde(default)]
    pub label: Option<String>,
    pub shift: Option<String>,
    pub altgr: Option<String>,
    pub finger: Finger,
}

impl KeyInfo {
    /// Check if this key should be highlighted for the given character.
    /// Prioritizes base keys over altgr variants to avoid multiple highlights.
    pub fn matches_char(&self, ch: char, layout: &KeyboardLayout) -> bool {
        if ch == ' ' {
            return false;
        }

        let ch_lower = ch.to_lowercase().next().unwrap();
        let base_char = self.base.chars().next().unwrap_or(' ');
        let base_lower = base_char.to_lowercase().next().unwrap();

        // Check base (case-insensitive)
        if ch_lower == base_lower {
            return true;
        }

        // Check shift (exact match)
        if self
            .shift
            .as_ref()
            .is_some_and(|s| s.chars().next().unwrap_or(' ') == ch)
        {
            return true;
        }

        // Only check altgr if character doesn't exist as base in any key
        if self
            .altgr
            .as_ref()
            .is_some_and(|a| a.chars().next().unwrap_or(' ') == ch)
        {
            let exists_as_base = layout.keys.iter().flatten().any(|k| {
                let k_char = k.base.chars().next().unwrap_or(' ');
                ch_lower == k_char.to_lowercase().next().unwrap()
            });
            return !exists_as_base;
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierKey {
    pub label: String,
    pub finger: Finger,
    #[serde(default)]
    pub rows: Option<Vec<u8>>,
}

impl ModifierKey {
    pub fn spans_row(&self, row: u8) -> bool {
        self.rows.as_ref().is_some_and(|r| r.contains(&row))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub keys: Vec<Vec<KeyInfo>>,
    pub space: KeyInfo,
    #[serde(default)]
    pub modifiers: HashMap<String, ModifierKey>,
}

impl KeyboardLayout {
    pub fn load_from_json(layout_code: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_data = match layout_code {
            "us" => include_str!("../data/keyboard_layouts/us.json"),
            "es" => include_str!("../data/keyboard_layouts/es.json"),
            "fr" => include_str!("../data/keyboard_layouts/fr.json"),
            "gl" => include_str!("../data/keyboard_layouts/gl.json"),
            "it" => include_str!("../data/keyboard_layouts/it.json"),
            "pl" => include_str!("../data/keyboard_layouts/pl.json"),
            "pt" => include_str!("../data/keyboard_layouts/pt.json"),
            "pt_br" => include_str!("../data/keyboard_layouts/pt_br.json"),
            _ => return Err(format!("Unsupported layout: {}", layout_code).into()),
        };
        Ok(serde_json::from_str(json_data)?)
    }

    pub fn get_finger_for_char(&self, ch: char) -> Option<Finger> {
        let ch_lower = ch.to_lowercase().next().unwrap();

        // Check space key
        if ch == ' ' {
            return Some(self.space.finger);
        }

        // Check all keys in the layout
        for row in &self.keys {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                let base_lower = key_char.to_lowercase().next().unwrap();

                // Check base (lowercase comparison)
                if ch_lower == base_lower {
                    return Some(key_info.finger);
                }

                // Check shift (exact match)
                if let Some(shift) = &key_info.shift
                    && shift.chars().next().unwrap_or(' ') == ch
                {
                    return Some(key_info.finger);
                }

                // Check altgr (exact match)
                if let Some(altgr) = &key_info.altgr
                    && altgr.chars().next().unwrap_or(' ') == ch
                {
                    return Some(key_info.finger);
                }
            }
        }

        None
    }
}

impl KeyboardLayout {
    pub fn contains_character(&self, ch: char) -> bool {
        let ch_lower = ch.to_lowercase().next().unwrap();

        // Check space key
        if ch == ' ' {
            return true;
        }

        // Check all keys in the layout
        for row in &self.keys {
            for key_info in row {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                let base_lower = key_char.to_lowercase().next().unwrap();

                // Check base (lowercase comparison)
                if ch_lower == base_lower {
                    return true;
                }

                // Check shift (exact match)
                if let Some(shift) = &key_info.shift
                    && shift.chars().next().unwrap_or(' ') == ch
                {
                    return true;
                }

                // Check altgr (exact match)
                if let Some(altgr) = &key_info.altgr
                    && altgr.chars().next().unwrap_or(' ') == ch
                {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::load_from_json("us").unwrap_or_else(|_| Self {
            name: "US".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "q".to_string(),
                    label: None,
                    shift: Some("Q".to_string()),
                    altgr: None,
                    finger: Finger::LeftPinky,
                },
                KeyInfo {
                    base: "w".to_string(),
                    label: None,
                    shift: Some("W".to_string()),
                    altgr: None,
                    finger: Finger::LeftRing,
                },
                KeyInfo {
                    base: "e".to_string(),
                    label: None,
                    shift: Some("E".to_string()),
                    altgr: None,
                    finger: Finger::LeftMiddle,
                },
            ]],
            space: KeyInfo {
                base: " ".to_string(),
                label: Some("SPACE".to_string()),
                shift: None,
                altgr: None,
                finger: Finger::BothThumbs,
            },
            modifiers: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json_us() {
        let layout = KeyboardLayout::load_from_json("us").unwrap();
        assert_eq!(layout.name, "US QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_es() {
        let layout = KeyboardLayout::load_from_json("es").unwrap();
        assert_eq!(layout.name, "Spanish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_it() {
        let layout = KeyboardLayout::load_from_json("it").unwrap();
        assert_eq!(layout.name, "Italian QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_pl() {
        let layout = KeyboardLayout::load_from_json("pl").unwrap();
        assert_eq!(layout.name, "Polish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_invalid() {
        let result = KeyboardLayout::load_from_json("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_contains_character_spanish() {
        let layout = KeyboardLayout::load_from_json("es").unwrap();

        // Test base characters (lowercase comparison)
        assert!(layout.contains_character('ñ'));
        assert!(layout.contains_character('Ñ'));
        assert!(layout.contains_character('a'));
        assert!(layout.contains_character('A'));

        // Test shift characters (exact match)
        assert!(layout.contains_character('!'));
        assert!(layout.contains_character('?'));

        // Test altgr characters (exact match)
        assert!(layout.contains_character('€'));
        assert!(layout.contains_character('['));

        // Test space
        assert!(layout.contains_character(' '));

        // Test characters not in layout
        assert!(!layout.contains_character('á'));
        assert!(!layout.contains_character('é'));
    }

    #[test]
    fn test_contains_character_us() {
        let layout = KeyboardLayout::load_from_json("us").unwrap();

        // Test base characters
        assert!(layout.contains_character('a'));
        assert!(layout.contains_character('z'));

        // Test that ñ exists in US layout as altgr character
        assert!(layout.contains_character('ñ'));

        // Test space
        assert!(layout.contains_character(' '));

        // Test characters not in layout (using characters that truly don't exist)
        assert!(!layout.contains_character('Ñ')); // Uppercase ñ not in US layout
        assert!(!layout.contains_character('ą')); // Polish character
        assert!(!layout.contains_character('ę')); // Polish character
    }

    #[test]
    fn test_get_finger_for_char_us() {
        let layout = KeyboardLayout::load_from_json("us").unwrap();

        // Test base characters
        assert_eq!(layout.get_finger_for_char('a'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('A'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('f'), Some(Finger::LeftIndex));
        assert_eq!(layout.get_finger_for_char('j'), Some(Finger::RightIndex));
        assert_eq!(layout.get_finger_for_char('l'), Some(Finger::RightRing));

        // Test space
        assert_eq!(layout.get_finger_for_char(' '), Some(Finger::BothThumbs));

        // Test shift characters
        assert_eq!(layout.get_finger_for_char('!'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('@'), Some(Finger::LeftRing));

        // Test character not in layout
        assert_eq!(layout.get_finger_for_char('Ñ'), None);
    }

    #[test]
    fn test_get_finger_for_char_es() {
        let layout = KeyboardLayout::load_from_json("es").unwrap();

        // Test Spanish-specific characters
        assert_eq!(layout.get_finger_for_char('ñ'), Some(Finger::RightPinky));
        assert_eq!(layout.get_finger_for_char('Ñ'), Some(Finger::RightPinky));

        // Test base characters
        assert_eq!(layout.get_finger_for_char('a'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('s'), Some(Finger::LeftRing));

        // Test space
        assert_eq!(layout.get_finger_for_char(' '), Some(Finger::BothThumbs));
    }

    fn create_test_layout() -> KeyboardLayout {
        KeyboardLayout {
            name: "Test".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "'".to_string(),
                    label: None,
                    shift: Some("\"".to_string()),
                    altgr: Some("´".to_string()),
                    finger: Finger::RightPinky,
                },
                KeyInfo {
                    base: "9".to_string(),
                    label: None,
                    shift: Some("(".to_string()),
                    altgr: Some("'".to_string()),
                    finger: Finger::RightRing,
                },
                KeyInfo {
                    base: "0".to_string(),
                    label: None,
                    shift: Some(")".to_string()),
                    altgr: Some("'".to_string()),
                    finger: Finger::RightPinky,
                },
            ]],
            modifiers: std::collections::HashMap::new(),
            space: KeyInfo {
                base: " ".to_string(),
                label: None,
                shift: None,
                altgr: None,
                finger: Finger::BothThumbs,
            },
        }
    }

    #[test]
    fn test_matches_char_base_priority() {
        let layout = create_test_layout();
        let apostrophe_key = &layout.keys[0][0];
        let nine_key = &layout.keys[0][1];
        let zero_key = &layout.keys[0][2];

        // Only the apostrophe key should match, not the altgr variants
        assert!(apostrophe_key.matches_char('\'', &layout));
        assert!(!nine_key.matches_char('\'', &layout));
        assert!(!zero_key.matches_char('\'', &layout));
    }

    #[test]
    fn test_matches_char_shift() {
        let layout = create_test_layout();
        let apostrophe_key = &layout.keys[0][0];

        assert!(apostrophe_key.matches_char('"', &layout));
    }

    #[test]
    fn test_matches_char_altgr_when_no_base() {
        let layout = create_test_layout();
        let apostrophe_key = &layout.keys[0][0];

        // ´ only exists as altgr, so it should match
        assert!(apostrophe_key.matches_char('´', &layout));
    }

    #[test]
    fn test_matches_char_case_insensitive_base() {
        let mut layout = create_test_layout();
        layout.keys[0].push(KeyInfo {
            base: "a".to_string(),
            label: None,
            shift: Some("A".to_string()),
            altgr: None,
            finger: Finger::LeftPinky,
        });

        let a_key = &layout.keys[0][3];
        assert!(a_key.matches_char('a', &layout));
        assert!(a_key.matches_char('A', &layout));
    }

    #[test]
    fn test_matches_char_space_never_matches() {
        let layout = create_test_layout();
        let key = &layout.keys[0][0];

        assert!(!key.matches_char(' ', &layout));
    }

    #[test]
    fn test_modifier_key_spans_row_two_rows() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Enter","finger":"right_pinky","rows":[1,2]}"#)
                .unwrap();
        assert!(m.spans_row(1));
        assert!(m.spans_row(2));
        assert!(!m.spans_row(3));
    }

    #[test]
    fn test_modifier_key_spans_row_single_row() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Enter","finger":"right_pinky","rows":[2]}"#).unwrap();
        assert!(!m.spans_row(1));
        assert!(m.spans_row(2));
    }

    #[test]
    fn test_modifier_key_spans_row_none() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Shift","finger":"left_pinky"}"#).unwrap();
        assert!(!m.spans_row(1));
        assert!(!m.spans_row(2));
    }

    #[test]
    fn test_us_layout_enter_rows() {
        let layout = KeyboardLayout::load_from_json("us").unwrap();
        let enter = layout.modifiers.get("enter").unwrap();
        assert_eq!(enter.rows, Some(vec![2]));
        assert!(!enter.spans_row(1));
        assert!(enter.spans_row(2));
    }

    #[test]
    fn test_es_layout_enter_rows() {
        let layout = KeyboardLayout::load_from_json("es").unwrap();
        let enter = layout.modifiers.get("enter").unwrap();
        assert_eq!(enter.rows, Some(vec![1, 2]));
        assert!(enter.spans_row(1));
        assert!(enter.spans_row(2));
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct KeyboardWidget {
        pub current_key: RefCell<Option<char>>,
        pub visible_keys: RefCell<Option<HashSet<char>>>,
        pub current_key_sequence: RefCell<Vec<char>>,
        pub sequence_index: RefCell<usize>,
        pub layout: RefCell<KeyboardLayout>,
        pub last_finger: RefCell<Option<Finger>>,
        pub cached_colors: RefCell<Option<HashMap<String, gdk::RGBA>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyboardWidget {
        const NAME: &'static str = "MecalinKeyboardWidget";
        type Type = super::KeyboardWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for KeyboardWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let layout_code = crate::utils::language_from_locale();
            *self.layout.borrow_mut() =
                KeyboardLayout::load_from_json(layout_code).unwrap_or_default();

            glib::idle_add_local_once(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move || {
                    this.cache_colors();
                }
            ));

            // Re-cache colors when theme changes
            let style_manager = adw::StyleManager::default();
            style_manager.connect_dark_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    glib::idle_add_local_once(glib::clone!(move || {
                        this.cache_colors();
                        this.obj().queue_draw();
                    }));
                }
            ));
        }
    }

    impl WidgetImpl for KeyboardWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width <= 0.0 || height <= 0.0 {
                return;
            }

            Self::draw_keyboard(
                snapshot,
                &widget,
                &self.current_key,
                &self.layout,
                &self.visible_keys,
                &self.cached_colors,
            );
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let (width, height) = Self::calculate_size(&self.layout.borrow());
            match orientation {
                gtk::Orientation::Horizontal => (width, width, -1, -1),
                gtk::Orientation::Vertical => (height, height, -1, -1),
                _ => (0, 0, -1, -1),
            }
        }
    }

    impl KeyboardWidget {
        fn cache_colors(&self) {
            let widget = self.obj();
            #[allow(deprecated)]
            let style_context = widget.style_context();

            #[allow(deprecated)]
            let lookup = |color_name: &str| -> gdk::RGBA {
                style_context
                    .lookup_color(color_name)
                    .unwrap_or_else(|| gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
            };

            let mut colors = HashMap::new();
            colors.insert(
                "keyboard-modifier".to_string(),
                lookup("keyboard-modifier-color"),
            );
            colors.insert(
                "keyboard-modifier-text".to_string(),
                lookup("keyboard-modifier-text-color"),
            );
            colors.insert(
                "keyboard-key-text".to_string(),
                lookup("keyboard-key-text-color"),
            );
            colors.insert("keyboard-key".to_string(), lookup("keyboard-key-color"));
            colors.insert(
                "keyboard-key-current-text".to_string(),
                lookup("keyboard-key-current-text-color"),
            );
            colors.insert(
                "keyboard-key-current".to_string(),
                lookup("keyboard-key-current-color"),
            );
            colors.insert(
                "keyboard-border".to_string(),
                lookup("keyboard-border-color"),
            );

            // Cache finger colors
            for finger in [
                Finger::LeftPinky,
                Finger::LeftRing,
                Finger::LeftMiddle,
                Finger::LeftIndex,
                Finger::LeftThumb,
                Finger::RightIndex,
                Finger::RightMiddle,
                Finger::RightRing,
                Finger::RightPinky,
                Finger::RightThumb,
                Finger::BothThumbs,
            ] {
                let class_name = Self::get_finger_css_class(&finger);
                let color_name = format!("{}-color", class_name);
                colors.insert(class_name, lookup(&color_name));
            }

            *self.cached_colors.borrow_mut() = Some(colors);
        }
    }

    impl KeyboardWidget {
        fn calculate_size(layout: &KeyboardLayout) -> (i32, i32) {
            let key_width = 50.0;
            let key_height = 50.0;
            let key_spacing = 5.0;
            let row_spacing = 5.0;

            let row0_keys = layout.keys.first().map(|r| r.len()).unwrap_or(12);
            let row0_width = row0_keys as f64 * key_width
                + (row0_keys - 1) as f64 * key_spacing
                + key_spacing
                + key_width * 2.0;

            let enter_on_row1 = layout
                .modifiers
                .get("enter")
                .is_some_and(|e| e.spans_row(1));

            let row1_keys = layout.keys.get(1).map(|r| r.len()).unwrap_or(12);
            let row1_width = key_width * 1.5
                + key_spacing
                + row1_keys as f64 * key_width
                + (row1_keys - 1) as f64 * key_spacing
                + if enter_on_row1 {
                    key_spacing + key_width * 1.75
                } else {
                    0.0
                };

            let row2_keys = layout.keys.get(2).map(|r| r.len()).unwrap_or(12);
            let row2_width = key_width * 1.75
                + key_spacing
                + row2_keys as f64 * key_width
                + row2_keys as f64 * key_spacing
                + key_width * 1.75
                - key_spacing * 2.0;

            let row3_keys = layout.keys.get(3).map(|r| r.len()).unwrap_or(11);
            let row3_width = key_width * 1.25
                + key_spacing
                + row3_keys as f64 * key_width
                + (row3_keys - 1) as f64 * key_spacing
                + key_spacing
                + key_width * 3.0;

            let row4_width = key_width * 1.5
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.3
                + key_spacing
                + key_width * 6.0
                + key_spacing
                + key_width * 1.3
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.5;

            let keyboard_width = row0_width
                .max(row1_width)
                .max(row2_width)
                .max(row3_width)
                .max(row4_width) as i32;
            let keyboard_height = (5.0 * key_height + 4.0 * row_spacing) as i32;

            (keyboard_width, keyboard_height)
        }

        fn get_finger_css_class(finger: &Finger) -> String {
            format!("finger-{}", finger.to_string().replace('_', "-"))
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_single_key(
            snapshot: &gtk::Snapshot,
            pango_context: &pango::Context,
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            key_info: Option<&KeyInfo>,
            label: Option<&str>,
            is_current: bool,
            should_show_text: bool,
            key_color: &gdk::RGBA,
            key_current_color: &gdk::RGBA,
            key_text_color: &gdk::RGBA,
            key_current_text_color: &gdk::RGBA,
            key_border_color: &gdk::RGBA,
            finger_border_color: &gdk::RGBA,
        ) {
            let bounds = graphene::Rect::new(x, y, width, height);
            let rounded = gsk::RoundedRect::new(
                bounds,
                graphene::Size::zero(),
                graphene::Size::zero(),
                graphene::Size::zero(),
                graphene::Size::zero(),
            );

            snapshot.save();
            snapshot.append_color(
                if is_current {
                    key_current_color
                } else {
                    key_color
                },
                &bounds,
            );
            snapshot.restore();

            let border_color = if is_current {
                key_border_color
            } else {
                finger_border_color
            };
            snapshot.save();
            snapshot.append_border(
                &rounded,
                &[1.0, 1.0, 1.0, 1.0],
                &[*border_color, *border_color, *border_color, *border_color],
            );
            snapshot.restore();

            if should_show_text {
                let text_color = if is_current {
                    key_current_text_color
                } else {
                    key_text_color
                };

                if let Some(label_text) = label {
                    let layout = pango::Layout::new(pango_context);
                    let font_desc = pango::FontDescription::from_string("Sans 9");
                    layout.set_font_description(Some(&font_desc));
                    layout.set_text(label_text);
                    let (_, logical_rect) = layout.pixel_extents();
                    let text_width = logical_rect.width();
                    let text_height = logical_rect.height();
                    snapshot.save();
                    snapshot.translate(&graphene::Point::new(
                        x + (width - text_width as f32) / 2.0,
                        y + (height - text_height as f32) / 2.0 - logical_rect.y() as f32,
                    ));
                    snapshot.append_layout(&layout, text_color);
                    snapshot.restore();
                } else if let Some(key) = key_info {
                    let base_text = if key.base.chars().next().unwrap().is_alphabetic() {
                        key.base.to_uppercase()
                    } else {
                        key.base.clone()
                    };
                    let is_alphabetic = key.base.chars().next().unwrap().is_alphabetic();

                    let layout = pango::Layout::new(pango_context);

                    if is_alphabetic {
                        let font_desc = pango::FontDescription::from_string("Sans 14");
                        layout.set_font_description(Some(&font_desc));
                        layout.set_text(&base_text);
                        let (text_width, text_height) = layout.pixel_size();
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(
                            x + (width - text_width as f32) / 2.0,
                            y + (height - text_height as f32) / 2.0,
                        ));
                        snapshot.append_layout(&layout, text_color);
                        snapshot.restore();
                    } else {
                        let font_desc = pango::FontDescription::from_string("Sans 14");
                        layout.set_font_description(Some(&font_desc));

                        layout.set_text(&base_text);
                        let (_, text_height) = layout.pixel_size();
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(
                            x + 5.0,
                            y + height - text_height as f32,
                        ));
                        snapshot.append_layout(&layout, text_color);
                        snapshot.restore();

                        if let Some(shift_text) = &key.shift {
                            layout.set_text(shift_text);
                            snapshot.save();
                            snapshot.translate(&graphene::Point::new(x + 5.0, y));
                            snapshot.append_layout(&layout, text_color);
                            snapshot.restore();
                        }

                        if let Some(altgr_text) = &key.altgr
                            && !altgr_text.is_empty()
                        {
                            layout.set_text(altgr_text);
                            let (text_width, text_height) = layout.pixel_size();
                            snapshot.save();
                            snapshot.translate(&graphene::Point::new(
                                x + width - text_width as f32 - 5.0,
                                y + height - text_height as f32,
                            ));
                            snapshot.append_layout(&layout, text_color);
                            snapshot.restore();
                        }
                    }
                }
            }
        }

        fn draw_keyboard(
            snapshot: &gtk::Snapshot,
            widget: &super::KeyboardWidget,
            current_key: &RefCell<Option<char>>,
            layout: &RefCell<KeyboardLayout>,
            visible_keys: &RefCell<Option<HashSet<char>>>,
            cached_colors: &RefCell<Option<HashMap<String, gdk::RGBA>>>,
        ) {
            let layout_borrowed = layout.borrow();
            let visible_keys_borrowed = visible_keys.borrow();

            let key_width = 50.0;
            let key_height = 50.0;
            let key_spacing = 5.0;
            let row_spacing = 5.0;

            let pango_context = widget.pango_context();

            let colors = cached_colors.borrow();
            let colors = colors.as_ref().expect("Colors should be cached");

            let modifier_color = colors["keyboard-modifier"];
            let modifier_text_color = colors["keyboard-modifier-text"];
            let key_text_color = colors["keyboard-key-text"];
            let key_color = colors["keyboard-key"];
            let key_current_text_color = colors["keyboard-key-current-text"];
            let key_current_color = colors["keyboard-key-current"];
            let key_border_color = colors["keyboard-border"];

            let settings = gio::Settings::new("io.github.nacho.mecalin");
            let use_finger_colors = settings.boolean("use-finger-colors");

            let get_finger_color = |finger: &Finger| -> gdk::RGBA {
                if use_finger_colors {
                    let class_name = Self::get_finger_css_class(finger);
                    colors.get(&class_name).copied().unwrap_or(key_border_color)
                } else {
                    key_border_color
                }
            };

            let current = current_key.borrow();

            let is_key_current = |key_info: &KeyInfo| -> bool {
                current.is_some_and(|c| key_info.matches_char(c, &layout_borrowed))
            };

            let should_show_key = |key_info: &KeyInfo| -> bool {
                visible_keys_borrowed.as_ref().is_none_or(|visible| {
                    // Check if any character this key can produce is in the visible set
                    let base_char = key_info.base.chars().next().unwrap_or(' ');
                    if visible.contains(&base_char) {
                        return true;
                    }
                    if let Some(shift) = &key_info.shift
                        && let Some(shift_char) = shift.chars().next()
                        && visible.contains(&shift_char)
                    {
                        return true;
                    }
                    if let Some(altgr) = &key_info.altgr
                        && let Some(altgr_char) = altgr.chars().next()
                        && visible.contains(&altgr_char)
                    {
                        return true;
                    }

                    // For characters not directly on keyboard, check if this key is needed
                    // to compose them (e.g., 'a' + '´' = 'á')
                    for &visible_char in visible.iter() {
                        if !layout_borrowed.contains_character(visible_char)
                            && let Some((spacing_accent, base)) =
                                decompose_with_spacing_accent(visible_char)
                        {
                            if base_char == spacing_accent || base_char == base {
                                return true;
                            }
                            if let Some(shift) = &key_info.shift
                                && let Some(shift_char) = shift.chars().next()
                                && (shift_char == spacing_accent || shift_char == base)
                            {
                                return true;
                            }
                            if let Some(altgr) = &key_info.altgr
                                && let Some(altgr_char) = altgr.chars().next()
                                && (altgr_char == spacing_accent || altgr_char == base)
                            {
                                return true;
                            }
                        }
                    }

                    false
                })
            };

            let should_show_char = |ch: char| -> bool {
                visible_keys_borrowed
                    .as_ref()
                    .is_none_or(|visible| visible.contains(&ch))
            };

            // Row 0: Number row + Backspace
            let mut x = 0.0;
            let y = 0.0;
            if let Some(row) = layout_borrowed.keys.first() {
                for key_info in row {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(key_info),
                        should_show_key(key_info),
                        &key_color,
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
                if let Some(backspace) = layout_borrowed.modifiers.get("backspace") {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y,
                        key_width * 2.0,
                        key_height,
                        None,
                        Some(&backspace.label),
                        false,
                        true,
                        &modifier_color,
                        &key_current_color,
                        &modifier_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &key_border_color,
                    );
                }
            }

            // Row 1: Tab + QWERTY row + Enter (spans to row 2)
            x = 0.0;
            let y1 = y + key_height + row_spacing;
            if let Some(tab) = layout_borrowed.modifiers.get("tab") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y1,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&tab.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.5 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(1) {
                for key_info in row {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y1,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(key_info),
                        should_show_key(key_info),
                        &key_color,
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(enter) = layout_borrowed.modifiers.get("enter")
                && enter.spans_row(1)
            {
                let enter_height = key_height * 2.0 + row_spacing;
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y1,
                    key_width * 1.75,
                    enter_height,
                    None,
                    Some(&enter.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
            }

            // Row 2: Caps Lock + Home row + Enter (if single-row)
            x = 0.0;
            let y2 = y1 + key_height + row_spacing;
            if let Some(caps) = layout_borrowed.modifiers.get("caps_lock") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y2,
                    key_width * 1.75,
                    key_height,
                    None,
                    Some(&caps.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.75 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(2) {
                for key_info in row {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y2,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(key_info),
                        should_show_key(key_info),
                        &key_color,
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(enter) = layout_borrowed.modifiers.get("enter")
                && enter.spans_row(2)
                && !enter.spans_row(1)
            {
                let row0_keys = layout_borrowed.keys.first().map(|r| r.len()).unwrap_or(13);
                let keyboard_right_edge = row0_keys as f32 * key_width
                    + (row0_keys - 1) as f32 * key_spacing
                    + key_spacing
                    + key_width * 2.0;
                let enter_width = keyboard_right_edge - x;
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y2,
                    enter_width,
                    key_height,
                    None,
                    Some(&enter.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
            }

            // Row 3: Left Shift + Bottom row + Right Shift
            x = 0.0;
            let y3 = y2 + key_height + row_spacing;
            if let Some(shift_l) = layout_borrowed.modifiers.get("shift_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y3,
                    key_width * 1.25,
                    key_height,
                    None,
                    Some(&shift_l.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.25 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(3) {
                for key_info in row {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y3,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(key_info),
                        should_show_key(key_info),
                        &key_color,
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(shift_r) = layout_borrowed.modifiers.get("shift_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y3,
                    key_width * 3.0,
                    key_height,
                    None,
                    Some(&shift_r.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
            }

            // Row 4: Ctrl + Super + Alt + Space + Alt + Super + Ctrl
            x = 0.0;
            let y4 = y3 + key_height + row_spacing;
            if let Some(ctrl_l) = layout_borrowed.modifiers.get("ctrl_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&ctrl_l.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.5 + key_spacing;
            }
            if let Some(super_l) = layout_borrowed.modifiers.get("super_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&super_l.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(alt_l) = layout_borrowed.modifiers.get("alt_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.3,
                    key_height,
                    None,
                    Some(&alt_l.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.3 + key_spacing;
            }
            let is_space_current = current.is_some_and(|c| c == ' ');
            let space_label = layout_borrowed.space.label.as_deref().unwrap_or("SPACE");
            Self::draw_single_key(
                snapshot,
                &pango_context,
                x,
                y4,
                key_width * 6.0,
                key_height,
                None,
                Some(space_label),
                is_space_current,
                should_show_char(' '),
                &key_color,
                &key_current_color,
                &key_text_color,
                &key_current_text_color,
                &key_border_color,
                &get_finger_color(&layout_borrowed.space.finger),
            );
            x += key_width * 6.0 + key_spacing;
            if let Some(alt_r) = layout_borrowed.modifiers.get("alt_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.3,
                    key_height,
                    None,
                    Some(&alt_r.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.3 + key_spacing;
            }
            if let Some(super_r) = layout_borrowed.modifiers.get("super_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&super_r.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(menu) = layout_borrowed.modifiers.get("menu") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&menu.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(ctrl_r) = layout_borrowed.modifiers.get("ctrl_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&ctrl_r.label),
                    false,
                    true,
                    &modifier_color,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &key_border_color,
                );
            }
        }
    }
}

glib::wrapper! {
    pub struct KeyboardWidget(ObjectSubclass<imp::KeyboardWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl KeyboardWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_current_key(&self, key: Option<char>) {
        let imp = self.imp();
        if let Some(ch) = key {
            // Only decompose if the character doesn't exist in the layout
            if !imp.layout.borrow().contains_character(ch) {
                if let Some((spacing_accent, base)) = decompose_with_spacing_accent(ch) {
                    *imp.current_key_sequence.borrow_mut() = vec![spacing_accent, base];
                    *imp.sequence_index.borrow_mut() = 0;
                    *imp.current_key.borrow_mut() = Some(spacing_accent);
                } else {
                    *imp.current_key_sequence.borrow_mut() = Vec::new();
                    *imp.sequence_index.borrow_mut() = 0;
                    *imp.current_key.borrow_mut() = key;
                }
            } else {
                *imp.current_key_sequence.borrow_mut() = Vec::new();
                *imp.sequence_index.borrow_mut() = 0;
                *imp.current_key.borrow_mut() = key;
            }

            // Track last finger for non-space characters
            if ch != ' '
                && let Some(finger) = imp.layout.borrow().get_finger_for_char(ch)
                && finger != Finger::BothThumbs
            {
                *imp.last_finger.borrow_mut() = Some(finger);
            }
        } else {
            *imp.current_key_sequence.borrow_mut() = Vec::new();
            *imp.sequence_index.borrow_mut() = 0;
            *imp.current_key.borrow_mut() = None;
        }
        self.queue_draw();
    }

    pub fn advance_sequence(&self) {
        let imp = self.imp();
        let sequence = imp.current_key_sequence.borrow();
        let mut index = imp.sequence_index.borrow_mut();

        if !sequence.is_empty() && *index < sequence.len() - 1 {
            *index += 1;
            let ch = sequence[*index];
            *imp.current_key.borrow_mut() = Some(ch);

            // Track last finger for the current character in the sequence
            if let Some(finger) = imp.layout.borrow().get_finger_for_char(ch)
                && finger != Finger::BothThumbs
            {
                *imp.last_finger.borrow_mut() = Some(finger);
            }

            self.queue_draw();
        }
    }

    pub fn set_visible_keys(&self, keys: Option<HashSet<char>>) {
        *self.imp().visible_keys.borrow_mut() = keys;
        self.queue_draw();
    }

    pub fn get_finger_for_char(&self, ch: char) -> Option<Finger> {
        let imp = self.imp();

        // Special handling for space - use opposite thumb based on last finger
        if ch == ' ' {
            let last_finger = imp.last_finger.borrow();
            return Some(
                last_finger
                    .as_ref()
                    .map_or(Finger::RightThumb, |f| f.opposite_thumb()),
            );
        }

        imp.layout.borrow().get_finger_for_char(ch)
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
