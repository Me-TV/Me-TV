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

use std::cell::RefCell;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use serde_yaml ;
use xdg;

use crate::dvb;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Preferences {
    delivery_system: dvb::DeliverySystem,
    use_opengl: bool,
    immediate_tv: bool,
    use_last_channel: bool,
    default_channel: String,
    last_channel: String,
    nongl_deinterlace_method: String,
    gl_deinterlace_method: String,
}

// TODO Replace the Mutex with a RwLock.
lazy_static! {
    static ref PREFERENCES: Mutex<RefCell<Preferences>> = Mutex::new(RefCell::new(Preferences{
        delivery_system: dvb::DeliverySystem::DVBT,
        use_opengl: true,
        immediate_tv: false,
        use_last_channel: false,
        default_channel: String::from(""),
        last_channel: String::from(""),
        nongl_deinterlace_method: "".to_string(),
        gl_deinterlace_method: "".to_string(),
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
                    //   forward the options that could be picked up
                }
            }
        }
    }
}

macro_rules! create_getter {
    ($function_name:ident, $field_name:ident, $return_type:ty, $default_value:expr) => {
        pub fn $function_name() -> $return_type {
            match PREFERENCES.lock() {
               Ok(preferences) => preferences.borrow().$field_name,
               Err(_) => $default_value,
            }
        }
    }
}

macro_rules! create_option_getter {
    ($function_name:ident, $field_name:ident, $return_type:ty, $default_value:expr) => {
        pub fn $function_name() -> Option<$return_type> {
            match PREFERENCES.lock() {
               Ok(preferences) => Some(preferences.borrow().$field_name.clone()),
               Err(_) => $default_value,
            }
        }
    }
}

macro_rules! create_setter {
    ($function_name:ident, $field_name:ident, $parameter_type:ty) => {
        pub fn $function_name(new_value: $parameter_type, write_back: bool) {
            if let Ok(preferences) = PREFERENCES.lock() {
                let mut new_preferences = preferences.borrow().clone();
                new_preferences.$field_name = new_value;
                preferences.replace(new_preferences);
            }
            if write_back { write_preferences(); }
        }
    }
}

pub fn get_delivery_system() -> dvb::DeliverySystem {
    match PREFERENCES.lock() {
        Ok(preferences) => preferences.borrow().delivery_system.clone(),  //  Must clone here so can't use the macro.
        Err(_) => dvb::DeliverySystem::DVBT,
    }
}
create_setter!(set_delivery_system, delivery_system, dvb::DeliverySystem);

create_getter!(get_use_opengl, use_opengl, bool, true);
create_setter!(set_use_opengl, use_opengl, bool);

create_getter!(get_immediate_tv, immediate_tv, bool, false);
create_setter!(set_immediate_tv, immediate_tv, bool);

create_getter!(get_use_last_channel, use_last_channel, bool, false);
create_setter!(set_use_last_channel, use_last_channel, bool);

create_option_getter!(get_default_channel, default_channel, String, None);
create_setter!(set_default_channel, default_channel, String);

create_option_getter!(get_last_channel, last_channel, String, None);
create_setter!(set_last_channel, last_channel, String);

create_option_getter!(get_nongl_deinterlace_method, nongl_deinterlace_method, String, None);
create_setter!(set_nongl_deinterlace_method, nongl_deinterlace_method, String);

create_option_getter!(get_gl_deinterlace_method, gl_deinterlace_method, String, None);
create_setter!(set_gl_deinterlace_method, gl_deinterlace_method, String);
