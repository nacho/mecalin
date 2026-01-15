use gtk::prelude::*;
use gtk::DrawingArea;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub base: String,
    pub shift: Option<String>,
    pub altgr: Option<String>,
    pub finger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub keys: Vec<Vec<KeyInfo>>,
    pub space: KeyInfo,
}

impl KeyboardLayout {
    pub fn load_from_json(layout_code: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_data = match layout_code {
            "us" => include_str!("../data/keyboard_layouts/us.json"),
            "es" => include_str!("../data/keyboard_layouts/es.json"),
            "it" => include_str!("../data/keyboard_layouts/it.json"),
            _ => return Err(format!("Unsupported layout: {}", layout_code).into()),
        };
        Ok(serde_json::from_str(json_data)?)
    }
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::load_from_json("us").unwrap_or_else(|_| Self {
            name: "US".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "q".to_string(),
                    shift: Some("Q".to_string()),
                    altgr: None,
                    finger: "left_pinky".to_string(),
                },
                KeyInfo {
                    base: "w".to_string(),
                    shift: Some("W".to_string()),
                    altgr: None,
                    finger: "left_ring".to_string(),
                },
                KeyInfo {
                    base: "e".to_string(),
                    shift: Some("E".to_string()),
                    altgr: None,
                    finger: "left_middle".to_string(),
                },
            ]],
            space: KeyInfo {
                base: " ".to_string(),
                shift: None,
                altgr: None,
                finger: "both_thumbs".to_string(),
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardWidget {
    drawing_area: DrawingArea,
    current_key: Rc<RefCell<Option<char>>>,
    visible_keys: Rc<RefCell<Option<std::collections::HashSet<char>>>>,
}

impl KeyboardWidget {
    pub fn new() -> Self {
        let layout_code = crate::utils::language_from_locale();
        let layout = Rc::new(RefCell::new(
            KeyboardLayout::load_from_json(layout_code).unwrap_or_default(),
        ));
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(800, 300);

        let current_key = Rc::new(RefCell::new(None));
        let visible_keys = Rc::new(RefCell::new(None));
        let current_key_clone = current_key.clone();
        let visible_keys_clone = visible_keys.clone();
        let layout_clone = layout.clone();

        drawing_area.set_draw_func(move |_, cr, width, height| {
            Self::draw_keyboard(
                cr,
                width,
                height,
                &current_key_clone,
                &layout_clone,
                &visible_keys_clone,
            );
        });

        Self {
            drawing_area,
            current_key,
            visible_keys,
        }
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn set_current_key(&self, key: Option<char>) {
        *self.current_key.borrow_mut() = key;
        self.drawing_area.queue_draw();
    }

    pub fn set_visible_keys(&self, keys: Option<HashSet<char>>) {
        *self.visible_keys.borrow_mut() = keys;
        self.drawing_area.queue_draw();
    }

    fn draw_keyboard(
        cr: &gtk::cairo::Context,
        width: i32,
        _height: i32,
        current_key: &Rc<RefCell<Option<char>>>,
        layout: &Rc<RefCell<KeyboardLayout>>,
        visible_keys: &Rc<RefCell<Option<HashSet<char>>>>,
    ) {
        let layout_borrowed = layout.borrow();
        let visible_keys_borrowed = visible_keys.borrow();

        let key_width = 50.0;
        let key_height = 50.0;
        let key_spacing = 5.0;
        let row_spacing = 5.0;

        let max_keys_in_row = layout_borrowed
            .keys
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(12);
        let total_width = max_keys_in_row as f64 * (key_width + key_spacing) - key_spacing;
        let start_x = (width as f64 - total_width) / 2.0;
        let start_y = 20.0;

        let current = current_key.borrow();

        for (row_idx, row) in layout_borrowed.keys.iter().enumerate() {
            let row_offset = match row_idx {
                1 => key_width * 0.5,
                2 => key_width * 0.75,
                3 => key_width * 1.25,
                _ => 0.0,
            };

            for (key_idx, key_info) in row.iter().enumerate() {
                let key_char = key_info.base.chars().next().unwrap_or(' ');
                let x = start_x + row_offset + key_idx as f64 * (key_width + key_spacing);
                let y = start_y + row_idx as f64 * (key_height + row_spacing);

                let is_current = current.is_some_and(|c| {
                    if c == ' ' {
                        // Space character should only match space, not other keys
                        false
                    } else {
                        let c_lower = c.to_lowercase().next().unwrap();
                        let base_lower = key_char.to_lowercase().next().unwrap();
                        c_lower == base_lower
                            || key_info
                                .shift
                                .as_ref()
                                .is_some_and(|s| s.chars().next().unwrap_or(' ') == c)
                            || key_info
                                .altgr
                                .as_ref()
                                .is_some_and(|a| a.chars().next().unwrap_or(' ') == c)
                    }
                });

                if is_current {
                    cr.set_source_rgb(0.29, 0.565, 0.886);
                } else {
                    cr.set_source_rgb(0.9, 0.9, 0.9);
                }

                cr.rectangle(x, y, key_width, key_height);
                cr.fill().unwrap();

                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.set_line_width(1.0);
                cr.rectangle(x, y, key_width, key_height);
                cr.stroke().unwrap();

                let should_show_text = visible_keys_borrowed.as_ref().is_none_or(|visible| {
                    visible.contains(&key_char.to_lowercase().next().unwrap())
                });

                if should_show_text {
                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    cr.select_font_face(
                        "Sans",
                        gtk::cairo::FontSlant::Normal,
                        gtk::cairo::FontWeight::Normal,
                    );

                    // Draw base character (bottom left)
                    let base_text = if key_info.base.chars().next().unwrap().is_alphabetic() {
                        key_info.base.to_uppercase()
                    } else {
                        key_info.base.clone()
                    };

                    // Use larger font for alphabetic keys (show only uppercase, centered)
                    let is_alphabetic = key_info.base.chars().next().unwrap().is_alphabetic();

                    if is_alphabetic {
                        cr.set_font_size(18.0);
                        let text_extents = cr.text_extents(&base_text).unwrap();
                        let text_x = x + (key_width - text_extents.width()) / 2.0;
                        let text_y = y + (key_height + text_extents.height()) / 2.0;
                        cr.move_to(text_x, text_y);
                        cr.show_text(&base_text).unwrap();
                    } else {
                        cr.set_font_size(20.0);
                        cr.move_to(x + 5.0, y + key_height - 5.0);
                        cr.show_text(&base_text).unwrap();

                        // Draw shift character (top left)
                        if let Some(shift_text) = &key_info.shift {
                            cr.move_to(x + 5.0, y + 15.0);
                            cr.show_text(shift_text).unwrap();
                        }

                        // Draw altgr character (bottom right)
                        if let Some(altgr_text) = &key_info.altgr {
                            if !altgr_text.is_empty() {
                                let text_extents = cr.text_extents(altgr_text).unwrap();
                                cr.move_to(
                                    x + key_width - text_extents.width() - 5.0,
                                    y + key_height - 5.0,
                                );
                                cr.show_text(altgr_text).unwrap();
                            }
                        }
                    }
                }
            }
        }

        // Space bar
        let space_x = start_x + key_width * 2.0;
        let space_y = start_y + 4.0 * (key_height + row_spacing);
        let space_width = key_width * 6.0;

        let is_space_current = current.is_some_and(|c| c == ' ');

        if is_space_current {
            cr.set_source_rgb(0.29, 0.565, 0.886);
        } else {
            cr.set_source_rgb(0.9, 0.9, 0.9);
        }

        cr.rectangle(space_x, space_y, space_width, key_height);
        cr.fill().unwrap();

        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.set_line_width(1.0);
        cr.rectangle(space_x, space_y, space_width, key_height);
        cr.stroke().unwrap();

        let should_show_space_text = visible_keys_borrowed
            .as_ref()
            .is_none_or(|visible| visible.contains(&' '));

        if should_show_space_text {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.move_to(
                space_x + space_width / 2.0 - 20.0,
                space_y + key_height / 2.0 + 5.0,
            );
            cr.show_text("SPACE").unwrap();
        }
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
