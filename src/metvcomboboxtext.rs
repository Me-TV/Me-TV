/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018, 2019  Russel Winder
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

/// An application specific version of a `ComboBoxText`.
///
/// `ComboBoxText` does not seem to allow for using a ready-made model.
/// Since the idea in this application is to have many rendering of the same model,
/// the list of available channels, `ComboBox` has to be used. However make it as
/// much like a ComboBoxText as possible by providing this abstraction.
pub type MeTVComboBoxText = gtk::ComboBox;

pub trait MeTVComboBoxTextExt {
    fn new_and_set_model(model: &gtk::ListStore) -> MeTVComboBoxText;
    fn set_new_model(&mut self, model: &gtk::ListStore);
    fn get_active_text(&self) -> Option<String>;
    fn set_active_text(&mut self, name: String) -> bool;
}

impl MeTVComboBoxTextExt for MeTVComboBoxText {

    fn new_and_set_model(model: &gtk::ListStore) -> MeTVComboBoxText {
        let mut combobox = gtk::ComboBox::new();
        combobox.set_new_model(model);
        combobox
    }

    fn set_new_model(&mut self, model: &gtk::ListStore) {
        self.set_model(Some(model));
        let renderer = gtk::CellRendererText::new();
        self.pack_start(&renderer, true);
        self.add_attribute(&renderer, "text", 0);
    }

    fn get_active_text(&self) -> Option<String> {
        match self.get_model() {
            Some(model) => {
                match self.get_active_iter() {
                    Some(iterator) => {
                        let x = model.get_value(&iterator, 0).get::<String>().unwrap().unwrap();
                        Some(x)
                    },
                    None => None,
                }
            },
            None => panic!("Could not get the model."),
        }
    }

    fn set_active_text(&mut self, target_name: String) -> bool {
        match self.get_model() {
            Some(model) => {
                match model.get_iter_first() {
                    Some(iterator) => {
                        loop {
                            if let Some(name) = model.get_value(&iterator, 0).get::<String>().unwrap() {
                                if target_name == name {
                                    self.set_active_iter(Some(&iterator));
                                    return true;
                                }
                            } else {
                                break
                            }
                            if ! model.iter_next(&iterator) { break }
                        };
                        false
                    },
                    None => panic!("Could not get an iterator."),
                }
            },
            None => panic!("Could not get the model.")
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
    fn various_tests() {
        match gtk::init() {
            Ok(_) => (),
            Err(_) => panic!("Could not initialise GTK"),
        }
        let store = gtk::ListStore::new(&[String::static_type()]);
        let mut thingy = MeTVComboBoxText::new_and_set_model(&store);
        thingy.set_active(Some(1)); // TODO Should this fail in some way?
        assert_eq!(thingy.get_active_text(), None);

        let store = create_test_model();
        thingy.set_new_model(&store);
        thingy.set_active(Some(0));
        assert_eq!(thingy.get_active_text().unwrap(), "fred");

        thingy.set_active(Some(2));
        assert_eq!(thingy.get_active_text().unwrap(), "jo");

        thingy.set_active(Some(1));
        assert_eq!(thingy.get_active_text().unwrap(), "jane");

        let mut another_thingy = MeTVComboBoxText::new_and_set_model(&store);

        let target = "jo".to_string();
        assert_eq!(another_thingy.set_active_text(target.clone()), true);
        assert_eq!(another_thingy.get_active_text().unwrap(), target);

        let target = "jane".to_string();
        assert_eq!(another_thingy.set_active_text(target.clone()), true);
        assert_eq!(another_thingy.get_active_text().unwrap(), target);
    }

}
