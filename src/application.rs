use gio::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

use crate::window::MecalinWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MecalinApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for MecalinApplication {
        const NAME: &'static str = "MecalinApplication";
        type Type = super::MecalinApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MecalinApplication {}
    impl ApplicationImpl for MecalinApplication {
        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.set_resource_base_path(Some("/io/github/nacho/mecalin"));

            // Setup actions
            app.setup_actions();

            // Set keyboard shortcuts
            app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
            app.set_accels_for_action("window.close", &["<Ctrl>W"]);
            app.set_accels_for_action("app.preferences", &["<Ctrl>comma"]);

            // Load CSS
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/io/github/nacho/mecalin/style.css");
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("Could not connect to a display"),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        fn activate(&self) {
            let app = self.obj();
            let window = MecalinWindow::new(app.upcast_ref());
            window.load_window_state();
            window.present();
        }
    }
    impl GtkApplicationImpl for MecalinApplication {}
    impl AdwApplicationImpl for MecalinApplication {}
}

glib::wrapper! {
    pub struct MecalinApplication(ObjectSubclass<imp::MecalinApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MecalinApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "io.github.nacho.mecalin")
            .build()
    }

    fn setup_actions(&self) {
        self.add_action_entries(vec![
            gio::ActionEntry::builder("quit")
                .activate(|app: &Self, _, _| app.quit())
                .build(),
            gio::ActionEntry::builder("preferences")
                .activate(|app: &Self, _, _| {
                    if let Some(window) = app.active_window()
                        && let Ok(window) = window.downcast::<MecalinWindow>()
                    {
                        window.show_preferences();
                    }
                })
                .build(),
            gio::ActionEntry::builder("about")
                .activate(|app: &Self, _, _| {
                    if let Some(window) = app.active_window()
                        && let Ok(window) = window.downcast::<MecalinWindow>()
                    {
                        window.show_about();
                    }
                })
                .build(),
        ]);
    }
}
