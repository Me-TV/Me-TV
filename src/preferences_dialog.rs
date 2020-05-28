/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018–2020  Russel Winder
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

use lazy_static::lazy_static;

use gtk;
use gtk::prelude::*;

use crate::control_window::ControlWindow;
use crate::dvb;
use crate::metvcomboboxtext::MeTVComboBoxText;
use crate::metvcomboboxtext::MeTVComboBoxTextExt;
use crate::preferences;

lazy_static! {
    static ref PREFERENCES: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
}

fn create(control_window: &ControlWindow) -> gtk::Window {
    let menu_builder = gtk::Builder::new_from_string(include_str!("resources/preferences_dialog.glade.xml"));
    let _delivery_system_comboboxtext = {
        let comboboxtext = menu_builder.get_object::<gtk::ComboBoxText>("delivery_system").unwrap();
        for delivery_system in dvb::DeliverySystem::iterator() {
            comboboxtext.append_text(&delivery_system.to_string());
        }
        comboboxtext.set_active(Some(preferences::get_delivery_system().get_index()));
        comboboxtext.connect_changed(
            move |selector| preferences::set_delivery_system(selector.get_active_text().unwrap().as_str().into(), true)
        );
        comboboxtext
    };
    let _use_opengl_button = {
        let button = menu_builder.get_object::<gtk::CheckButton>("use_opengl").unwrap();
        button.set_active(preferences::get_use_opengl());
        button.connect_toggled(
            move |b| preferences::set_use_opengl(b.get_active(), true)
        );
        button
    };
    let _immediate_tv_button = {
        let button = menu_builder.get_object::<gtk::CheckButton>("immediate_tv").unwrap();
        button.set_active(preferences::get_immediate_tv());
        button.connect_toggled(
            move |b| preferences::set_immediate_tv(b.get_active(), true)
        );
        button
    };
    let  use_last_channel_button = menu_builder.get_object::<gtk::RadioButton>("last_channel").unwrap();
    let  use_default_channel_button = {
        let button = menu_builder.get_object::<gtk::RadioButton>("default_channel").unwrap();
        button.join_group(Some(&use_last_channel_button));
        button
    };
    if preferences::get_use_last_channel() { use_last_channel_button.set_active(true); }
    else { use_default_channel_button.set_active(true); }
    use_last_channel_button.connect_clicked(
        move |_| preferences::set_use_last_channel(true, true)
    );
    use_default_channel_button.connect_clicked(
        move |_| preferences::set_use_last_channel(false, true)
    );
    let _default_channel_selector = {
        let mut combobox = menu_builder.get_object::<MeTVComboBoxText>("channel_name").unwrap();
        combobox.set_new_model(&control_window.channel_names_store);
        if let Some(channel_name) = preferences::get_default_channel() {
            if channel_name != "" {
                if ! combobox.set_active_text(channel_name.clone()) {
                    panic!("Could not set the default channel to {}.", channel_name);
                }
            }
        }
        combobox.connect_changed(
            move |selector: &MeTVComboBoxText| preferences::set_default_channel(selector.get_active_text().unwrap(), true)
        );
        combobox
    };
    let _nongl_deinterlace_method_selector = {
        let comboboxtext = menu_builder.get_object::<gtk::ComboBoxText>("nongl_deinterlace_method").unwrap();
        if let Some(method) = preferences::get_nongl_deinterlace_method() {
            if method != "" {
                if ! comboboxtext.set_active_id(Some(&method)) {
                    panic!("Could not set the Non-GL deinterlacing method.");
                }
            }
        }
        comboboxtext.connect_changed(
            move |selector| preferences::set_nongl_deinterlace_method(selector.get_active_id().unwrap().as_str().into(), true)
        );
        comboboxtext
    };
    let _gl_deinterlace_method_selector = {
        let comboboxtext = menu_builder.get_object::<gtk::ComboBoxText>("gl_deinterlace_method").unwrap();
        if let Some(method) = preferences::get_gl_deinterlace_method() {
            if method != "" {
                if ! comboboxtext.set_active_id(Some(&method)) {
                    panic!("Could not set the Non-GL deinterlacing method: {}", &method);
                }
            }
        }
        comboboxtext.connect_changed(
            move |selector| preferences::set_gl_deinterlace_method(selector.get_active_id().unwrap().as_str().into(), true)
        );
        comboboxtext
    };
    let preferences_dialog = {
        let window = menu_builder.get_object::<gtk::Window>("preferences_dialog").unwrap();
        window.set_transient_for(Some(&control_window.window));
        window.show_all();
        window
    };
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
