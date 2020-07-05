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

use std::cell::Cell;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::RwLock;

use ini;
use lazy_static::lazy_static;
use percent_encoding;
use xdg;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
/// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH: &percent_encoding::AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');

/// Struct for the data of each channel.
///
/// It is assumed that these are the data pointed to by various indexes so as to
/// create lookups between for example logical_channel_number and name.
///
/// Fields of the struct that can be filled in from the initial read of the channel data file
/// are immutable. Other fields must be filled in by analysing the SI table data so are mutable.
#[derive(Clone, Eq, PartialEq)]
pub struct ChannelData {
    name: String,
    service_id: u16,
    logical_channel_number: u16, // Channel 0 is not used so 0 can be used as "not yet known".
}

lazy_static! {
    static ref CHANNELS_DATA: RwLock<Option<Vec<ChannelData>>> =
        RwLock::new(
            match ini::Ini::load_from_file(channels_file_path()) {
                Ok(ini) => Some(process_ini(&ini)),
                Err(_) => None,
            }
        );
}

/// Process an `Ini` to give the `Vec<ChannelData>`
fn process_ini(ini: &ini::Ini) -> Vec<ChannelData> {
    let mut rv = vec![];
    for section in ini.sections() {
        match section {
            Some(name) => {
                match ini.section(Some(name)) {
                    Some(properties) => {
                        rv.push(ChannelData{
                            name: name.to_string(),
                            service_id: properties.get("SERVICE_ID").unwrap().parse::<u16>().unwrap(),
                            logical_channel_number: 0,
                        })
                    },
                    None => {},
                }
            },
            None => {},
        }
    }
    rv
}

/// Read channels data from the channels file.
pub fn read_channels_file(path: &PathBuf) -> bool {
    match ini::Ini::load_from_file(path) {
        Ok(ini) => {
            let data = process_ini(&ini);
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
            true
        },
        Err(_) => {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = None;
            false
        },
    }
}

/// Extract the names of the channels from the channels file.
///
/// GStreamer uses the XDG directory structure with, currently, gstreamer-1.0 as its
/// name. The dvbsrc plugin assumes the name dvb-channels.conf. The DVBv5 file format
/// is INI style: a sequence of blocks, one for each channel, starting with a channel
/// name surrounded by brackets and then a sequence of binding of keys to values each
/// one indented.
fn get_names_from_channels_data(channels_data: &Vec<ChannelData>) -> Vec<String> {
    channels_data.iter().map(|x| x.name.clone() ).collect()
}

/// Return a `PathBuf` to the GStreamer dvbsrc plugin channels file using the XDG directory structure.
pub fn channels_file_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("gstreamer-1.0").expect("Cannot set XDG prefix.");
    let mut path_buf = xdg_dirs.get_config_home();
    path_buf.push("dvb-channels.conf");
    path_buf
}

/// Returns the names of the channels.
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

/// Encode a string as used for display to one suitable to be an MRL.
pub fn encode_to_mrl(channel_name: &String) -> String {
    "dvb://".to_owned() + &percent_encoding::utf8_percent_encode(channel_name, PATH).to_string()
}

/// Update the channels file data.
///
/// Return `true` if a change was made to the channels data, `false` otherwise.
pub fn add_logical_channel_number_for_service_id(service_id: u16, logical_channel_number: u16) -> bool {
    // TODO This does a full copy of the data structure, should a more efficient way
    //   of doing the update be found?
    let mut channels_data = CHANNELS_DATA.write().unwrap();
    match &*channels_data {
        Some(c_d) => {
            let mut rv = false;
            *channels_data = Some(c_d.iter().map(
                |x| {
                    if x.service_id == service_id {
                        rv = true;
                        //println!("****  Found the channel with service_id {}, named {}.", x.service_id, x.name.clone());
                        ChannelData{
                            name: x.name.clone(),
                            service_id: x.service_id,
                            logical_channel_number,
                        }
                    } else { x.clone() }
                }
            ).collect());
            rv
        },
        None => {
            //println!("**** Channels data is None.");
            false
        },
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
            // TODO Can we do better than linnear search, or does it not matter?
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

#[cfg(test)]
mod tests {

    use ini;

    use super::{
        add_logical_channel_number_for_service_id,
        channels_file_path, encode_to_mrl, process_ini,
        get_names_from_channels_data, read_channels_file,
        ChannelData, CHANNELS_DATA
    };
    use crate::channels_data::get_channel_name_of_logical_channel_number;

    #[test]
    fn get_names_from_empty_file() {
        let empty_input: Vec<ChannelData> = vec![];
        let empty_output: Vec<String> = vec![];
        assert_eq!(get_names_from_channels_data(&empty_input), empty_output);
    }

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

    fn create_small_data_set() -> Vec<ChannelData> {
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
        let result = create_small_data_set();
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

    #[test]
    fn process_channels_data_file() {
        let read_file = read_channels_file(&channels_file_path());
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
        let data = create_small_data_set();
        {
            let mut channels_data = CHANNELS_DATA.write().unwrap();
            *channels_data = Some(data);
        }
        {
            let channel_data = CHANNELS_DATA.read().unwrap();
            let data: &Vec<ChannelData> = (*channel_data).as_ref().unwrap();
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

        assert_eq!(get_channel_name_of_logical_channel_number(1).unwrap(), "BBC ONE Lon");
        assert_eq!(get_channel_name_of_logical_channel_number(2).unwrap(), "BBC TWO");
        assert_eq!(get_channel_name_of_logical_channel_number(10), None);
    }

}
