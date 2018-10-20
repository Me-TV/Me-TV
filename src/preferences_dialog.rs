/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::cell::Cell;
use std::sync::Mutex;

use gtk;
use gtk::prelude::*;

use control_window::ControlWindow;
use metvcomboboxtext::MeTVComboBoxText;
use metvcomboboxtext::MeTVComboBoxTextExt;
use preferences;

lazy_static! {
    static ref PREFERENCES: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
}

fn create(control_window: &ControlWindow) -> gtk::Window {
    let menu_builder = gtk::Builder::new_from_string(include_str!("resources/preferences_dialog.glade.xml"));
    let use_opengl_button = menu_builder.get_object::<gtk::CheckButton>("use_opengl").unwrap();
    use_opengl_button.set_active(preferences::get_use_opengl());
    use_opengl_button.connect_toggled(
        move |button| preferences::set_use_opengl(button.get_active(), true)
    );
    let immediate_tv_button = menu_builder.get_object::<gtk::CheckButton>("immediate_tv").unwrap();
    immediate_tv_button.set_active(preferences::get_immediate_tv());
    immediate_tv_button.connect_toggled(
        move |button| preferences::set_immediate_tv(button.get_active(), true)
    );
    let  use_last_channel_button = menu_builder.get_object::<gtk::RadioButton>("last_channel").unwrap();
    let  use_default_channel_button = menu_builder.get_object::<gtk::RadioButton>("default_channel").unwrap();
    use_default_channel_button.join_group(Some(&use_last_channel_button));
    if preferences::get_use_last_channel() { use_last_channel_button.set_active(true); }
    else { use_default_channel_button.set_active(true); }
    use_last_channel_button.connect_clicked(
        move |_| preferences::set_use_last_channel(true, true)
    );
    use_default_channel_button.connect_clicked(
        move |_| preferences::set_use_last_channel(false, true)
    );
    let mut default_channel_selector = menu_builder.get_object::<MeTVComboBoxText>("channel_name").unwrap();
    default_channel_selector.set_new_model(&control_window.channel_names_store);
    if let Some(channel_name) = preferences::get_default_channel() {
        if channel_name != "" {
            if ! default_channel_selector.set_active_text(channel_name.clone()) {
                panic!("Could not set the default channel to {}.", channel_name);
            }
        }
    }
    default_channel_selector.connect_changed(
        move |selector: &MeTVComboBoxText| preferences::set_default_channel(selector.get_active_text().unwrap(), true)
    );
    let preferences_dialog = menu_builder.get_object::<gtk::Window>("preferences_dialog").unwrap();
    preferences_dialog.set_transient_for(&control_window.window);
    preferences_dialog.show_all();
    preferences_dialog
}

/// Display a preferences dialog in a non-modal way, but only if one is not already being displayed.
pub fn present(control_window: &ControlWindow) {
    if let Ok(active) = PREFERENCES.lock() {
        if ! active.get() {
            let dialog = create(control_window);
            dialog.connect_destroy(move |d| {
                if let Ok(active) = PREFERENCES.lock() {
                    d.destroy();
                    active.set(false);
                }
            });
            dialog.show();
            active.set(true);
        }
    }
}
