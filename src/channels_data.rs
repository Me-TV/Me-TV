/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017–2020  Russel Winder
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

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::RwLock;

use ini;
use lazy_static::lazy_static;
use percent_encoding;
use serde_derive::{Serialize, Deserialize};
use serde_yaml;
use xdg;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
/// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH: &percent_encoding::AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');

/// Encode a string as used for display to one suitable to be an MRL.
pub fn encode_to_mrl(channel_name: &String) -> String {
    "dvb://".to_owned() + &percent_encoding::utf8_percent_encode(channel_name, PATH).to_string()
}

/// Struct for the data of each channel stored for various lookups.
///
/// It is assumed that instances are the data pointed to by various indexes so as to
/// create lookups between for example logical_channel_number and name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelData {
    name: String,
    service_id: u16,
    logical_channel_number: u16, // Channel 0 is not used so 0 can be used as "not yet known".
}

// A singleton of the channels data currently known.
//
// This is initialised from the GStreamer channels data file, then augmented from the
// Me TV data cache file, and then updated as `LogicalChannelDescriptor` are received.
// The data is written to the cache file as and when.
//
// TODO need to update the ListStore in the ControlWindow instance
//   whenever a change is made here.
//
lazy_static! {
    static ref CHANNELS_DATA: RwLock<Option<Vec<ChannelData>>> = RwLock::new(initialise_channels_data());
}

/// Construct the value to be used to initialise `CHANNELS_DATA`.
///
/// First read the data from the GStreamer channels data file (if it exists) and then
/// augment using the Me TV data cache file (if it exists).
fn initialise_channels_data() -> Option<Vec<ChannelData>> {
    match ini::Ini::load_from_file(channels_file_path()) {
        Ok(ini) => {
            let mut channel_data = process_ini(&ini);
            if let Some(cache) = read_channels_data_cache(&channels_data_cache_path()) {
                let table = cache
                    .iter()
                    .map(|x|(x.service_id, x.logical_channel_number))
                    .collect::<HashMap<u16, u16>>();
                channel_data = channel_data
                    .iter()
                    .map(|x| if x.logical_channel_number == 0 {
                        ChannelData {
                            name: x.name.clone(),
                            service_id: x.service_id,
                            logical_channel_number: *table.get(&x.service_id).unwrap(),
                        }
                    } else {
                        x.clone()
                    })
                    .collect();
            }
            Some(channel_data)
        },
        Err(_) => None,
    }
}

/// Process an `Ini` to create a `Vec<ChannelData>`
fn process_ini(ini: &ini::Ini) -> Vec<ChannelData> {
    ini.iter()
        .map(|(name, properties)| ChannelData{
            name: name.unwrap().to_string(),
            service_id: properties.get("SERVICE_ID").unwrap().parse::<u16>().unwrap(),
            logical_channel_number: 0,
        })
        .collect()
}

/// Read channels data from the channels file, if it exists, augmented by the cache data, if it exists.
///
/// Return Boolean specifies whether the data was set to `Some`thing (`true`) or `None` (`false`).
pub fn read_channels_data() -> bool { // Used in control_window.rs after a channel search from the UI.
    match initialise_channels_data() {
        Some(data) => {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
            true
        },
        None => {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = None;
            false
        },
    }
}

/// Return a `Vec` containing the names of the channels from the channels data.
fn get_names_from_channels_data(channels_data: &Vec<ChannelData>) -> Vec<String> {
    channels_data.iter().map(|x| x.name.clone() ).collect()
}

/// Return a `Vec` containing the (logical number, name) pairs of the channels from the channels data.
fn get_numbers_and_names_from_channels_data(channels_data: &Vec<ChannelData>) -> Vec<(u16, String)> {
    channels_data.iter().map(|x| (x.logical_channel_number, x.name.clone()) ).collect()
}

/// Return a `Box<Path>` to the GStreamer dvbsrc plugin channels file using the XDG directory structure.
pub fn channels_file_path() -> Box<Path> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("gstreamer-1.0").expect("Cannot set XDG prefix.");
    let mut path_buf = xdg_dirs.get_config_home();
    path_buf.push("dvb-channels.conf");
    path_buf.into_boxed_path()
}

/// Return a `Box<Path>` to the Me TV channels data cache file using the XDG directory structure.
pub fn channels_data_cache_path() -> Box<Path> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("me-tv").expect("Cannot set XDG prefix.");
    let mut path_buf = xdg_dirs.get_cache_home();
    path_buf.push("channels_data.yml");
    path_buf.into_boxed_path()
}

