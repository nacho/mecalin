use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use std::cell::{Cell, RefCell};

use crate::course::Lesson;
use crate::keyboard_widget::KeyboardWidget;
use crate::target_text_view::TargetTextView;
use crate::text_view::TextView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/gnome/mecalin/ui/lesson_view.ui")]
    #[properties(wrapper_type = super::LessonView)]
    pub struct LessonView {
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub step_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub continue_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub text_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub repetition_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub target_text_view: TemplateChild<TargetTextView>,
        #[template_child]
        pub text_view: TemplateChild<TextView>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,

        pub keyboard_widget: RefCell<Option<KeyboardWidget>>,
        #[property(get, set, nullable)]
        pub current_lesson: RefCell<Option<glib::BoxedAnyObject>>,
        #[property(get, set)]
        pub current_step_index: Cell<u32>,
        pub current_repetition: Cell<u32>,
        pub course: RefCell<Option<crate::course::Course>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LessonView {
        const NAME: &'static str = "LessonView";
        type Type = super::LessonView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LessonView {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.setup_keyboard();
            self.setup_signals();
            self.setup_settings_signals();
        }
    }

    impl WidgetImpl for LessonView {}
    impl BoxImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_keyboard(&self) {
        let keyboard = KeyboardWidget::new();
        self.keyboard_container.append(keyboard.widget());
        self.keyboard_widget.replace(Some(keyboard));
    }

    fn setup_signals(&self) {
        // Setup continue button for introduction steps
        let lesson_view_weak = self.obj().downgrade();
        self.continue_button.connect_clicked(move |_| {
            if let Some(lesson_view) = lesson_view_weak.upgrade() {
                lesson_view.advance_to_next_step();
            }
        });

        let keyboard_widget = self.keyboard_widget.borrow();
        if let Some(keyboard) = keyboard_widget.as_ref() {
            let keyboard_clone = keyboard.clone();
            let target_text_view = self.target_text_view.clone();
            let target_text_view_clone = self.target_text_view.clone();
            let lesson_view_clone = self.obj().downgrade();
            let lesson_view_clone2 = self.obj().downgrade();

            let buffer = self.text_view.text_view().buffer();
            buffer.connect_insert_text(move |buffer, _iter, text| {
                let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let target_buffer = target_text_view.text_view().buffer();
                let target_text = target_buffer.text(
                    &target_buffer.start_iter(),
                    &target_buffer.end_iter(),
                    false,
                );

                let current_str = current_text.as_str();
                let target_str = target_text.as_str();
                let new_text = format!("{}{}", current_str, text);

                // Check if the new text would match target text
                if !target_str.starts_with(&new_text) {
                    // Mistake made - reset repetition count
                    if let Some(lesson_view) = lesson_view_clone.upgrade() {
                        lesson_view.reset_repetition_count();
                    }

                    // Find the last space position or go to beginning
                    let last_space_pos = current_str.rfind(' ').map(|pos| pos + 1).unwrap_or(0);

                    // Reset to last space position
                    let corrected_text = &current_str[..last_space_pos];

                    glib::idle_add_local_once({
                        let buffer = buffer.clone();
                        let corrected_text = corrected_text.to_string();
                        move || {
                            buffer.set_text(&corrected_text);
                            let end_iter = buffer.end_iter();
                            buffer.place_cursor(&end_iter);
                        }
                    });
                }
            });

            buffer.connect_changed(move |buffer| {
                let typed_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let target_buffer = target_text_view_clone.text_view().buffer();
                let target_text = target_buffer.text(
                    &target_buffer.start_iter(),
                    &target_buffer.end_iter(),
                    false,
                );

                let typed_str = typed_text.as_str();
                let target_str = target_text.as_str();

                let cursor_pos = typed_str.chars().count() as i32;
                target_text_view_clone.set_cursor_position(cursor_pos);

                // Check if step is completed
                if typed_str == target_str && !target_str.is_empty() {
                    // Step completed - check if we need more repetitions
                    glib::idle_add_local_once({
                        let lesson_view = lesson_view_clone2.clone();
                        move || {
                            if let Some(lesson_view) = lesson_view.upgrade() {
                                lesson_view.handle_step_completion();
                            }
                        }
                    });
                    return;
                }

                // Update keyboard highlighting for next character
                let next_char = target_str.chars().nth(cursor_pos as usize);
                keyboard_clone.set_current_key(next_char);
            });
        }
    }

    fn setup_settings_signals(&self) {
        let obj = self.obj();
        obj.connect_notify_local(Some("current-step-index"), |lesson_view, _| {
            let settings = gio::Settings::new("org.gnome.mecalin");
            settings
                .set_uint("current-step", lesson_view.current_step_index() + 1)
                .unwrap();
        });
    }
}

