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

use gtk;
use gtk::prelude::*;

use preferences;

fn create(parent: Option<&gtk::ApplicationWindow>) -> gtk::Dialog {
    let dialog = gtk::Dialog::new_with_buttons(
        Some("Me TV Preferences"),
        parent,
        gtk::DialogFlags::MODAL,
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
    let dialog = create(parent);
    dialog.run();
    dialog.destroy();
}
