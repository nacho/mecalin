use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/text_view.ui")]
    pub struct TextView {
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextView {
        const NAME: &'static str = "MecalinTextView";
        type Type = super::TextView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TextView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_text_view();
        }
    }
    impl WidgetImpl for TextView {}
    impl BoxImpl for TextView {}
}

impl imp::TextView {
    fn setup_text_view(&self) {
        self.text_view.set_monospace(true);
        // Ensure dead keys (like acute accent ´) work properly without double presses
        self.text_view.set_input_hints(gtk::InputHints::NONE);
    }
}

glib::wrapper! {
    pub struct TextView(ObjectSubclass<imp::TextView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TextView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn text_view(&self) -> &gtk::TextView {
        &self.imp().text_view
    }

    pub fn set_text(&self, text: &str) {
        let buffer = self.text_view().buffer();
        buffer.set_text(text);
    }

    pub fn text(&self) -> String {
        let buffer = self.text_view().buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        buffer.text(&start, &end, false).to_string()
    }
}

impl Default for TextView {
    fn default() -> Self {
        Self::new()
    }
}
