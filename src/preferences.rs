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

use std::cell::RefCell;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

use serde_yaml ;
use xdg;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Preferences {
    use_opengl: bool,
    immediate_tv: bool,
    default_channel: String,
}

lazy_static! {
static ref PREFERENCES: Mutex<RefCell<Preferences>> = Mutex::new(RefCell::new(Preferences{
    use_opengl: true,
    immediate_tv: false,
    default_channel: String::from(""),
}));
}

/// Return a `PathBuf` to the Me TV preferences file location.
/// Currently use a YAML file to store the preferences rather than
/// getting involved with the DConf
fn get_preferences_file_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("me-tv").expect("Cannot set XDG prefix.");
    let mut path_buf = xdg_dirs.get_config_home();
    path_buf.push("preferences.yml");
    path_buf
}

/// Write the current `Preferences` instance, serialised to YAML, to the preferences
/// file location overwriting whatever was there.
fn write_preferences() {
    match OpenOptions::new().write(true).truncate(true).create(true).open(get_preferences_file_path()) {
        Ok(mut file) => {
            if let Ok(preferences) = PREFERENCES.lock() {
                let buffer = serde_yaml::to_string(&*preferences.borrow()).unwrap();
                file.write(&buffer.into_bytes()).unwrap();
                file.flush().unwrap();
            }
        },
        Err(error) => panic!("Cannot create or open {:?} {:?}", get_preferences_file_path(), error),
    }
}

/// Initialise the preferences system. Ensures the XDG config directory exists then
/// reads the preferences file if it exists and swaps the deserialized `Preferences`
/// instance with the hard-coded default.
pub fn init() {
    let path = get_preferences_file_path();
    if let Err(error) = create_dir_all(path.parent().unwrap()) {
        panic!("create_dir_all({:?}) failed: {:?}", path.parent().unwrap(), error);
    }
    if path.is_file() {
        if let Ok(mut file) = File::open(path) {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            match serde_yaml::from_str(&buffer) {
                Ok(new_preferences) => if let Ok(preferences) = PREFERENCES.lock() {
                    preferences.replace(new_preferences);
                },
                Err(_) => {
                    // TODO Missing field. Should not just assume default, need to carry
                    // forward the options that could be picked up
                }
            }
        }
    }
}

/// Getter for the current state of the `use_opengl` preference.
pub fn get_use_opengl() -> bool {
    match PREFERENCES.lock() {
        Ok(preferences) => preferences.borrow().use_opengl,
        Err(_) => true,
    }
}

/// Setter for the `use_opengl` preference. If `write_back` is true the
/// current `Preferences` instance  is written to file.
pub fn set_use_opengl(use_opengl: bool, write_back: bool) {
    if let Ok(preferences) = PREFERENCES.lock() {
        let mut new_preferences = preferences.borrow().clone();
        new_preferences.use_opengl = use_opengl;
        preferences.replace(new_preferences);
    }
    if write_back {
        write_preferences();
    }
}

/// Getter for the current state of the `immediate_tv` preference.
pub fn get_immediate_tv() -> bool {
    match PREFERENCES.lock() {
        Ok(preferences) => preferences.borrow().immediate_tv,
        Err(_) => true,
    }
}

/// Setter for the `immediate_tv` preference. If `write_back` is true the
/// current `Preferences` instance  is written to file.
pub fn set_immediate_tv(immediate_tv: bool, write_back: bool) {
    if let Ok(preferences) = PREFERENCES.lock() {
        let mut new_preferences = preferences.borrow().clone();
        new_preferences.immediate_tv = immediate_tv;
        preferences.replace(new_preferences);
    }
    if write_back {
        write_preferences();
    }
}

/// Getter for the current state of the `default_channel` preference.
pub fn get_default_channel() -> Option<String> {
    match PREFERENCES.lock() {
        Ok(preferences) => Some(preferences.borrow().default_channel.clone()),
        Err(_) => None,
    }
}

/// Setter for the `default_channel` preference. If `write_back` is true the
/// current `Preferences` instance  is written to file.
pub fn set_default_channel(default_channel: String, write_back: bool) {
    if let Ok(preferences) = PREFERENCES.lock() {
        let mut new_preferences = preferences.borrow().clone();
        new_preferences.default_channel = default_channel;
        preferences.replace(new_preferences);
    }
    if write_back {
        write_preferences();
    }
}
