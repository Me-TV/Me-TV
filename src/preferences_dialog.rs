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

use preferences;

lazy_static! {
    static ref PREFERENCES: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
}

fn create(parent: Option<&gtk::ApplicationWindow>) -> gtk::Dialog {
    let dialog = gtk::Dialog::new_with_buttons(
        Some("Me TV Preferences"),
        parent,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        &[],
    );
    let content_area = dialog.get_content_area();
    let use_opengl_button = gtk::CheckButton::new_with_label("Use OpenGL if possible");
    use_opengl_button.set_active(preferences::get_use_opengl());
    use_opengl_button.connect_toggled(
        move |button| preferences::set_use_opengl(button.get_active(), true)
    );
    content_area.pack_start(&use_opengl_button, false, false, 10);
    dialog.show_all();
    dialog
}

pub fn present(parent: Option<&gtk::ApplicationWindow>) {
    if let Ok(active) = PREFERENCES.lock() {
        if ! active.get() {
            let dialog = create(parent);
            dialog.connect_response(move |_, _| {
                if let Ok(active) = PREFERENCES.lock() {
                    active.set(false);
                }
            });
            dialog.move_(0, 0);
            dialog.show();
            active.set(true);
        }
    }
}
