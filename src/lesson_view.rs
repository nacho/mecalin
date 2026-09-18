use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_format;
use libadwaita as adw;
use libadwaita::prelude::NavigationPageExt;
use libadwaita::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::course::Lesson;
use crate::course_completion_view::CourseCompletionView;
use crate::hand_widget::HandWidget;
use crate::keyboard_widget::KeyboardWidget;
use crate::typing_row::TypingRow;
use crate::utils::extract_keys;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/nacho/mecalin/ui/lesson_view.ui")]
    #[properties(wrapper_type = super::LessonView)]
    pub struct LessonView {
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub lesson_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub step_description: TemplateChild<gtk::Label>,
        #[template_child]
        pub continue_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub text_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub typing_row: TemplateChild<TypingRow>,
        #[template_child]
        pub keyboard_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub hand_widget: TemplateChild<HandWidget>,
        #[template_child]
        pub keyboard_widget: TemplateChild<KeyboardWidget>,

        pub settings: RefCell<Option<gio::Settings>>,
        #[property(get, set, nullable)]
        pub current_lesson: RefCell<Option<glib::BoxedAnyObject>>,
        #[property(get, set)]
        pub current_step_index: Cell<u32>,
        pub current_repetition: Cell<u32>,
        pub course: RefCell<Option<crate::course::Course>>,
        pub has_mistake: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LessonView {
        const NAME: &'static str = "LessonView";
        type Type = super::LessonView;
        type ParentType = adw::NavigationPage;

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
            self.setup_settings();
            self.setup_signals();
            self.obj().load_course_and_lesson();
            self.obj().setup_title_updates();
        }
    }

    impl WidgetImpl for LessonView {}
    impl NavigationPageImpl for LessonView {}
}

impl imp::LessonView {
    fn setup_signals(&self) {
        // Setup continue button for introduction steps
        let lesson_view_weak = self.obj().downgrade();
        self.continue_button.connect_clicked(move |_| {
            if let Some(lesson_view) = lesson_view_weak.upgrade() {
                lesson_view.advance_to_next_step();
            }
        });

        // Connect to TypingRow signals
        self.typing_row.connect_closure(
            "mistake-made",
            false,
            glib::closure_local!(
                #[strong(rename_to = lesson_view)]
                self.obj(),
                move |_: TypingRow, at_beginning: bool| {
                    let imp = lesson_view.imp();
                    if !at_beginning {
                        imp.has_mistake.set(true);
                    } else {
                        lesson_view.reset_repetition_count();
                    }
                }
            ),
        );

        self.typing_row.connect_closure(
            "step-completed",
            false,
            glib::closure_local!(
                #[strong(rename_to = lesson_view)]
                self.obj(),
                move |_: TypingRow| {
                    lesson_view.handle_step_completion();
                }
            ),
        );

        self.typing_row.connect_closure(
            "next-char-changed",
            false,
            glib::closure_local!(
                #[strong(rename_to = lesson_view)]
                self.obj(),
                move |_: TypingRow, next_char_str: String| {
                    let imp = lesson_view.imp();
                    let next_char = next_char_str.chars().next();
                    imp.keyboard_widget.set_current_key(next_char);

                    // Get finger for the actual current key (might be dead key after decomposition)
                    let current_key = *imp.keyboard_widget.imp().current_key.borrow();
                    let finger =
                        current_key.and_then(|ch| imp.keyboard_widget.get_finger_for_char(ch));
                    imp.hand_widget.set_current_finger(finger);
                }
            ),
        );

        self.typing_row.connect_closure(
            "dead-key-started",
            false,
            glib::closure_local!(
                #[strong(rename_to = lesson_view)]
                self.obj(),
                move |_: TypingRow| {
                    let imp = lesson_view.imp();
                    imp.keyboard_widget.advance_sequence();

                    // Update hand widget for the next character in the sequence
                    if let Some(current_key) = *imp.keyboard_widget.imp().current_key.borrow() {
                        let finger = imp.keyboard_widget.get_finger_for_char(current_key);
                        imp.hand_widget.set_current_finger(finger);
                    }
                }
            ),
        );

        // Focus typing row when page is shown
        self.obj().connect_shown(|page| {
            let imp = page.imp();
            imp.typing_row.grab_focus();
        });
    }

