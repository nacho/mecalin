use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use libadwaita as adw;
use libadwaita::prelude::AdwDialogExt;
use libadwaita::subclass::prelude::*;

use crate::config;
use crate::course::Lesson;
use crate::lesson_view::LessonView;
use crate::main_action_list::MainActionList;
use crate::study_room::StudyRoom;
use crate::target_text_view::TargetTextView;
use crate::text_view::TextView;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/window.ui")]
    pub struct MecalinWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub main_action_list_widget: TemplateChild<MainActionList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinWindow {
        const NAME: &'static str = "MecalinWindow";
        type Type = super::MecalinWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            MainActionList::ensure_type();
            StudyRoom::ensure_type();
            LessonView::ensure_type();
            TextView::ensure_type();
            TargetTextView::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MecalinWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_signals();
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

    pub fn show_study_room(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("study_room");
        imp.back_button.set_visible(true);
        self.setup_lesson_view_signals();
    }

    pub fn go_back(&self) {
        let imp = self.imp();
        let current_page = imp.main_stack.visible_child_name();

        if let Some("study_room") = current_page.as_deref() {
            // Check if we're in a lesson view within the study room
            if let Some(study_room) = imp.main_stack.visible_child() {
                if let Ok(study_room) = study_room.downcast::<StudyRoom>() {
                    if study_room.can_go_back() {
                        study_room.go_back();
                        return;
                    }
                }
            }
            // If we can't go back within study room, go to main menu
            imp.main_stack.set_visible_child_name("main_menu");
            imp.back_button.set_visible(false);
            imp.window_title.set_title("Mecalin");
            imp.window_title.set_subtitle("");
        }
    }

    pub fn show_about(&self) {
        let about = adw::AboutDialog::builder()
            .application_name("Mecalin")
            .application_icon(config::APPLICATION_ID)
            .developer_name("Ignacio Casal Quinteiro")
            .version(config::VERSION)
            .website("https://github.com/mecalin/mecalin")
            .issue_url("https://github.com/mecalin/mecalin/issues")
            .copyright("© 2026 Ignacio Casal Quinteiro")
            .license_type(gtk::License::Gpl20)
            .build();

        about.present(Some(self));
    }

    pub fn set_title(&self, title: &str) {
        let imp = self.imp();
        imp.window_title.set_title(title);
    }

    pub fn set_subtitle(&self, subtitle: &str) {
        let imp = self.imp();
        imp.window_title.set_subtitle(subtitle);
    }

    fn setup_lesson_view_signals(&self) {
        let imp = self.imp();
        if let Some(study_room) = imp.main_stack.child_by_name("study_room") {
            if let Ok(study_room) = study_room.downcast::<StudyRoom>() {
                let lesson_view = study_room.lesson_view();

                let window = self.downgrade();
                lesson_view.connect_notify_local(Some("current-lesson"), move |lesson_view, _| {
                    if let Some(window) = window.upgrade() {
                        window.update_title_from_lesson_view(lesson_view);
                    }
                });

                let window = self.downgrade();
                lesson_view.connect_notify_local(
                    Some("current-step-index"),
                    move |lesson_view, _| {
                        if let Some(window) = window.upgrade() {
                            window.update_title_from_lesson_view(lesson_view);
                        }
                    },
                );
            }
        }
    }

    fn update_title_from_lesson_view(&self, lesson_view: &LessonView) {
        if let Some(lesson_boxed) = lesson_view.current_lesson() {
            if let Ok(lesson) = lesson_boxed.try_borrow::<Lesson>() {
                self.set_title(&lesson.title);

                if lesson.introduction {
                    let subtitle = i18n_fmt! { i18n_fmt("Lesson {}", lesson.id) };
                    self.set_subtitle(&subtitle);
                } else {
                    let current_step = lesson_view.current_step_index() as usize;
                    let total_steps = lesson.steps.len();
                    let subtitle = i18n_fmt! { i18n_fmt("Lesson {}: Step {}/{}", lesson.id, current_step + 1, total_steps) };
                    self.set_subtitle(&subtitle);
                }
            }
        } else {
            // No lesson selected, reset to default title
            self.set_title("Mecalin");
            self.set_subtitle("");
        }
    }

    pub fn load_window_state(&self) {
        let settings = gio::Settings::new("org.gnome.mecalin.state.window");

        let (width, height) = settings.get::<(i32, i32)>("size");
        self.set_default_size(width, height);

        if settings.boolean("maximized") {
            self.maximize();
        }

        self.connect_notify_local(Some("maximized"), move |window, _| {
            let settings = gio::Settings::new("org.gnome.mecalin.state.window");
            settings
                .set_boolean("maximized", window.is_maximized())
                .unwrap();
        });

        self.connect_notify_local(Some("default-width"), move |window, _| {
            let settings = gio::Settings::new("org.gnome.mecalin.state.window");
            if !window.is_maximized() {
                let size = (window.default_width(), window.default_height());
                settings.set("size", size).unwrap();
            }
        });

        self.connect_notify_local(Some("default-height"), move |window, _| {
            let settings = gio::Settings::new("org.gnome.mecalin.state.window");
            if !window.is_maximized() {
                let size = (window.default_width(), window.default_height());
                settings.set("size", size).unwrap();
            }
        });
    }
}

impl imp::MecalinWindow {
    fn setup_signals(&self) {
        let window = self.obj().downgrade();
        self.main_action_list_widget
            .connect_local("study-room-selected", false, move |_| {
                if let Some(window) = window.upgrade() {
                    window.show_study_room();
                }
                None
            });

        let window = self.obj().downgrade();
        self.main_action_list_widget
            .connect_local("about-selected", false, move |_| {
                if let Some(window) = window.upgrade() {
                    window.show_about();
                }
                None
            });

        let window = self.obj().downgrade();
        self.back_button.connect_clicked(move |_| {
            if let Some(window) = window.upgrade() {
                window.go_back();
            }
        });
    }
}