glib::wrapper! {
    pub struct LessonView(ObjectSubclass<imp::LessonView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LessonView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_lesson(&self, lesson: &Lesson) {
        self.set_current_lesson(Some(glib::BoxedAnyObject::new(lesson.clone())));

        // Save current lesson to settings
        let settings = gio::Settings::new("org.gnome.mecalin");
        settings.set_uint("current-lesson", lesson.id).unwrap();

        let imp = self.imp();
        imp.lesson_description.set_text(&lesson.description);

        // Reset step index and repetition count
        self.set_current_step_index(0);
        imp.current_repetition.set(0);

        if lesson.introduction {
            // Introduction lesson - show description and continue button, hide everything else
            imp.step_description.set_visible(false);
            imp.continue_button.set_visible(true);
            imp.text_container.set_visible(false);
        } else {
            // Regular lesson - handle first step
            // Set the first step's text as target text
            if let Some(first_step) = lesson.steps.first() {
                if first_step.introduction {
                    imp.step_description.set_visible(true);
                    imp.step_description.set_text(
                        first_step
                            .description
                            .as_deref()
                            .unwrap_or(&first_step.text),
                    );
                    imp.continue_button.set_visible(true);
                    imp.text_container.set_visible(false);
                } else {
                    imp.step_description.set_visible(false);
                    imp.continue_button.set_visible(false);
                    imp.text_container.set_visible(true);
                    imp.target_text_view.set_text(&first_step.text);
                    self.update_repetition_label();
                }

                // Extract unique characters from the lesson text for keyboard display
                let mut target_keys = std::collections::HashSet::new();
                for ch in first_step.text.chars() {
                    if ch.is_alphabetic() || ch == ' ' {
                        target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                    }
                }

                let keyboard_widget = imp.keyboard_widget.borrow();
                if let Some(keyboard) = keyboard_widget.as_ref() {
                    keyboard.set_visible_keys(Some(target_keys));
                }
            }
        }

        imp.text_view.set_text("");
    }

    pub fn load_step(&self, step_index: u32) {
        self.set_current_step_index(step_index);

        let imp = self.imp();
        // Reset repetition count for new step
        imp.current_repetition.set(0);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                if let Some(step) = lesson.steps.get(step_index as usize) {
                    if step.introduction {
                        // Introduction step - show description and continue button, hide text views
                        imp.step_description.set_visible(true);
                        imp.step_description
                            .set_text(step.description.as_deref().unwrap_or(&step.text));
                        imp.continue_button.set_visible(true);
                        imp.text_container.set_visible(false);
                    } else {
                        // Regular step - hide description and button, show text views
                        imp.step_description.set_visible(false);
                        imp.continue_button.set_visible(false);
                        imp.text_container.set_visible(true);
                        imp.target_text_view.set_text(&step.text);
                        imp.text_view.set_text("");
                        self.update_repetition_label();
                    }

                    // Update keyboard for this step
                    let mut target_keys = std::collections::HashSet::new();
                    for ch in step.text.chars() {
                        if ch.is_alphabetic() || ch == ' ' {
                            target_keys.insert(ch.to_lowercase().next().unwrap_or(ch));
                        }
                    }

                    let keyboard_widget = imp.keyboard_widget.borrow();
                    if let Some(keyboard) = keyboard_widget.as_ref() {
                        keyboard.set_visible_keys(Some(target_keys));
                    }
                }
            }
        }
    }

    pub fn set_course(&self, course: crate::course::Course) {
        let imp = self.imp();
        *imp.course.borrow_mut() = Some(course);
    }

    pub fn reset_repetition_count(&self) {
        let imp = self.imp();
        imp.current_repetition.set(0);
        self.update_repetition_label();
    }

    pub fn update_repetition_label(&self) {
        let imp = self.imp();
        let current_repetition = imp.current_repetition.get();

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                let step_index = self.current_step_index() as usize;
                if let Some(step) = lesson.steps.get(step_index) {
                    let label_text =
                        i18n_fmt! { i18n_fmt("{}/{} Good", current_repetition, step.repetitions) };
                    imp.repetition_label.set_text(&label_text);
                }
            }
        }
    }

    pub fn handle_step_completion(&self) {
        let imp = self.imp();
        let current_repetition = imp.current_repetition.get() + 1;
        imp.current_repetition.set(current_repetition);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref() {
            if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                let step_index = self.current_step_index() as usize;
                if let Some(step) = lesson.steps.get(step_index) {
                    self.update_repetition_label();

                    if current_repetition >= step.repetitions {
                        // Required repetitions completed, advance to next step
                        self.advance_to_next_step();
                    } else {
                        // Need more repetitions, clear text for next attempt
                        imp.text_view.set_text("");
                    }
                }
            }
        }
    }

    pub fn advance_to_next_step(&self) {
        let imp = self.imp();

        // Check if this is an introduction lesson
        let is_introduction_lesson = {
            let current_lesson_boxed = imp.current_lesson.borrow();
            if let Some(boxed) = current_lesson_boxed.as_ref() {
                if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                    lesson.introduction
                } else {
                    false
                }
            } else {
                false
            }
        };

        if is_introduction_lesson {
            // Introduction lesson completed - try to load next lesson
            let current_lesson_id = {
                let current_lesson_boxed = imp.current_lesson.borrow();
                if let Some(boxed) = current_lesson_boxed.as_ref() {
                    if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                        lesson.id
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            let next_lesson_option = {
                let course = imp.course.borrow();
                course
                    .as_ref()
                    .and_then(|c| c.get_lesson(current_lesson_id + 1).cloned())
            };

            if let Some(next_lesson) = next_lesson_option {
                // Load next lesson
                self.set_lesson(&next_lesson);
            } else {
                // All lessons completed
                imp.lesson_description
                    .set_text(&gettext("Course completed! Congratulations!"));
                imp.step_description.set_visible(false);
                imp.continue_button.set_visible(false);
                imp.text_container.set_visible(false);
            }
            return;
        }

        // Get the current lesson info without borrowing
        let (current_lesson_id, current_step, total_steps) = {
            let current_lesson_boxed = imp.current_lesson.borrow();
            if let Some(boxed) = current_lesson_boxed.as_ref() {
                if let Ok(lesson) = boxed.try_borrow::<Lesson>() {
                    (
                        lesson.id,
                        self.current_step_index() as usize,
                        lesson.steps.len(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let next_step = current_step + 1;

        if next_step < total_steps {
            // Move to next step within current lesson
            self.load_step(next_step as u32);
        } else {
            // Current lesson completed - try to load next lesson
            let next_lesson_option = {
                let course = imp.course.borrow();
                course
                    .as_ref()
                    .and_then(|c| c.get_lesson(current_lesson_id + 1).cloned())
            };

            if let Some(next_lesson) = next_lesson_option {
                // Load next lesson
                self.set_lesson(&next_lesson);
            } else {
                // Check if we have a course to determine the message
                let has_course = imp.course.borrow().is_some();
                if has_course {
                    // All lessons completed
                    imp.target_text_view
                        .set_text(&gettext("Course completed! Congratulations!"));
                } else {
                    // No course set, just show lesson completion
                    imp.target_text_view
                        .set_text(&gettext("Lesson completed! Well done!"));
                }
                imp.text_view.set_text("");
            }
        }
    }
}