/// Returns the names of the channels from the GStreamer channels file.
///
/// GStreamer uses the XDG directory structure with, currently, gstreamer-1.0 as its
/// name. The dvbsrc plugin assumes the name dvb-channels.conf. The DVBv5 file format
/// is INI style: a sequence of blocks, one for each channel, starting with a channel
/// name surrounded by brackets and then a sequence of binding of keys to values each
/// one indented.
pub fn get_channel_names() -> Option<Vec<String>> {
    let channels_data = CHANNELS_DATA.read().unwrap();
    match &*channels_data {
        Some(c_d) => Some(get_names_from_channels_data(c_d)),
        None => None,
    }
}

/// Return a `Vec<(u16, String)>` where the `(u16, String)` is the logical number
/// and name of a named channel as found in the GStreamer channels file.
///
/// GStreamer uses the XDG directory structure with, currently, gstreamer-1.0 as its
/// name. The dvbsrc plugin assumes the name dvb-channels.conf. The DVBv5 file format
/// is INI style: a sequence of blocks, one for each channel, starting with a channel
/// name surrounded by brackets and then a sequence of binding of keys to values each
/// one indented.
///
/// Logical channel numbers are found by searching the SI packets and caching them.
pub fn get_channels_data() -> Option<Vec<(u16, String)>> {
    let channels_data = CHANNELS_DATA.read().unwrap();
    match &*channels_data {
        Some(c_d) => Some(get_numbers_and_names_from_channels_data(c_d)),
        None => None,
    }
}

/// Update the channels file data.
///
/// For use when getting SI packets that build the Logical Channel Table.
///
/// Return `true` if a change was made to the channels data, `false` otherwise.
pub fn add_logical_channel_number_for_service_id(service_id: u16, logical_channel_number: u16) -> bool {
    // TODO This does a full (albeit shallow) copy of the data structure, should a more
    //   efficient way of doing the update be found?
    //   Freeview from Crystal Palace has a maximum 182 channels as at 2020-07-07.
    let mut channels_data = CHANNELS_DATA.write().unwrap();
    match &*channels_data {
        Some(c_d) => {
            let mut rv = false;
            *channels_data = Some(
                c_d
                    .iter()
                    .map(|x| {
                        if x.service_id == service_id && x.logical_channel_number != logical_channel_number {
                            rv = true;
                            ChannelData {
                                name: x.name.clone(),
                                service_id: x.service_id,
                                logical_channel_number,
                            }
                        } else {
                            x.clone()
                        }
                    })
                    .collect()
            );
            rv
        },
        None => false,
    }
}

/// Return the channel name for a given channel number.
///
/// Return is actually an `Option`, `None` is returned if the logical_channel_number was
/// not found in the channel data.
pub fn get_channel_name_of_logical_channel_number(logical_channel_number: u16) -> Option<String> {
    let channel_data = CHANNELS_DATA.read().unwrap();
    match &*channel_data {
        Some(c_d) => {
            // TODO Can we do better than linear search, or does it not matter?
            //    Freeview from Crystal Palace has a maximum 182 channels as at 2020-07-07.
            let result: Vec<&ChannelData> = c_d.iter().filter(|x| x.logical_channel_number == logical_channel_number).collect();
            match result.len() {
                0 => None,
                1 => Some(result[0].name.clone()),
                _ => panic!("Got more than one channel with the same logical number."),
            }
        },
        None => None,
    }
}

/// Write the channels data to a cache file.
fn write_channels_data_cache(path: &Path) {
    match OpenOptions::new().write(true).open(path) {
        Ok(mut f) => {
            let channels_data_ptr = CHANNELS_DATA.read().unwrap();
            let channels_data: &Vec<ChannelData> = (*channels_data_ptr).as_ref().unwrap();
            let s = serde_yaml::to_string(&channels_data).unwrap();
            match f.write(s.as_ref()) {
                Ok(count) => {
                    assert_eq!(count, s.len());
                    f.flush().unwrap();
                },
                Err(e) => println!("Error writing {:?}, {:?} – {}", path.to_str().unwrap(), f, e),
            };
        },
        Err(e) => println!("Failed to open {} – {}", path.to_str().unwrap(), e),
    };
}

/// Read the channels data given a path and return the result.
fn read_channels_data_cache(path: &Path) -> Option<Vec<ChannelData>> {
    match File::open(path) {
        Ok(mut f) => {
            let mut buffer = [0u8; 20000];
            match f.read(&mut buffer) {
                Ok(count) => {
                    let s = String::from_utf8_lossy(&buffer[..count]).to_string();
                    match serde_yaml::from_str::<Vec<ChannelData>>(&s) {
                        Ok(x) => Some(x),
                        Err(e) => {
                            println!("Failed to deserialise {} – {}", path.to_str().unwrap(), e);
                            None
                        },
                    }
                },
                Err(e) => {
                    println!("Failed to read {} – {}", path.to_str().unwrap(), e);
                    None
                },
            }
        },
        Err(e) => {
            println!("Failed to open {} – {}", path.to_str().unwrap(), e);
            None
        },
    }
}

