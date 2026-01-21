use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use i18n_format::i18n_fmt;
use libadwaita as adw;
use libadwaita::prelude::{ActionRowExt, AdwDialogExt};
use libadwaita::subclass::prelude::*;

use crate::config;
use crate::course::Lesson;
use crate::falling_keys_game::FallingKeysGame;
use crate::lesson_view::LessonView;
use crate::scrolling_lanes_game::ScrollingLanesGame;
use crate::speed_test_view::SpeedTestView;
use crate::typing_row::TypingRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/window.ui")]
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
        pub lessons_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub speed_test_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub falling_keys_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub scrolling_lanes_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub about_row: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinWindow {
        const NAME: &'static str = "MecalinWindow";
        type Type = super::MecalinWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            LessonView::ensure_type();
            TypingRow::ensure_type();
            FallingKeysGame::ensure_type();
            ScrollingLanesGame::ensure_type();
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
            self.obj().setup_lesson_view_signals();
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

    pub fn show_lessons(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("lessons");
        imp.back_button.set_visible(true);

        if let Some(lesson_view) = imp.main_stack.child_by_name("lessons") {
            lesson_view.grab_focus();

            if let Ok(lesson_view) = lesson_view.downcast::<LessonView>() {
                self.update_title_from_lesson_view(&lesson_view);
            }
        }
    }

    pub fn show_game(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("game");
        imp.back_button.set_visible(true);
        imp.window_title.set_title(&gettext("Falling Keys"));
        imp.window_title.set_subtitle("");

        // Reset game when showing
        if let Some(game) = imp.main_stack.child_by_name("game") {
            if let Ok(game) = game.downcast::<FallingKeysGame>() {
                game.reset();
            }
        }
    }

    pub fn show_lanes_game(&self) {
        let imp = self.imp();
        imp.main_stack.set_visible_child_name("lanes_game");
        imp.back_button.set_visible(true);
        imp.window_title.set_title(&gettext("Scrolling Lanes"));
        imp.window_title.set_subtitle("");

        // Reset game when showing
        if let Some(game) = imp.main_stack.child_by_name("lanes_game") {
            if let Ok(game) = game.downcast::<ScrollingLanesGame>() {
                game.reset();
            }
        }
    }

    pub fn show_speed_test(&self) {
        let imp = self.imp();

        // Create speed test view if it doesn't exist
        if imp.main_stack.child_by_name("speed_test").is_none() {
            let speed_test = SpeedTestView::new();
            imp.main_stack
                .add_named(speed_test.widget(), Some("speed_test"));
        }

        imp.main_stack.set_visible_child_name("speed_test");
        imp.back_button.set_visible(true);
        imp.window_title.set_title(&gettext("Speed Test"));
        imp.window_title.set_subtitle("");
    }

    pub fn go_back(&self) {
        let imp = self.imp();
        let current_page = imp.main_stack.visible_child_name();

        if let Some("lessons" | "game" | "lanes_game" | "speed_test") = current_page.as_deref() {
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
            .website("https://github.com/nacho/mecalin")
            .issue_url("https://github.com/nacho/mecalin/issues")
            .copyright("© 2026 Ignacio Casal Quinteiro\n© 2024–2025 Brage Fuglseth")
            .license_type(gtk::License::Gpl30)
            .build();

        about.add_credit_section(
            Some("Code by"),
            &[
                "Ignacio Casal Quinteiro",
                "Brage Fuglseth https://bragefuglseth.dev",
            ],
        );

        about.add_credit_section(
            Some("Speed Test Orthography"),
            &[
                "Angelo Verlain Shema https://www.vixalien.com/",
                "Arnob Goswami",
                "Daniel Uhrinyi",
                "Fabio Lovato https://loviuz.it/",
                "Gregor Niehl https://gregorni.gitlab.io/",
                "Hadi Azarnasab https://hadi7546.ir/",
                "Ibrahim Muhammad",
                "Kim Jimin https://developomp.com",
                "Shellheim",
                "Tamazight teachers of Tizi Ouzou",
                "Urtsi Santsi",
                "Yevhen Popok",
            ],
        );

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
        if let Some(lesson_view) = imp.main_stack.child_by_name("lessons") {
            if let Ok(lesson_view) = lesson_view.downcast::<LessonView>() {
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

impl imp::MecalinWindow {
    fn setup_signals(&self) {
        let window = self.obj().downgrade();
        self.lessons_row.connect_activated(move |_| {
            if let Some(window) = window.upgrade() {
                window.show_lessons();
            }
        });

        let window = self.obj().downgrade();
        self.falling_keys_row.connect_activated(move |_| {
            if let Some(window) = window.upgrade() {
                window.show_game();
            }
        });

        let window = self.obj().downgrade();
        self.scrolling_lanes_row.connect_activated(move |_| {
            if let Some(window) = window.upgrade() {
                window.show_lanes_game();
            }
        });

        let window = self.obj().downgrade();
        self.speed_test_row.connect_activated(move |_| {
            if let Some(window) = window.upgrade() {
                window.show_speed_test();
            }
        });

        let window = self.obj().downgrade();
        self.about_row.connect_activated(move |_| {
            if let Some(window) = window.upgrade() {
                window.show_about();
            }
        });

        let window = self.obj().downgrade();
        self.back_button.connect_clicked(move |_| {
            if let Some(window) = window.upgrade() {
                window.go_back();
            }
        });
    }
}
