use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

/// GSettings schema that stores the first-run `welcome-seen` flag.
const SETTINGS_SCHEMA: &str = "io.github.nacho.mecalin";
/// Key that records whether the welcome screen has been dismissed.
const WELCOME_SEEN_KEY: &str = "welcome-seen";
/// Navigation tag of the lessons overview page (kept beneath the lesson so
/// the back gesture returns to the list rather than the welcome screen).
const LESSONS_OVERVIEW_TAG: &str = "lessons-overview";
/// Navigation tag of the lesson view (the page that runs an individual lesson).
const LESSON_TAG: &str = "lessons";
/// The first lesson is numbered 0 after the base-0 renumbering.
const FIRST_LESSON_ID: u32 = 0;

/// Decide which of the two footer buttons should be visible for a given slide.
/// Returns `(continue_visible, get_started_visible)`. "Get Started" is shown
/// only on the last slide; "Continue" on every earlier slide.
fn footer_visibility(current_page: u32, n_pages: u32) -> (bool, bool) {
    let on_last = n_pages == 0 || current_page + 1 >= n_pages;
    (!on_last, on_last)
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/welcome_view.ui")]
    pub struct WelcomeView {
        #[template_child]
        pub carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub continue_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub get_started_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WelcomeView {
        const NAME: &'static str = "WelcomeView";
        type Type = super::WelcomeView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WelcomeView {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_buttons();
            obj.setup_page_tracking();
            obj.update_footer_buttons();
        }
    }
    impl WidgetImpl for WelcomeView {}
    impl NavigationPageImpl for WelcomeView {}
}

glib::wrapper! {
    pub struct WelcomeView(ObjectSubclass<imp::WelcomeView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WelcomeView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_buttons(&self) {
        let imp = self.imp();

        imp.continue_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.advance_slide();
            }
        ));

        imp.get_started_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.finish_welcome();
            }
        ));
    }

    /// Update the footer buttons whenever the visible slide changes.
    fn setup_page_tracking(&self) {
        self.imp().carousel.connect_position_notify(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.update_footer_buttons();
            }
        ));
    }

    /// Advance to the next slide (used by the Continue button).
    fn advance_slide(&self) {
        let carousel = &self.imp().carousel;
        let n_pages = carousel.n_pages();
        if n_pages == 0 {
            return;
        }

        let current = carousel.position().round() as u32;
        let next = (current + 1).min(n_pages - 1);
        let page = carousel.nth_page(next);
        carousel.scroll_to(&page, true);
    }

    /// Show "Continue" on all but the last slide, and "Get Started" on the last.
    fn update_footer_buttons(&self) {
        let imp = self.imp();
        let current = imp.carousel.position().round() as u32;
        let n_pages = imp.carousel.n_pages();
        let (continue_visible, get_started_visible) = footer_visibility(current, n_pages);
        imp.continue_button.set_visible(continue_visible);
        imp.get_started_button.set_visible(get_started_visible);
    }

    /// Mark the welcome screen as seen and jump straight into the first lesson.
    fn finish_welcome(&self) {
        let settings = gio::Settings::new(SETTINGS_SCHEMA);
        if let Err(error) = settings.set_boolean(WELCOME_SEEN_KEY, true) {
            glib::g_warning!("mecalin", "Failed to set {WELCOME_SEEN_KEY}: {error}");
        }

        // Start the first lesson from its beginning.
        settings.set_uint("current-lesson", FIRST_LESSON_ID).ok();
        settings.set_uint("current-step", 0).ok();

        if let Some(nav_view) = self
            .ancestor(adw::NavigationView::static_type())
            .and_downcast::<adw::NavigationView>()
        {
            // Replace the stack with the lessons overview followed by the
            // lesson view: the lesson is shown immediately, and the back
            // gesture returns to the lesson list rather than the welcome page.
            nav_view.replace_with_tags(&[LESSONS_OVERVIEW_TAG, LESSON_TAG]);
        }
    }
}

impl Default for WelcomeView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_constants() {
        assert_eq!(SETTINGS_SCHEMA, "io.github.nacho.mecalin");
        assert_eq!(WELCOME_SEEN_KEY, "welcome-seen");
    }

    #[test]
    fn test_lessons_overview_tag() {
        assert_eq!(LESSONS_OVERVIEW_TAG, "lessons-overview");
    }

    #[test]
    fn test_lesson_navigation_targets() {
        assert_eq!(LESSON_TAG, "lessons");
        assert_eq!(FIRST_LESSON_ID, 0);
    }

    #[test]
    fn test_footer_visibility_first_slide() {
        // (continue, get_started)
        assert_eq!(footer_visibility(0, 3), (true, false));
    }

    #[test]
    fn test_footer_visibility_middle_slide() {
        assert_eq!(footer_visibility(1, 3), (true, false));
    }

    #[test]
    fn test_footer_visibility_last_slide() {
        assert_eq!(footer_visibility(2, 3), (false, true));
    }

    #[test]
    fn test_footer_visibility_no_pages() {
        // Degenerate case: show Get Started so the user is never stuck.
        assert_eq!(footer_visibility(0, 0), (false, true));
    }
}
