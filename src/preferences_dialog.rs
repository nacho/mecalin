use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::*;
use i18n_format::i18n_format;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::course::Course;

/// Build the standard Adwaita preferences dialog, wired to GSettings.
pub fn build() -> adw::PreferencesDialog {
    let settings = gio::Settings::new("io.github.nacho.mecalin");

    let page = adw::PreferencesPage::new();

    // Display group
    let display_group = adw::PreferencesGroup::builder()
        .title(gettext("Display"))
        .build();

    let show_hand_switch = adw::SwitchRow::builder()
        .title(gettext("Show Hand Widget"))
        .subtitle(gettext("Display finger position guide"))
        .build();
    let use_finger_colors_switch = adw::SwitchRow::builder()
        .title(gettext("Use Finger Colors"))
        .subtitle(gettext("Color keyboard keys and hand by finger assignment"))
        .build();

    display_group.add(&show_hand_switch);
    display_group.add(&use_finger_colors_switch);

    settings
        .bind("show-hand-widget", &show_hand_switch, "active")
        .build();
    settings
        .bind("use-finger-colors", &use_finger_colors_switch, "active")
        .build();

    // Lesson Progress group
    let lesson_group = adw::PreferencesGroup::builder()
        .title(gettext("Lesson Progress"))
        .build();

    let lesson_combo = adw::ComboRow::builder()
        .title(gettext("Current Lesson"))
        .subtitle(gettext("Select active lesson (resets step to beginning)"))
        .use_subtitle(true)
        .build();

    let layout_code = crate::utils::language_from_locale();
    if let Ok(course) = Course::new_with_language(layout_code) {
        let lesson_names: Vec<String> = course
            .get_lessons()
            .iter()
            .enumerate()
            .map(|(i, lesson)| i18n_format!("Lesson {}: {}", i, &lesson.title))
            .collect();
        let lesson_strs: Vec<&str> = lesson_names.iter().map(|s| s.as_str()).collect();
        let lesson_model = gtk::StringList::new(&lesson_strs);
        lesson_combo.set_model(Some(&lesson_model));
        lesson_combo.set_selected(settings.uint("current-lesson"));

        lesson_combo.connect_selected_notify(move |combo| {
            let settings = gio::Settings::new("io.github.nacho.mecalin");
            settings.set_uint("current-lesson", combo.selected()).ok();
            // Reset step to 0 when lesson changes
            settings.set_uint("current-step", 0).ok();
        });
    }

    lesson_group.add(&lesson_combo);

    page.add(&display_group);
    page.add(&lesson_group);

    let dialog = adw::PreferencesDialog::new();
    dialog.add(&page);
    dialog
}
