use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

use crate::lesson_view::LessonView;
use crate::lessons_view::LessonsView;
use crate::typing_row::TypingRow;
use crate::welcome_view::WelcomeView;

/// Navigation tag shown on first launch (the welcome carousel).
const WELCOME_TAG: &str = "welcome";
/// Navigation tag shown on subsequent launches (the lessons overview).
const LESSONS_OVERVIEW_TAG: &str = "lessons-overview";

/// Decide which navigation page should be visible at startup based on whether
/// the first-run welcome screen has already been dismissed.
fn initial_page_tag(welcome_seen: bool) -> &'static str {
    if welcome_seen {
        LESSONS_OVERVIEW_TAG
    } else {
        WELCOME_TAG
    }
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/window.ui")]
    pub struct MecalinWindow {
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child(id = "lessons_view_widget")]
        pub lessons_view: TemplateChild<LessonsView>,
        #[template_child(id = "lesson_view_widget")]
        pub lesson_view: TemplateChild<LessonView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinWindow {
        const NAME: &'static str = "MecalinWindow";
        type Type = super::MecalinWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            WelcomeView::ensure_type();
            LessonsView::ensure_type();
            LessonView::ensure_type();
            TypingRow::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MecalinWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_initial_page();
        }
    }
    impl WidgetImpl for MecalinWindow {
        fn realize(&self) {
            self.parent_realize();
            let obj = self.obj();

            // The display is available now: resolve the keyboard-layout and
            // lesson-content codes once and distribute them to the views.
            obj.distribute_languages();

            // Subscribe once to keyboard-layout changes and re-distribute.
            if let Some(keyboard) = WidgetExt::display(&*obj)
                .default_seat()
                .and_then(|seat| seat.keyboard())
            {
                for prop in ["active-layout-index", "layout-names"] {
                    keyboard.connect_notify_local(
                        Some(prop),
                        glib::clone!(
                            #[weak]
                            obj,
                            move |_, _| {
                                obj.distribute_languages();
                            }
                        ),
                    );
                }
            }
        }
    }
    impl WindowImpl for MecalinWindow {}
    impl ApplicationWindowImpl for MecalinWindow {}
    impl AdwApplicationWindowImpl for MecalinWindow {}
}

glib::wrapper! {
    pub struct MecalinWindow(ObjectSubclass<imp::MecalinWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MecalinWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    /// Choose the startup page. The welcome carousel is the first declared
    /// child of the `AdwNavigationView`, so it is shown by default on first
    /// launch. Once the welcome screen has been seen, replace the stack with
    /// the lessons overview instead.
    fn setup_initial_page(&self) {
        let settings = gio::Settings::new("io.github.nacho.mecalin");
        let welcome_seen = settings.boolean("welcome-seen");

        if initial_page_tag(welcome_seen) == LESSONS_OVERVIEW_TAG {
            self.imp()
                .navigation_view
                .replace_with_tags(&[LESSONS_OVERVIEW_TAG]);
        }
    }

    /// Resolve the keyboard-layout and lesson-content language codes from the
    /// active keyboard layout (with locale/US fallback) and push them to the
    /// child views via their GObject properties. Called once when the window
    /// is realized and again whenever the active keyboard layout changes.
    fn distribute_languages(&self) {
        let imp = self.imp();
        let display = WidgetExt::display(self);
        let (layout, lesson) = crate::utils::resolve_languages_for_display(&display);
        let layout_supported = crate::utils::layout_is_supported_for_display(&display);

        // Properties are strings at the GObject boundary; convert via as_code().
        imp.lessons_view.set_lesson_code(lesson.as_code());
        imp.lessons_view.set_layout_supported(layout_supported);

        imp.lesson_view.set_lesson_code(lesson.as_code());
        imp.lesson_view.set_layout_code(layout.as_code());
    }

    pub fn load_window_state(&self) {
        let settings = gio::Settings::new("io.github.nacho.mecalin.state.window");

        let (width, height) = settings.get::<(i32, i32)>("size");
        self.set_default_size(width, height);

        if settings.boolean("maximized") {
            self.maximize();
        }

        self.connect_notify_local(Some("maximized"), move |window, _| {
            let settings = gio::Settings::new("io.github.nacho.mecalin.state.window");
            settings
                .set_boolean("maximized", window.is_maximized())
                .unwrap();
        });

        self.connect_notify_local(Some("default-width"), move |window, _| {
            let settings = gio::Settings::new("io.github.nacho.mecalin.state.window");
            if !window.is_maximized() {
                let size = (window.default_width(), window.default_height());
                settings.set("size", size).unwrap();
            }
        });

        self.connect_notify_local(Some("default-height"), move |window, _| {
            let settings = gio::Settings::new("io.github.nacho.mecalin.state.window");
            if !window.is_maximized() {
                let size = (window.default_width(), window.default_height());
                settings.set("size", size).unwrap();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_page_tag_first_launch() {
        assert_eq!(initial_page_tag(false), "welcome");
    }

    #[test]
    fn test_initial_page_tag_welcome_already_seen() {
        assert_eq!(initial_page_tag(true), "lessons-overview");
    }
}
