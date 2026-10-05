use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use libadwaita as adw;
use libadwaita::subclass::prelude::*;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/typing_row.ui")]
    pub struct TypingRow {
        #[template_child]
        pub target_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_input: TemplateChild<gtk::Text>,
        #[template_child]
        pub repetition_label: TemplateChild<gtk::Label>,
        pub cursor_position: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TypingRow {
        const NAME: &'static str = "MecalinTypingRow";
        type Type = super::TypingRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TypingRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    glib::subclass::Signal::builder("mistake-made")
                        .param_types([bool::static_type()])
                        .build(),
                    glib::subclass::Signal::builder("step-completed").build(),
                    glib::subclass::Signal::builder("next-char-changed")
                        .param_types([String::static_type()])
                        .build(),
                    glib::subclass::Signal::builder("dead-key-started").build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.setup_cursor_lock();
            self.setup_text_validation();
            self.setup_dead_key_detection();
        }
    }

    impl WidgetImpl for TypingRow {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            self.draw_cursor(snapshot);
        }

        fn grab_focus(&self) -> bool {
            self.text_input.grab_focus()
        }
    }
    impl ListBoxRowImpl for TypingRow {}
    impl PreferencesRowImpl for TypingRow {}
}

impl imp::TypingRow {
    fn setup_cursor_lock(&self) {
        // Prevent cursor movement - always keep cursor at the end
        self.text_input
            .connect_move_cursor(move |text_input, _, _, _| {
                glib::idle_add_local_once(glib::clone!(
                    #[weak]
                    text_input,
                    move || {
                        let buffer = text_input.buffer();
                        let text_len = buffer.text().len() as u16;
                        text_input.set_position(text_len as i32);
                    }
                ));
            });

        // Also reset cursor position on any notify::cursor-position
        self.text_input
            .connect_notify_local(Some("cursor-position"), move |text_input, _| {
                let buffer = text_input.buffer();
                let text_len = buffer.text().len() as u16;
                let current_pos = text_input.position();
                if current_pos != text_len as i32 {
                    text_input.set_position(text_len as i32);
                }
            });
    }

    fn setup_text_validation(&self) {
        let buffer = self.text_input.buffer();
        buffer.connect_notify_local(
            Some("text"),
            glib::clone!(
                #[strong(rename_to = typing_row)]
                self.obj(),
                move |buffer, _| {
                    let imp = typing_row.imp();
                    let typed_text = buffer.text();
                    let target_text = imp.target_label.text();

                    let typed_normalized =
                        glib::normalize(typed_text.as_str(), glib::NormalizeMode::DefaultCompose);
                    let target_normalized =
                        glib::normalize(target_text.as_str(), glib::NormalizeMode::DefaultCompose);
                    let typed_str = typed_normalized.as_str();
                    let target_str = target_normalized.as_str();

                    // Check if the new text would match target text
                    if !target_str.starts_with(typed_str) && !typed_str.is_empty() {
                        typing_row.show_error();

                        let mut chars: Vec<char> = typed_str.chars().collect();
                        chars.pop();
                        let text_without_last = chars.iter().collect::<String>();

                        // Find the last space position in the corrected text, or go to beginning
                        let last_space_pos =
                            text_without_last.rfind(' ').map(|pos| pos + 1).unwrap_or(0);

                        // Emit mistake signal (true if at beginning)
                        typing_row.emit_by_name::<()>("mistake-made", &[&(last_space_pos == 0)]);

                        // Reset to last space position
                        let corrected_text = text_without_last[..last_space_pos].to_string();

                        glib::idle_add_local_once(glib::clone!(
                            #[strong]
                            typing_row,
                            move || {
                                let imp = typing_row.imp();
                                let corrected_len = corrected_text.len();
                                imp.text_input.buffer().set_text(&corrected_text);
                                imp.text_input.set_position(corrected_len as i32);
                            }
                        ));
                        return;
                    }

                    let cursor_pos = typed_str.chars().count() as i32;
                    imp.cursor_position.set(cursor_pos);
                    typing_row.queue_draw();

                    // Check if step is completed
                    if typed_str == target_str && !target_str.is_empty() {
                        glib::idle_add_local_once(glib::clone!(
                            #[strong]
                            typing_row,
                            move || {
                                typing_row.emit_by_name::<()>("step-completed", &[]);
                            }
                        ));
                        return;
                    }

                    let next_char = target_str.chars().nth(cursor_pos as usize);
                    if let Some(ch) = next_char {
                        typing_row.emit_by_name::<()>("next-char-changed", &[&ch.to_string()]);
                    }
                }
            ),
        );
    }

