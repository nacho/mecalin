/* speed_test_view.rs
 *
 * Speed test view integrating Keypunch functionality
 * SPDX-FileCopyrightText: © 2024–2025 Brage Fuglseth <bragefuglseth@gnome.org>
 * SPDX-FileCopyrightText: © 2026 Ignacio Casal Quinteiro
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::speed_test_results_view::SpeedTestResultsView;
use crate::speed_test_text_view::SpeedTestTextView;
use crate::text_generation::{advanced, simple, Language};
use crate::typing_test_utils::{GeneratedTestDifficulty, TestConfig, TestDuration, TestSummary};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Instant;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/github/nacho/mecalin/ui/speed_test_view.ui")]
    pub struct SpeedTestView {
        #[template_child]
        pub text_view: TemplateChild<SpeedTestTextView>,
        #[template_child]
        pub results_view: TemplateChild<SpeedTestResultsView>,
        #[template_child]
        pub timer_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub text_type_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub duration_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub settings_box: TemplateChild<gtk::Box>,

        pub start_time: Rc<RefCell<Option<Instant>>>,
        pub timer_source_id: Rc<RefCell<Option<glib::SourceId>>>,
        pub test_duration: Rc<RefCell<TestDuration>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SpeedTestView {
        const NAME: &'static str = "SpeedTestView";
        type Type = super::SpeedTestView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                text_view: Default::default(),
                results_view: Default::default(),
                timer_label: Default::default(),
                text_type_dropdown: Default::default(),
                duration_dropdown: Default::default(),
                settings_box: Default::default(),
                start_time: Rc::new(RefCell::new(None)),
                timer_source_id: Rc::new(RefCell::new(None)),
                test_duration: Rc::new(RefCell::new(TestDuration::Sec30)),
            }
        }
    }

    impl ObjectImpl for SpeedTestView {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.setup_signals();
            obj.reset_test();

            let text_view = self.text_view.clone();
            glib::idle_add_local_once(move || {
                text_view.grab_focus();
            });
        }
    }

    impl WidgetImpl for SpeedTestView {}
    impl BoxImpl for SpeedTestView {}
}

glib::wrapper! {
    pub struct SpeedTestView(ObjectSubclass<imp::SpeedTestView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Orientable;
}

impl SpeedTestView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn get_duration(&self) -> TestDuration {
        match self.imp().duration_dropdown.selected() {
            0 => TestDuration::Sec15,
            1 => TestDuration::Sec30,
            2 => TestDuration::Min1,
            3 => TestDuration::Min2,
//          3 => TestDuration::Min5,
            _ => TestDuration::Min1,
        }
    }

    fn reset_test(&self) {
        let imp = self.imp();
        if let Some(source_id) = imp.timer_source_id.borrow_mut().take() {
            source_id.remove();
        }
        *imp.start_time.borrow_mut() = None;

        let duration = self.get_duration();
        *imp.test_duration.borrow_mut() = duration;

        imp.settings_box.set_visible(true);
        imp.timer_label.set_visible(false);

        let lang_code = crate::utils::language_from_locale();
        let language = Language::from_str(lang_code).unwrap_or(Language::English);

        let text = if imp.text_type_dropdown.selected() == 0 {
            simple(language)
        } else {
            advanced(language)
        };

        imp.text_view.set_original_text(&text);
        imp.text_view.set_typed_text("");
        imp.text_view.set_running(false);
        imp.text_view.set_accepts_input(true);
        imp.text_view.set_visible(true);
        imp.text_view.grab_focus();
        imp.results_view.set_visible(false);
    }

    fn show_results(&self, start_instant: Instant) {
        let imp = self.imp();

        let lang_code = crate::utils::language_from_locale();
        let language = Language::from_str(lang_code).unwrap_or(Language::English);

        let difficulty = if imp.text_type_dropdown.selected() == 0 {
            GeneratedTestDifficulty::Simple
        } else {
            GeneratedTestDifficulty::Advanced
        };

        let config = TestConfig::Generated {
            difficulty,
            language,
            duration: *imp.test_duration.borrow(),
        };

        let keystrokes = imp.text_view.keystrokes();
        let keystrokes_vec: Vec<_> = keystrokes.iter().copied().collect();

        let summary = TestSummary::new(
            std::time::SystemTime::now(),
            start_instant,
            Instant::now(),
            config,
            &imp.text_view.original_text(),
            &imp.text_view.typed_text(),
            &keystrokes_vec,
        );

        imp.results_view.set_summary(summary);
        imp.results_view.set_visible(true);
        imp.text_view.set_visible(false);
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.results_view.connect_closure(
            "retry",
            false,
            glib::closure_local!(
                #[weak(rename_to = view)]
                self,
                move |_results_view: SpeedTestResultsView| {
                    view.reset_test();
                }
            ),
        );

        imp.text_type_dropdown.connect_selected_notify(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.reset_test();
            }
        ));

        imp.duration_dropdown.connect_selected_notify(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.reset_test();
            }
        ));

        imp.text_view.connect_closure(
            "typed-text-changed",
            false,
            glib::closure_local!(
                #[weak(rename_to = view)]
                self,
                move |text_view: SpeedTestTextView| {
                    let imp = view.imp();
                    let typed = text_view.typed_text();

                    if typed.len() == 1 && imp.start_time.borrow().is_none() {
                        text_view.set_running(true);
                        *imp.start_time.borrow_mut() = Some(Instant::now());

                        imp.settings_box.set_visible(false);
                        imp.timer_label.set_visible(true);

                        let timer_label = imp.timer_label.clone();
                        let start_time = imp.start_time.clone();
                        let text_view_clone = text_view.clone();
                        let timer_source_id = imp.timer_source_id.clone();
                        let test_duration = imp.test_duration.clone();
                        let view_weak = view.downgrade();
                        let source_id = glib::timeout_add_local(
                            std::time::Duration::from_millis(100),
                            move || {
                                let start_opt = *start_time.borrow();
                                if let Some(start) = start_opt {
                                    let elapsed = start.elapsed();
                                    let duration_secs = test_duration.borrow().as_seconds();
                                    let remaining = duration_secs.saturating_sub(elapsed.as_secs());

                                    if remaining == 0 {
                                        text_view_clone.set_running(false);
                                        text_view_clone.set_accepts_input(false);
                                        timer_label.set_text("0:00");
                                        *start_time.borrow_mut() = None;
                                        *timer_source_id.borrow_mut() = None;

                                        if let Some(view) = view_weak.upgrade() {
                                            view.show_results(start);
                                        }
                                        return glib::ControlFlow::Break;
                                    }

                                    let minutes = remaining / 60;
                                    let seconds = remaining % 60;
                                    timer_label.set_text(&format!("{}:{:02}", minutes, seconds));
                                    glib::ControlFlow::Continue
                                } else {
                                    *timer_source_id.borrow_mut() = None;
                                    glib::ControlFlow::Break
                                }
                            },
                        );
                        *imp.timer_source_id.borrow_mut() = Some(source_id);
                    }

                    let original = text_view.original_text();
                    if typed.len() >= original.len() {
                        text_view.set_running(false);
                        text_view.set_accepts_input(false);

                        let imp = view.imp();
                        let start_instant = *imp.start_time.borrow();
                        *imp.start_time.borrow_mut() = None;
                        if let Some(source_id) = imp.timer_source_id.borrow_mut().take() {
                            source_id.remove();
                        }

                        if let Some(start) = start_instant {
                            view.show_results(start);
                        }
                    }
                }
            ),
        );
    }
}

impl Default for SpeedTestView {
    fn default() -> Self {
        Self::new()
    }
}
