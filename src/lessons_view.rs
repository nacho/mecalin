use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::course::Course;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/nacho/mecalin/ui/lessons_view.ui")]
    #[properties(wrapper_type = super::LessonsView)]
    pub struct LessonsView {
        #[template_child]
        pub continue_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub lessons_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub layout_banner: TemplateChild<adw::Banner>,

        // Rows currently added to each group, so we can clear them before
        // rebuilding on refresh (avoids accumulating duplicates).
        pub continue_rows: RefCell<Vec<adw::ActionRow>>,
        pub lesson_rows: RefCell<Vec<adw::ActionRow>>,

        /// Lesson-content language, set by the window.
        #[property(get, set = Self::set_lesson_language_prop, builder(crate::language::Language::default()))]
        pub lesson_language: Cell<crate::language::Language>,
        /// Whether the resolved layout maps to supported content; drives the banner.
        #[property(get, set = Self::set_layout_supported_prop)]
        pub layout_supported: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LessonsView {
        const NAME: &'static str = "MecalinLessonsView";
        type Type = super::LessonsView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LessonsView {
        fn constructed(&self) {
            self.parent_constructed();
            // Default to supported so the banner stays hidden until the window
            // sets the resolved state. Seed the lesson language from the locale
            // until the window pushes the resolved value.
            self.layout_supported.set(true);
            self.lesson_language
                .set(crate::language::language_from_locale());
            self.obj().refresh();

            // Keep the current-lesson marker and Continue row up to date after
            // returning from a lesson.
            self.obj().connect_shown(|view| {
                view.refresh();
            });
        }
    }

    impl WidgetImpl for LessonsView {}
    impl NavigationPageImpl for LessonsView {}

    impl LessonsView {
        /// `lesson-language` setter: rebuild the list.
        fn set_lesson_language_prop(&self, language: crate::language::Language) {
            self.lesson_language.set(language);
            self.obj().refresh();
        }

        /// `layout-supported` setter: update the banner.
        fn set_layout_supported_prop(&self, supported: bool) {
            self.layout_supported.set(supported);
            self.update_layout_banner();
        }

        /// Reveal the "unsupported layout" banner unless the layout is supported.
        fn update_layout_banner(&self) {
            if self.layout_supported.get() {
                self.layout_banner.set_revealed(false);
            } else {
                self.layout_banner.set_title(&gettext(
                    "Your system keyboard layout isn’t supported yet — showing US English",
                ));
                self.layout_banner.set_revealed(true);
            }
        }
    }
}

glib::wrapper! {
    pub struct LessonsView(ObjectSubclass<imp::LessonsView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LessonsView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Rebuild the Continue group and the lessons list, marking the current
    /// lesson. Safe to call repeatedly (clears previous rows first).
    fn refresh(&self) {
        let imp = self.imp();
        let language = imp.lesson_language.get();
        let Ok(course) = Course::new_with_language(language) else {
            return;
        };

        let settings = gio::Settings::new("io.github.nacho.mecalin");
        let current_lesson = settings.uint("current-lesson");

        // Clear any previously added rows.
        for row in imp.continue_rows.borrow_mut().drain(..) {
            imp.continue_group.remove(&row);
        }
        for row in imp.lesson_rows.borrow_mut().drain(..) {
            imp.lessons_group.remove(&row);
        }

        // Continue group: a single row resuming the current lesson.
        if let Some(lesson) = course.get_lesson(current_lesson) {
            let row = adw::ActionRow::builder()
                .title(gettext("Continue"))
                .subtitle(glib::markup_escape_text(&lesson.title))
                .activatable(true)
                .build();
            row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));

            let view = self.downgrade();
            row.connect_activated(move |_| {
                if let Some(view) = view.upgrade() {
                    view.resume_lesson();
                }
            });

            imp.continue_group.add(&row);
            imp.continue_rows.borrow_mut().push(row);
        }

        // Full lesson list, with the current lesson marked.
        for lesson in course.get_lessons() {
            let row = adw::ActionRow::builder()
                .title(glib::markup_escape_text(&lesson.title))
                .subtitle(glib::markup_escape_text(&lesson.description))
                .activatable(true)
                .build();

            if lesson.id == current_lesson {
                let marker = gtk::Image::from_icon_name("object-select-symbolic");
                marker.set_tooltip_text(Some(&gettext("Current lesson")));
                row.add_suffix(&marker);
            }

            let lesson_id = lesson.id;
            let view = self.downgrade();
            row.connect_activated(move |_| {
                if let Some(view) = view.upgrade() {
                    view.start_lesson(lesson_id);
                }
            });

            imp.lessons_group.add(&row);
            imp.lesson_rows.borrow_mut().push(row);
        }
    }

    /// Start a lesson from the list: select it and restart from the beginning.
    fn start_lesson(&self, lesson_id: u32) {
        let settings = gio::Settings::new("io.github.nacho.mecalin");
        settings.set_uint("current-lesson", lesson_id).ok();
        settings.set_uint("current-step", 0).ok();
        self.push_lesson();
    }

    /// Resume the current lesson from the saved step (no progress reset).
    fn resume_lesson(&self) {
        self.push_lesson();
    }

    fn push_lesson(&self) {
        if let Some(nav_view) = self
            .ancestor(adw::NavigationView::static_type())
            .and_downcast::<adw::NavigationView>()
        {
            nav_view.push_by_tag("lessons");
        }
    }
}

impl Default for LessonsView {
    fn default() -> Self {
        Self::new()
    }
}
