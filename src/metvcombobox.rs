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

use gtk;
use gtk::prelude::*;

/// An application specific version of a `ComboBox`.
pub type MeTVComboBox = gtk::ComboBox;

pub trait MeTVComboBoxExt {
    // Can't use any names from `gtk::ComboBox`
    fn new_with_model<T: IsA<gtk::TreeModel>>(model: &T) -> MeTVComboBox;
    fn init_with_model<T: IsA<gtk::TreeModel>>(&mut self, model: &T);
    fn get_active_text(&self) -> Option<String>;
    fn set_active_text(&mut self, name: String) -> bool;
}

impl MeTVComboBoxExt for MeTVComboBox {

    /// Create a new `MeTVComboBox` and set the data model.
    ///
    /// It is assumed that the `TreeModel` is actually a `ListStore` or a
    /// `TreeModelSort` backed by a `ListStore` with the `ListStore` having
    /// two columns (`String`, `String`) being the channel number and
    /// the channel name.
    fn new_with_model<T: IsA<gtk::TreeModel>>(model: &T) -> MeTVComboBox {
        let mut combobox = gtk::ComboBox::new();
        combobox.init_with_model(model);
        combobox
    }

    /// Initialise and set the data model of a `MeTVComboBox`.
    ///
    /// It is assumed that the `TreeModel` is actually a `ListStore` or a
    /// `TreeModelSort` backed by a `ListStore` with the `ListStore` having
    /// two columns (`String`, `String`) being the channel number and
    /// the channel name.
    fn init_with_model<T: IsA<gtk::TreeModel>>(&mut self, model: &T) {
        self.set_model(Some(model));
        let number_renderer = gtk::CellRendererText::new();
        self.pack_start(&number_renderer, true);
        self.add_attribute(&number_renderer, "text", 0);
        let name_renderer = gtk::CellRendererText::new();
        self.pack_start(&name_renderer, true);
        self.add_attribute(&name_renderer, "text", 1);
   }

    fn get_active_text(&self) -> Option<String> {
        match self.get_model() {
            Some(model) => {
                match self.get_active_iter() {
                    Some(iterator) => {
                        let x = model.get_value(&iterator, 1).get::<String>().unwrap().unwrap();
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
                            if let Some(name) = model.get_value(&iterator, 1).get::<String>().unwrap() {
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
        let store = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        store.insert_with_values(None, &[0, 1], &[&4.to_string(), &"fred"]);
        store.insert_with_values(None, &[0, 1], &[&3.to_string(), &"jane"]);
        store.insert_with_values(None, &[0, 1], &[&2.to_string(), &"jo"]);
        store.insert_with_values(None, &[0, 1], &[&1.to_string(), &"bert"]);
        store
    }

    #[test]
    fn various_tests() {
        match gtk::init() {
            Ok(_) => (),
            Err(_) => panic!("Could not initialise GTK"),
        }
        let store = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        let mut thingy = MeTVComboBox::new_with_model(&store);
        thingy.set_active(Some(1)); // TODO Should this fail in some way?
        assert_eq!(thingy.get_active_text(), None);

        let store = create_test_model();
        thingy.init_with_model(&store);
        thingy.set_active(Some(0));
        assert_eq!(thingy.get_active_text().unwrap(), "fred");

        thingy.set_active(Some(2));
        assert_eq!(thingy.get_active_text().unwrap(), "jo");

        thingy.set_active(Some(1));
        assert_eq!(thingy.get_active_text().unwrap(), "jane");

        let mut another_thingy = MeTVComboBox::new_with_model(&store);

        let target = "jo".to_string();
        assert_eq!(another_thingy.set_active_text(target.clone()), true);
        assert_eq!(another_thingy.get_active_text().unwrap(), target);

        let target = "jane".to_string();
        assert_eq!(another_thingy.set_active_text(target.clone()), true);
        assert_eq!(another_thingy.get_active_text().unwrap(), target);
    }

}
