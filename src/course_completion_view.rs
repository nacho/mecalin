use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

/// Keypunch application ID (used both as the themed icon name and the
/// software-center `appstream:` target).
const KEYPUNCH_APP_ID: &str = "no.bragefuglseth.Keypunch";
/// URI opened in the software center when the Keypunch row is activated.
const KEYPUNCH_APPSTREAM_URI: &str = "appstream:no.bragefuglseth.Keypunch";
/// Flathub web page used as a fallback when no `appstream:` handler exists.
const KEYPUNCH_FLATHUB_URL: &str = "https://flathub.org/apps/no.bragefuglseth.Keypunch";
/// Bundled fallback icon shown when the themed Keypunch icon is not installed.
const KEYPUNCH_FALLBACK_ICON_RESOURCE: &str =
    "/io/github/nacho/mecalin/icons/scalable/actions/keypunch-fallback-symbolic.svg";

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/course_completion_view.ui")]
    pub struct CourseCompletionView {
        #[template_child]
        pub keypunch_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub keypunch_icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CourseCompletionView {
        const NAME: &'static str = "CourseCompletionView";
        type Type = super::CourseCompletionView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CourseCompletionView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_keypunch_row();
        }
    }
    impl WidgetImpl for CourseCompletionView {}
    impl NavigationPageImpl for CourseCompletionView {}
}

glib::wrapper! {
    pub struct CourseCompletionView(ObjectSubclass<imp::CourseCompletionView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CourseCompletionView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_keypunch_row(&self) {
        let imp = self.imp();

        // Resolve the icon: prefer the themed app icon (present when Keypunch is
        // installed), otherwise use the bundled fallback icon. This mirrors the
        // Adwaita "other apps" approach but adds a fallback, which Adwaita lacks.
        self.resolve_keypunch_icon();

        imp.keypunch_row.connect_activated(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.launch_keypunch();
            }
        ));
    }

    fn resolve_keypunch_icon(&self) {
        let imp = self.imp();
        let display = self.display();
        let use_themed = gtk::IconTheme::for_display(&display).has_icon(KEYPUNCH_APP_ID);

        if use_themed {
            imp.keypunch_icon.set_icon_name(Some(KEYPUNCH_APP_ID));
        } else {
            imp.keypunch_icon
                .set_resource(Some(KEYPUNCH_FALLBACK_ICON_RESOURCE));
        }
    }

    fn launch_keypunch(&self) {
        // Try the software center first; fall back to the Flathub web page if
        // no handler is registered for the `appstream:` scheme.
        let launcher = gtk::UriLauncher::new(KEYPUNCH_APPSTREAM_URI);
        let parent_window = self.root().and_downcast::<gtk::Window>();

        launcher.launch(
            parent_window.as_ref(),
            gio::Cancellable::NONE,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move |result| {
                    if result.is_err() {
                        view.launch_flathub_fallback();
                    }
                }
            ),
        );
    }

    fn launch_flathub_fallback(&self) {
        let launcher = gtk::UriLauncher::new(KEYPUNCH_FLATHUB_URL);
        let parent_window = self.root().and_downcast::<gtk::Window>();
        launcher.launch(parent_window.as_ref(), gio::Cancellable::NONE, |result| {
            if let Err(error) = result {
                glib::g_warning!("mecalin", "Failed to open Keypunch Flathub page: {error}");
            }
        });
    }
}

impl Default for CourseCompletionView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypunch_uris_match_app_id() {
        assert_eq!(KEYPUNCH_APP_ID, "no.bragefuglseth.Keypunch");
        assert_eq!(
            KEYPUNCH_APPSTREAM_URI,
            "appstream:no.bragefuglseth.Keypunch"
        );
        assert_eq!(
            KEYPUNCH_FLATHUB_URL,
            "https://flathub.org/apps/no.bragefuglseth.Keypunch"
        );
    }

    #[test]
    fn test_appstream_uri_uses_app_id() {
        assert_eq!(
            KEYPUNCH_APPSTREAM_URI,
            format!("appstream:{KEYPUNCH_APP_ID}")
        );
    }

    #[test]
    fn test_flathub_url_uses_app_id() {
        assert_eq!(
            KEYPUNCH_FLATHUB_URL,
            format!("https://flathub.org/apps/{KEYPUNCH_APP_ID}")
        );
    }
}
