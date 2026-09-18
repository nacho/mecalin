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
    impl WidgetImpl for MecalinWindow {}
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