#[cfg(test)]
mod tests {

    use std::io::Read;
    use std::sync::Mutex;

    use ini;
    use lazy_static::lazy_static;
    use tempfile;

    use super::{
        add_logical_channel_number_for_service_id,
        channels_file_path,
        encode_to_mrl, process_ini,
        get_names_from_channels_data,
        get_numbers_and_names_from_channels_data,
        get_channel_name_of_logical_channel_number,
        read_channels_data,
        write_channels_data_cache,
        read_channels_data_cache,
        ChannelData, CHANNELS_DATA
    };

    #[test]
    fn encode_to_mrl_with_no_spaces() {
        assert_eq!(encode_to_mrl(&"ITV".to_owned()), "dvb://ITV");
    }

    #[test]
    fn encode_to_mrl_with_one_space() {
        assert_eq!(encode_to_mrl(&"BBC NEWS".to_owned()), "dvb://BBC%20NEWS");
    }

    #[test]
    fn encode_to_mrl_with_two_spaces() {
        assert_eq!(encode_to_mrl(&"BBC One Lon".to_owned()), "dvb://BBC%20One%20Lon");
    }

    #[test]
    fn encode_to_mrl_with_hash() {
        assert_eq!(encode_to_mrl(&"Channel #1".to_owned()), "dvb://Channel%20%231");
    }

    #[test]
    fn get_names_from_empty_channel_data_vec() {
        let empty_input = vec![];
        let empty_output: Vec<String> = vec![];
        assert_eq!(get_names_from_channels_data(&empty_input), empty_output);
    }

    #[test]
    fn get_numbers_and_names_from_empty_channel_data_vec() {
        let empty_input = vec![];
        let empty_output: Vec<(u16, String)> = vec![];
        assert_eq!(get_numbers_and_names_from_channels_data(&empty_input), empty_output);
    }

    fn create_two_entry_channel_data_vec() -> Vec<ChannelData> {
               let data = "
[BBC ONE Lon]
        SERVICE_ID = 4164
        NETWORK_ID = 9018
        TRANSPORT_ID = 4164
        VIDEO_PID = 101
        AUDIO_PID = 102 106
        PID_0b = 7219 7201
        PID_06 = 152 105
        PID_05 = 7105 7103
        FREQUENCY = 490000000
        MODULATION = QAM/64
        BANDWIDTH_HZ = 8000000
        INVERSION = AUTO
        CODE_RATE_HP = 2/3
        CODE_RATE_LP = AUTO
        GUARD_INTERVAL = 1/32
        TRANSMISSION_MODE = 8K
        HIERARCHY = NONE
        DELIVERY_SYSTEM = DVBT

[BBC TWO]
        SERVICE_ID = 4287
        NETWORK_ID = 9018
        TRANSPORT_ID = 4164
        VIDEO_PID = 201
        AUDIO_PID = 202 206
        PID_0b = 7219 7201
        PID_06 = 205
        PID_05 = 7105 7103
        FREQUENCY = 490000000
        MODULATION = QAM/64
        BANDWIDTH_HZ = 8000000
        INVERSION = AUTO
        CODE_RATE_HP = 2/3
        CODE_RATE_LP = AUTO
        GUARD_INTERVAL = 1/32
        TRANSMISSION_MODE = 8K
        HIERARCHY = NONE
        DELIVERY_SYSTEM = DVBT
";
        let ini = ini::Ini::load_from_str(data).unwrap();
        process_ini(&ini)
    }

    #[test]
    fn process_ini_with_two_entries() {
        let result = create_two_entry_channel_data_vec();
        assert_eq!(result.len(), 2);
        let bbc_1 = &result[0];
        assert_eq!(bbc_1.name,  "BBC ONE Lon");
        assert_eq!(bbc_1.service_id,  4164);
        assert_eq!(bbc_1.logical_channel_number,  0);
        let bbc_2 = &result[1];
        assert_eq!(bbc_2.name,  "BBC TWO");
        assert_eq!(bbc_2.service_id,  4287);
        assert_eq!(bbc_2.logical_channel_number,  0);
    }

