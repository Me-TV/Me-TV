/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017  Russel Winder
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

// gtkmm has a really useful function set_active_text on Gtk::ComboBoxText
// that is not a function in Gtk+, but an extra in gtkmm. Replicate this for gtk-rs.

pub trait ComboBoxTextExtras {
    fn set_active_text(&self, text: &str);
}

impl ComboBoxTextExtras for gtk::ComboBoxText {
    fn set_active_text(&self, text: &str) {
        if let Some(model) = self.get_model() {
            if let Some(iterator) = model.get_iter_first() {
                loop {
                    if model.get_value(&iterator, 0).get::<String>().unwrap() == text {
                        self.set_active_iter(Some(&iterator));
                        return;
                    }
                    if ! model.iter_next(&iterator) {
                        self.set_active_iter(None);
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_active_using_string() {
        gtk::init().unwrap();
        let thingy = gtk::ComboBoxText::new();
        thingy.append_text("fred");
        thingy.append_text("jane");
        thingy.set_active_text("jane");
        assert_eq!(thingy.get_active_text().unwrap(), "jane");
    }

}