    fn setup_dead_key_detection(&self) {
        self.text_input.connect_preedit_changed(glib::clone!(
            #[strong(rename_to = typing_row)]
            self.obj(),
            move |_, preedit| {
                let is_composing = !preedit.is_empty();
                // If composition started with a single character, it's a dead key
                if is_composing && preedit.chars().count() == 1 {
                    typing_row.emit_by_name::<()>("dead-key-started", &[]);
                }
            }
        ));
    }

    fn char_pos_to_byte_index(text: &str, char_pos: usize) -> usize {
        for (char_count, (byte_idx, _)) in text.char_indices().enumerate() {
            if char_count >= char_pos {
                return byte_idx;
            }
        }
        text.len()
    }

    fn draw_cursor(&self, snapshot: &gtk::Snapshot) {
        let cursor_pos = self.cursor_position.get();
        let label = self.target_label.get();

        let text = label.text();
        if text.is_empty() {
            return;
        }

        let layout = label.layout();

        let index = Self::char_pos_to_byte_index(&text, cursor_pos as usize);
        let (rect, _) = layout.cursor_pos(index as i32);

        // Get the label's position relative to the TypingRow widget
        let typing_row = self.obj();
        let point = label
            .compute_point(
                typing_row.upcast_ref::<gtk::Widget>(),
                &gtk::graphene::Point::new(0.0, 0.0),
            )
            .unwrap_or_else(|| gtk::graphene::Point::new(0.0, 0.0));

        let x = point.x() + (rect.x() / pango::SCALE) as f32 - 1.0;
        let y = point.y() + (rect.y() / pango::SCALE) as f32;
        let height = (rect.height() / pango::SCALE) as f32;

        #[allow(deprecated)]
        let style_ctx = label.style_context();
        #[allow(deprecated)]
        let color = style_ctx.color();

        let cursor_rect = gtk::graphene::Rect::new(x, y, 1.0, height);
        snapshot.append_color(&color, &cursor_rect);
    }
}

glib::wrapper! {
    pub struct TypingRow(ObjectSubclass<imp::TypingRow>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl TypingRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_target_text(&self, text: &str) {
        let imp = self.imp();
        imp.target_label.set_text(text);
        imp.cursor_position.set(0);
        self.queue_draw();
    }

    pub fn clear(&self) {
        self.imp().text_input.buffer().set_text("");
    }

    pub fn set_repetition_text(&self, text: &str) {
        self.imp().repetition_label.set_text(text);
    }

    fn show_error(&self) {
        self.add_css_class("typing-error");

        glib::timeout_add_local_once(std::time::Duration::from_millis(400), {
            let typing_row = self.clone();
            move || {
                typing_row.remove_css_class("typing-error");
            }
        });
    }
}

impl Default for TypingRow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_pos_to_byte_index_ascii() {
        let text = "hello";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 2), 2);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 5);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 10), 5); // Beyond end
    }

    #[test]
    fn test_char_pos_to_byte_index_multibyte() {
        let text = "añ";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0); // 'a' at byte 0
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 1), 1); // 'ñ' at byte 1
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 2), 3); // End (ñ is 2 bytes)
    }

    #[test]
    fn test_char_pos_to_byte_index_mixed() {
        let text = "hola ñ mundo";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0); // 'h'
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 5); // 'ñ'
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 6), 7); // ' ' after ñ
    }

    #[test]
    fn test_char_pos_to_byte_index_empty() {
        let text = "";
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 0), 0);
        assert_eq!(imp::TypingRow::char_pos_to_byte_index(text, 5), 0);
    }
}
