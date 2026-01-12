use gtk::prelude::*;
use gtk::DrawingArea;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub rows: Vec<Vec<String>>,
    pub finger_map: HashMap<String, String>,
}

impl KeyboardLayout {
    pub fn load_from_json(layout_code: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_data = match layout_code {
            "us" => include_str!("../data/keyboard_layouts/us.json"),
            "es" => include_str!("../data/keyboard_layouts/es.json"),
            _ => return Err(format!("Unsupported layout: {}", layout_code).into()),
        };
        Ok(serde_json::from_str(json_data)?)
    }
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::load_from_json("us").unwrap_or_else(|_| Self {
            name: "US".to_string(),
            rows: vec![
                vec![
                    "q".to_string(),
                    "w".to_string(),
                    "e".to_string(),
                    "r".to_string(),
                    "t".to_string(),
                    "y".to_string(),
                    "u".to_string(),
                    "i".to_string(),
                    "o".to_string(),
                    "p".to_string(),
                ],
                vec![
                    "a".to_string(),
                    "s".to_string(),
                    "d".to_string(),
                    "f".to_string(),
                    "g".to_string(),
                    "h".to_string(),
                    "j".to_string(),
                    "k".to_string(),
                    "l".to_string(),
                ],
                vec![
                    "z".to_string(),
                    "x".to_string(),
                    "c".to_string(),
                    "v".to_string(),
                    "b".to_string(),
                    "n".to_string(),
                    "m".to_string(),
                ],
            ],
            finger_map: HashMap::new(),
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
        drawing_area.set_size_request(600, 250);

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

        let key_width = 40.0;
        let key_height = 40.0;
        let key_spacing = 5.0;
        let row_spacing = 5.0;

        let start_x = (width as f64 - (12.0 * (key_width + key_spacing) - key_spacing)) / 2.0;
        let start_y = 20.0;

        let current = current_key.borrow();

        for (row_idx, row) in layout_borrowed.rows.iter().enumerate() {
            let row_offset = match row_idx {
                1 => key_width * 0.5,
                2 => key_width * 0.75,
                3 => key_width * 1.25,
                _ => 0.0,
            };

            for (key_idx, key_str) in row.iter().enumerate() {
                let key_char = key_str.chars().next().unwrap_or(' ');
                let x = start_x + row_offset + key_idx as f64 * (key_width + key_spacing);
                let y = start_y + row_idx as f64 * (key_height + row_spacing);

                let is_current = current.is_some_and(|c| {
                    c.to_lowercase().next() == Some(key_char.to_lowercase().next().unwrap())
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

                // Only show text if visible_keys is None or contains this key
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
                    cr.set_font_size(14.0);

                    let text = key_str.to_uppercase();
                    let text_extents = cr.text_extents(&text).unwrap();
                    let text_x = x + (key_width - text_extents.width()) / 2.0;
                    let text_y = y + (key_height + text_extents.height()) / 2.0;

                    cr.move_to(text_x, text_y);
                    cr.show_text(&text).unwrap();
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

        // Only show SPACE text if visible_keys is None or contains space
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
