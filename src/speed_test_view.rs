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
use gtk::prelude::*;
use gtk::{glib, Box, DropDown, Label, Orientation, StringList};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct SpeedTestView {
    container: Box,
    text_view: SpeedTestTextView,
    results_view: SpeedTestResultsView,
    timer_label: Label,
    text_type_dropdown: DropDown,
    duration_dropdown: DropDown,
    start_time: Rc<RefCell<Option<Instant>>>,
    timer_source_id: Rc<RefCell<Option<glib::SourceId>>>,
    test_duration: Rc<RefCell<u64>>,
    settings_box: Box,
}

impl SpeedTestView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 24);
        container.set_margin_top(48);
        container.set_margin_bottom(48);
        container.set_margin_start(48);
        container.set_margin_end(48);
        container.set_vexpand(true);
        container.set_valign(gtk::Align::Fill);

        // Settings row
        let settings_box = Box::new(Orientation::Horizontal, 12);
        settings_box.set_halign(gtk::Align::Center);
        settings_box.set_margin_bottom(12);

        let text_type_list = StringList::new(&["Simple", "Advanced"]);
        let text_type_dropdown = DropDown::new(Some(text_type_list), None::<gtk::Expression>);
        text_type_dropdown.set_selected(0);
        settings_box.append(&text_type_dropdown);

        let duration_list = StringList::new(&["15s", "30s", "60s", "120s"]);
        let duration_dropdown = DropDown::new(Some(duration_list), None::<gtk::Expression>);
        duration_dropdown.set_selected(2);
        settings_box.append(&duration_dropdown);

        container.append(&settings_box);

        // Timer label
        let timer_label = Label::new(None);
        timer_label.add_css_class("title-2");
        timer_label.set_halign(gtk::Align::Center);
        timer_label.set_margin_bottom(12);
        timer_label.set_visible(false);
        container.append(&timer_label);

        let text_view = glib::Object::new::<SpeedTestTextView>();
        text_view.set_valign(gtk::Align::Center);
        container.append(&text_view);

        let results_view = glib::Object::new::<SpeedTestResultsView>();
        results_view.set_visible(false);
        container.append(&results_view);

        let view = Self {
            container,
            text_view,
            results_view,
            timer_label,
            text_type_dropdown,
            duration_dropdown,
            start_time: Rc::new(RefCell::new(None)),
            timer_source_id: Rc::new(RefCell::new(None)),
            test_duration: Rc::new(RefCell::new(60)),
            settings_box,
        };

        view.setup_signals();
        view.reset_test();

        // Grab focus after construction
        let text_view_clone = view.text_view.clone();
        glib::idle_add_local_once(move || {
            text_view_clone.grab_focus();
        });

        view
    }

    fn get_duration_seconds(&self) -> u64 {
        match self.duration_dropdown.selected() {
            0 => 15,
            1 => 30,
            2 => 60,
            3 => 120,
            _ => 60,
        }
    }

    fn reset_test(&self) {
        if let Some(source_id) = self.timer_source_id.borrow_mut().take() {
            source_id.remove();
        }
        *self.start_time.borrow_mut() = None;

        let duration = self.get_duration_seconds();
        *self.test_duration.borrow_mut() = duration;

        self.settings_box.set_visible(true);
        self.timer_label.set_visible(false);

        let text = if self.text_type_dropdown.selected() == 0 {
            simple(Language::English)
        } else {
            advanced(Language::English)
        };

        self.text_view.set_original_text(&text);
        self.text_view.set_typed_text("");
        self.text_view.set_running(false);
        self.text_view.set_accepts_input(true);
        self.text_view.grab_focus();
        self.results_view.set_visible(false);
    }

    fn setup_signals(&self) {
        let results_view = self.results_view.clone();
        let timer_label = self.timer_label.clone();
        let start_time = self.start_time.clone();
        let timer_source_id = self.timer_source_id.clone();
        let test_duration = self.test_duration.clone();
        let settings_box = self.settings_box.clone();

        let self_weak = Rc::new(RefCell::new(None::<Self>));
        let self_weak_clone = self_weak.clone();
        *self_weak.borrow_mut() = Some(Self {
            container: self.container.clone(),
            text_view: self.text_view.clone(),
            results_view: self.results_view.clone(),
            timer_label: self.timer_label.clone(),
            text_type_dropdown: self.text_type_dropdown.clone(),
            duration_dropdown: self.duration_dropdown.clone(),
            start_time: self.start_time.clone(),
            timer_source_id: self.timer_source_id.clone(),
            test_duration: self.test_duration.clone(),
            settings_box: self.settings_box.clone(),
        });

        self.text_type_dropdown.connect_selected_notify(move |_| {
            if let Some(view) = self_weak.borrow().as_ref() {
                view.reset_test();
            }
        });

        let self_weak_clone2 = self_weak_clone.clone();
        self.duration_dropdown.connect_selected_notify(move |_| {
            if let Some(view) = self_weak_clone2.borrow().as_ref() {
                view.reset_test();
            }
        });

        self.text_view.connect_closure(
            "typed-text-changed",
            false,
            glib::closure_local!(move |text_view: SpeedTestTextView| {
                let typed = text_view.typed_text();

                if typed.len() == 1 && start_time.borrow().is_none() {
                    text_view.set_running(true);
                    *start_time.borrow_mut() = Some(Instant::now());

                    settings_box.set_visible(false);
                    timer_label.set_visible(true);

                    let timer_label_clone = timer_label.clone();
                    let start_time_clone = start_time.clone();
                    let text_view_clone = text_view.clone();
                    let timer_source_id_clone = timer_source_id.clone();
                    let test_duration_clone = test_duration.clone();
                    let source_id =
                        glib::timeout_add_local(Duration::from_millis(100), move || {
                            if let Some(start) = *start_time_clone.borrow() {
                                let elapsed = start.elapsed();
                                let duration_secs = *test_duration_clone.borrow();
                                let remaining = duration_secs.saturating_sub(elapsed.as_secs());

                                if remaining == 0 {
                                    text_view_clone.set_running(false);
                                    text_view_clone.set_accepts_input(false);
                                    timer_label_clone.set_text("0:00");
                                    *start_time_clone.borrow_mut() = None;
                                    *timer_source_id_clone.borrow_mut() = None;
                                    return glib::ControlFlow::Break;
                                }

                                let minutes = remaining / 60;
                                let seconds = remaining % 60;
                                timer_label_clone.set_text(&format!("{}:{:02}", minutes, seconds));
                                glib::ControlFlow::Continue
                            } else {
                                *timer_source_id_clone.borrow_mut() = None;
                                glib::ControlFlow::Break
                            }
                        });
                    *timer_source_id.borrow_mut() = Some(source_id);
                }

                let original = text_view.original_text();
                if typed.len() >= original.len() {
                    text_view.set_running(false);
                    text_view.set_accepts_input(false);

                    *start_time.borrow_mut() = None;
                    if let Some(source_id) = timer_source_id.borrow_mut().take() {
                        source_id.remove();
                    }

                    results_view.set_visible(true);
                }
            }),
        );
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
