use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

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

    page.add(&display_group);

    let dialog = adw::PreferencesDialog::new();
    dialog.add(&page);
    dialog
}
