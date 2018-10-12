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

// TODO Explain why a ComboBoxText cannot be used?

/// Me TV would have problems using ComboBoxText so use a ComboBox but in
/// the knowledge it is always a single column of strings.
pub type MeTVComboBoxText = gtk::ComboBox;

pub trait MeTVComboBoxTextExt {
    fn new_with_core_model(model: &gtk::ListStore) -> MeTVComboBoxText;
    fn set_with_core_model(&mut self, model: &gtk::ListStore);
    fn get_active_text(&self) -> Option<String>;
}

impl MeTVComboBoxTextExt  for MeTVComboBoxText {

    fn new_with_core_model(model: &gtk::ListStore) -> MeTVComboBoxText {
        let mut combobox = gtk::ComboBox::new();
        combobox.set_with_core_model(model);
        combobox
    }

    fn set_with_core_model(&mut self, model: &gtk::ListStore) {
        self.set_model(model);
        let renderer = gtk::CellRendererText::new();
        self.pack_start(&renderer, true);
        self.add_attribute(&renderer, "text", 0);
    }

    fn get_active_text(&self) -> Option<String> {
        match self.get_model() {
            Some(model) => {
                match self.get_active_iter() {
                    Some(iterator) => {
                        let x = model.get_value(&iterator, 0).get::<String>().unwrap();
                        Some(x)
                    },
                    None => None,
                }
            },
            None => None,
        }
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_test_model() -> gtk::ListStore {
        let store = gtk::ListStore::new(&[String::static_type()]);
        store.insert_with_values(None, &[0], &[&"fred"]);
        store.insert_with_values(None, &[0], &[&"jane"]);
        store.insert_with_values(None, &[0], &[&"jo"]);
        store.insert_with_values(None, &[0], &[&"bert"]);
        store
    }

    #[test]
    fn get_active_text() {
        gtk::init().unwrap();
        let store = gtk::ListStore::new(&[String::static_type()]);
        let thingy = MeTVComboBoxText::new();
        thingy.set_model(&store);
        thingy.set_active(1); // TODO Should this fail in some way?
        assert_eq!(thingy.get_active_text(), None);

        let store = create_test_model();
        thingy.set_model(&store);
        thingy.set_active(0);
        assert_eq!(thingy.get_active_text().unwrap(), "fred");

        thingy.set_active(2);
        assert_eq!(thingy.get_active_text().unwrap(), "jo");

        thingy.set_active(1);
        assert_eq!(thingy.get_active_text().unwrap(), "jane");
    }

}