    fn setup_settings(&self) {
        let obj = self.obj();
        let settings = gio::Settings::new("io.github.nacho.mecalin");

        // Bind widget visibility to settings
        settings
            .bind("show-hand-widget", &*self.hand_widget, "visible")
            .build();
        settings
            .bind("show-keyboard-widget", &*self.keyboard_widget, "visible")
            .build();

        // Save current step index to settings
        obj.connect_notify_local(Some("current-step-index"), |lesson_view, _| {
            if let Some(settings) = lesson_view.imp().settings.borrow().as_ref() {
                settings
                    .set_uint("current-step", lesson_view.current_step_index() + 1)
                    .unwrap();
            }
        });

        // Listen to settings changes for current-lesson
        settings.connect_changed(
            Some("current-lesson"),
            glib::clone!(
                #[weak(rename_to = lesson_view)]
                obj,
                move |_settings, _| {
                    lesson_view.load_lesson_from_settings();
                }
            ),
        );

        self.settings.replace(Some(settings));
    }
}

glib::wrapper! {
    pub struct LessonView(ObjectSubclass<imp::LessonView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LessonView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_title_updates(&self) {
        self.connect_notify_local(Some("current-lesson"), |lesson_view, _| {
            lesson_view.update_title();
        });

        self.connect_notify_local(Some("current-step-index"), |lesson_view, _| {
            lesson_view.update_title();
        });
    }

    fn update_title(&self) {
        let imp = self.imp();
        if let Some(lesson_boxed) = self.current_lesson() {
            if let Ok(lesson) = lesson_boxed.try_borrow::<Lesson>() {
                imp.window_title.set_title(&lesson.title);

                let current_step = self.current_step_index() as usize;
                let total_steps = lesson.steps.len();
                let subtitle = i18n_format!(
                    "Lesson {}: Step {}/{}",
                    lesson.id,
                    current_step + 1,
                    total_steps
                );
                imp.window_title.set_subtitle(&subtitle);
            }
        } else {
            imp.window_title.set_title("Lessons");
            imp.window_title.set_subtitle("");
        }
    }

    fn load_course(&self) {
        let language = crate::utils::language_from_locale();
        let course = crate::course::Course::new_with_language(language).unwrap_or_default();
        self.set_course(course);
    }

    fn load_course_and_lesson(&self) {
        self.load_course();
        self.load_lesson_from_settings();
    }

    fn load_lesson_from_settings(&self) {
        let imp = self.imp();
        let course = imp.course.borrow();

        if let Some(course) = course.as_ref() {
            let settings = gio::Settings::new("io.github.nacho.mecalin");
            let current_lesson = settings.uint("current-lesson");
            let current_step = settings.uint("current-step");

            if let Some(lesson) = course.get_lesson(current_lesson) {
                self.apply_lesson(lesson);

                // Load the saved step
                if current_step > 0 {
                    let step_index = current_step - 1;
                    self.load_step(step_index);
                }
            }
        }
    }

    fn apply_lesson(&self, lesson: &Lesson) {
        self.set_current_lesson(Some(glib::BoxedAnyObject::new(lesson.clone())));

        let imp = self.imp();
        imp.lesson_description.set_text(&lesson.description);

        // Reset step index and repetition count
        self.set_current_step_index(0);
        imp.current_repetition.set(0);

        // Handle first step
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
                imp.typing_row.set_target_text(&first_step.text);

                // Show step description if available
                if let Some(description) = &first_step.description {
                    imp.step_description.set_visible(true);
                    imp.step_description.set_text(description);
                }

                self.update_repetition_label();

                // Focus the text view for immediate typing
                imp.typing_row.grab_focus();
            }

            self.update_keyboard_keys(&first_step.text);
        }

        imp.typing_row.clear();
        imp.has_mistake.set(false);
    }

    fn set_lesson(&self, lesson: &Lesson) {
        // Save current lesson to settings
        let settings = gio::Settings::new("io.github.nacho.mecalin");
        settings.set_uint("current-lesson", lesson.id).unwrap();

        // Apply the lesson (this will be called again by the settings change handler,
        // but that's okay - it's idempotent)
        self.apply_lesson(lesson);
    }

    fn load_step(&self, step_index: u32) {
        self.set_current_step_index(step_index);

        let imp = self.imp();
        // Reset repetition count for new step
        imp.current_repetition.set(0);
        imp.has_mistake.set(false);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref()
            && let Ok(lesson) = boxed.try_borrow::<Lesson>()
            && let Some(step) = lesson.steps.get(step_index as usize)
        {
            // Construct accessibility announcement
            let announcement = if step.introduction {
                // Introduction step - announce description
                step.description
                    .as_deref()
                    .unwrap_or(&step.text)
                    .to_string()
            } else {
                // Regular step - announce description (if available) and target text
                if let Some(description) = &step.description {
                    format!("{}. {}", description, step.text)
                } else {
                    step.text.clone()
                }
            };

            if step.introduction {
                // Introduction step - show description and continue button, hide text views
                imp.step_description.set_visible(true);
                imp.step_description
                    .set_text(step.description.as_deref().unwrap_or(&step.text));
                imp.continue_button.set_visible(true);
                imp.text_container.set_visible(false);
            } else {
                // Regular step - show description if available, show text views
                if let Some(description) = &step.description {
                    imp.step_description.set_visible(true);
                    imp.step_description.set_text(description);
                } else {
                    imp.step_description.set_visible(false);
                }
                imp.continue_button.set_visible(false);
                imp.text_container.set_visible(true);
                imp.typing_row.set_target_text(&step.text);
                imp.typing_row.clear();
                self.update_repetition_label();

                // Focus the text view for immediate typing
                imp.typing_row.grab_focus();
            }

            self.update_keyboard_keys(&step.text);

            // Set initial key/finger highlight
            let first_char = step.text.chars().next();
            imp.keyboard_widget.set_current_key(first_char);

            let finger = first_char.and_then(|ch| imp.keyboard_widget.get_finger_for_char(ch));
            imp.hand_widget.set_current_finger(finger);

            // Announce step content to screen readers
            self.announce(&announcement, gtk::AccessibleAnnouncementPriority::Medium);
        }
    }

    fn set_course(&self, course: crate::course::Course) {
        let imp = self.imp();
        *imp.course.borrow_mut() = Some(course);
    }

    fn reset_repetition_count(&self) {
        let imp = self.imp();
        imp.current_repetition.set(0);
        self.update_repetition_label();
    }

    fn update_repetition_label(&self) {
        let imp = self.imp();
        let current_repetition = imp.current_repetition.get();

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref()
            && let Ok(lesson) = boxed.try_borrow::<Lesson>()
        {
            let step_index = self.current_step_index() as usize;
            if let Some(step) = lesson.steps.get(step_index) {
                let label_text = i18n_format!("{}/{} Good", current_repetition, step.repetitions);
                imp.typing_row.set_repetition_text(&label_text);
            }
        }
    }

    fn handle_step_completion(&self) {
        let imp = self.imp();

        // Check if there was a mistake during this attempt
        if imp.has_mistake.get() {
            // Restart the step - reset repetition count and clear text
            self.reset_repetition_count();
            imp.has_mistake.set(false);
            imp.typing_row.clear();
            imp.typing_row.grab_focus();
            return;
        }

        let current_repetition = imp.current_repetition.get() + 1;
        imp.current_repetition.set(current_repetition);

        let current_lesson_boxed = imp.current_lesson.borrow();
        if let Some(boxed) = current_lesson_boxed.as_ref()
            && let Ok(lesson) = boxed.try_borrow::<Lesson>()
        {
            let step_index = self.current_step_index() as usize;
            if let Some(step) = lesson.steps.get(step_index) {
                self.update_repetition_label();

                if current_repetition >= step.repetitions {
                    // Required repetitions completed, advance to next step
                    self.advance_to_next_step();
                } else {
                    // Need more repetitions, clear text for next attempt
                    imp.typing_row.clear();

                    // Focus the text view for next repetition
                    imp.typing_row.grab_focus();
                }
            }
        }
    }

    fn advance_to_next_step(&self) {
        let imp = self.imp();

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
                // All lessons completed - show completion view
                self.show_completion_view();
            }
        }
    }

    fn show_completion_view(&self) {
        // Get the navigation view from the widget hierarchy
        let mut ancestor = self.parent();
        while let Some(widget) = ancestor {
            if let Some(nav_view) = widget.downcast_ref::<adw::NavigationView>() {
                let completion_view = CourseCompletionView::new();
                nav_view.push(&completion_view);
                return;
            }
            ancestor = widget.parent();
        }
    }

    fn update_keyboard_keys(&self, text: &str) {
        self.imp()
            .keyboard_widget
            .set_visible_keys(Some(extract_keys(text)));
    }
}

#[cfg(test)]
mod tests {}
