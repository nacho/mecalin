use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/mecalin/ui/target_text_view.ui")]
    pub struct TargetTextView {
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
        pub cursor_position: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TargetTextView {
        const NAME: &'static str = "MecalinTargetTextView";
        type Type = super::TargetTextView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TargetTextView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_text_view();
        }
    }

    impl WidgetImpl for TargetTextView {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            self.draw_cursor(snapshot);
        }
    }

    impl BoxImpl for TargetTextView {}
}

impl imp::TargetTextView {
    fn setup_text_view(&self) {
        self.text_view.set_can_target(false);
        self.text_view.set_cursor_visible(false);
        self.text_view.set_monospace(true);
    }

    fn draw_cursor(&self, snapshot: &gtk::Snapshot) {
        let cursor_pos = self.cursor_position.get();
        let buffer = self.text_view.buffer();

        let mut iter = buffer.start_iter();
        iter.forward_chars(cursor_pos);
        let rect = self.text_view.iter_location(&iter);

        // Convert text view coordinates to widget coordinates
        let (x, y) =
            self.text_view
                .buffer_to_window_coords(gtk::TextWindowType::Widget, rect.x(), rect.y());

        // Get the text view's allocation to offset properly
        let allocation = self.text_view.allocation();
        let final_x = allocation.x() + x;
        let final_y = allocation.y() + y;

        // Get caret color from style context
        #[allow(deprecated)]
        let style_ctx = self.obj().style_context();
        #[allow(deprecated)]
        let color = style_ctx.color();

        let cursor_rect =
            gtk::graphene::Rect::new(final_x as f32, final_y as f32, 2.0, rect.height() as f32);
        snapshot.append_color(&color, &cursor_rect);
    }
}

glib::wrapper! {
    pub struct TargetTextView(ObjectSubclass<imp::TargetTextView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TargetTextView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn text_view(&self) -> &gtk::TextView {
        &self.imp().text_view
    }

    pub fn set_text(&self, text: &str) {
        let buffer = self.text_view().buffer();
        buffer.set_text(text);
        buffer.place_cursor(&buffer.start_iter());
    }

    pub fn set_cursor_position(&self, position: i32) {
        let imp = self.imp();
        imp.cursor_position.set(position);
        self.queue_draw();
    }
}

impl Default for TargetTextView {
    fn default() -> Self {
        Self::new()
    }
}
