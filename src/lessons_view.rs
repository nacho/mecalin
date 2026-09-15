use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::course::Course;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/lessons_view.ui")]
    pub struct LessonsView {
        #[template_child]
        pub lessons_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub layout_banner: TemplateChild<adw::Banner>,
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

    impl ObjectImpl for LessonsView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_language_communication();
            self.populate_lessons();
        }
    }

    impl WidgetImpl for LessonsView {}
    impl NavigationPageImpl for LessonsView {}

    impl LessonsView {
        fn setup_language_communication(&self) {
            // Warn when the system locale is not explicitly supported and we
            // fell back to US English.
            if crate::utils::supported_language_from_locale().is_none() {
                self.layout_banner.set_title(&gettext(
                    "Your system keyboard layout isn’t supported yet — showing US English",
                ));
                self.layout_banner.set_revealed(true);
            }
        }

        fn populate_lessons(&self) {
            let language = crate::utils::language_from_locale();
            let Ok(course) = Course::new_with_language(language) else {
                return;
            };

            for lesson in course.get_lessons() {
                let row = adw::ActionRow::builder()
                    .title(&lesson.title)
                    .subtitle(&lesson.description)
                    .activatable(true)
                    .build();

                let lesson_id = lesson.id;
                let view = self.obj().downgrade();
                row.connect_activated(move |_| {
                    if let Some(view) = view.upgrade() {
                        view.open_lesson(lesson_id);
                    }
                });

                self.lessons_group.add(&row);
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

    fn open_lesson(&self, lesson_id: u32) {
        // Persist the selected lesson and reset progress to its beginning.
        let settings = gio::Settings::new("io.github.nacho.mecalin");
        settings.set_uint("current-lesson", lesson_id).ok();
        settings.set_uint("current-step", 0).ok();

        // Push the lesson page onto the navigation view.
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