    // Tests need to be able to set specific values to CHANNELS_DATA rather than just
    // load the files. Although access to CHANNELS_DATA is controlled, there is an
    // assumption the value is that of reading the files. By default, tests are run
    // multi-threaded, but this is only a useful test if the value of CHANNEL_DATA is
    // not test dependent. Single threaded test execution of these tests must be enforced.
    // Rather than get the developer to remember to use "cargo test -- --test-threads=1"
    // which has all tests run single threaded, we use this Mutex so that only tests that
    // must be single threaded are single-threaded.

    lazy_static! {
        static ref TEST_LOCK: Mutex<bool> = Mutex::new(false);
    }

    #[test]
    fn process_channels_data_file() {
        let test_lock = TEST_LOCK.lock().unwrap();
        let read_file = read_channels_data();
        let channels_data = CHANNELS_DATA.read().unwrap();
        match &*channels_data {
            Some(c_d) => if read_file {
                assert_ne!(c_d.len(), 0);
            } else {
                assert_eq!(c_d.len(), 0);
            },
            None => {},
        };
    }

    #[test]
    fn update_channels_data() {
        let test_lock = TEST_LOCK.lock().unwrap();
        let data = create_two_entry_channel_data_vec();
        {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
        }
        {
            let channels_data = CHANNELS_DATA.read().unwrap();
            let data: &Vec<ChannelData> = (*channels_data).as_ref().unwrap();
            let bbc_1 = &data[0];
            assert_eq!(bbc_1.name, "BBC ONE Lon");
            assert_eq!(bbc_1.service_id, 4164);
            assert_eq!(bbc_1.logical_channel_number, 0);
            let bbc_2 = &data[1];
            assert_eq!(bbc_2.name, "BBC TWO");
            assert_eq!(bbc_2.service_id, 4287);
            assert_eq!(bbc_2.logical_channel_number, 0);
        }
        let rc = add_logical_channel_number_for_service_id(4164, 1);
        assert!(rc);
        let rc = add_logical_channel_number_for_service_id(4287, 2);
        assert!(rc);
        let rc = add_logical_channel_number_for_service_id(3000, 76);
        assert!(!rc);
        let channel_data = CHANNELS_DATA.read().unwrap();
        let data: &Vec<ChannelData> = (*channel_data).as_ref().unwrap();
        let bbc_1 = &data[0];
        assert_eq!(bbc_1.name, "BBC ONE Lon");
        assert_eq!(bbc_1.service_id, 4164);
        assert_eq!(bbc_1.logical_channel_number, 1);
        let bbc_2 = &data[1];
        assert_eq!(bbc_2.name, "BBC TWO");
        assert_eq!(bbc_2.service_id, 4287);
        assert_eq!(bbc_2.logical_channel_number, 2);
    }

    #[test]
    fn ensure_channel_name_accessible_from_channel_number() {
        let test_lock = TEST_LOCK.lock().unwrap();
        let data = create_two_entry_channel_data_vec();
        {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
        }
        let rc = add_logical_channel_number_for_service_id(4164, 1);
        assert!(rc);
        let rc = add_logical_channel_number_for_service_id(4287, 2);
        assert!(rc);
        assert_eq!(get_channel_name_of_logical_channel_number(1).unwrap(), "BBC ONE Lon");
        assert_eq!(get_channel_name_of_logical_channel_number(2).unwrap(), "BBC TWO");
        assert_eq!(get_channel_name_of_logical_channel_number(10), None);
    }

    #[test]
    fn write_and_read_channels_data_cache() {
        let test_lock = TEST_LOCK.lock().unwrap();
        let data = create_two_entry_channel_data_vec();
        {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
        }
        let rc = add_logical_channel_number_for_service_id(4164, 1);
        assert!(rc);
        let rc = add_logical_channel_number_for_service_id(4287, 2);
        assert!(rc);
        let mut file_path = tempfile::NamedTempFile::new().unwrap();
        write_channels_data_cache(file_path.path());
        let mut file = file_path.as_file_mut();
        // TODO Should not need this but it seems needed.
        //file.seek(SeekFrom::Start(0)).unwrap();
        let mut buffer = [0u8; 4096];
        match file.read(&mut buffer) {
            Ok(count) => {
                assert_eq!(count, 133);
                let result = String::from_utf8_lossy(&buffer[..count]).to_string();
                assert_eq!(result, "---
- name: BBC ONE Lon
  service_id: 4164
  logical_channel_number: 1
- name: BBC TWO
  service_id: 4287
  logical_channel_number: 2");
            },
            Err(e) => assert!(false, "Failed to read file {:?} – {}", file_path, e),
        }
        let result = read_channels_data_cache(file_path.path());
        let channel_data = CHANNELS_DATA.read().unwrap();
        assert_eq!(&result.unwrap(), (*channel_data).as_ref().unwrap());
    }
}
