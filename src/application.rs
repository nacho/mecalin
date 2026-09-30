use gio::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::window::TeclazoWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct TeclazoApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for TeclazoApplication {
        const NAME: &'static str = "TeclazoApplication";
        type Type = super::TeclazoApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for TeclazoApplication {}
    impl ApplicationImpl for TeclazoApplication {
        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.set_resource_base_path(Some("/io/github/nacho/teclazo"));

            // Setup actions
            app.setup_actions();

            // Set keyboard shortcuts
            app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
            app.set_accels_for_action("window.close", &["<Ctrl>W"]);

            // Load CSS
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/io/github/nacho/teclazo/style.css");
            gtk::style_context_add_provider_for_display(
                &gtk::gdk::Display::default().expect("Could not connect to a display"),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        fn activate(&self) {
            let app = self.obj();
            let window = TeclazoWindow::new(app.upcast_ref());
            window.load_window_state();
            window.present();
        }
    }
    impl GtkApplicationImpl for TeclazoApplication {}
    impl AdwApplicationImpl for TeclazoApplication {}
}

glib::wrapper! {
    pub struct TeclazoApplication(ObjectSubclass<imp::TeclazoApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl TeclazoApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "io.github.nacho.teclazo")
            .build()
    }

    fn setup_actions(&self) {
        self.add_action_entries(vec![
            gio::ActionEntry::builder("quit")
                .activate(|app: &Self, _, _| app.quit())
                .build(),
            gio::ActionEntry::builder("about")
                .activate(|app: &Self, _, _| {
                    let dialog = adw::AboutDialog::builder()
                        .application_name("Teclazo")
                        .application_icon(crate::config::APPLICATION_ID)
                        .version(crate::config::VERSION)
                        .developer_name("Ignacio Casal Quinteiro")
                        .developers(["Ignacio Casal Quinteiro"])
                        .license_type(gtk::License::Gpl30)
                        .website("https://github.com/nacho/teclazo")
                        .issue_url("https://github.com/nacho/teclazo/issues")
                        .copyright("© 2026 Ignacio Casal Quinteiro")
                        .build();
                    dialog.present(app.active_window().as_ref());
                })
                .build(),
        ]);
    }
}
