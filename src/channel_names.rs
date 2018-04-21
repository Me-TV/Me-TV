/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017, 2018  Russel Winder
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

extern crate xdg;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;

/// An internal function that can be tested.
fn get_names_from_file(file: &File) -> Vec<String> {
    let buf_reader = BufReader::new(file);
    buf_reader
        .lines()
        .filter(|i|{ i.is_ok() })
        .map(|i|{ String::from(i.unwrap().trim()) })
        .filter(|i|{ i.starts_with('[') && i.ends_with(']') })
        .map(|i|{ String::from(i[1..(i.len() - 1)].trim()) })
        .collect()
}

/// Return the `PathBuf` to the GStreamer dvbsrc plugin channels file using the XDG directory structure.
pub fn channels_file_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("gstreamer-1.0").expect("Cannot set XDG prefix.");
    let mut path_buf = xdg_dirs.get_config_home();
    path_buf.push("dvb-channels.conf");
    path_buf
}

/// Read the file that the GStreamer dvbsrc plugin uses and extract a list of the channels.
///
/// GStreamer uses the XDG directory structure with, currently, gstreamer-1.0 as its
/// name. The dvbsrc plugin assumes the name dvb-channels.conf. The DVBv5 file format
/// is INI/TOML style: a sequence of blocks, one for each channel, starting with a channel
/// name surrounded by brackets and then a sequence of binding of keys to values each
/// one indented.
pub fn get_names() -> Option<Vec<String>> {
    match File::open(channels_file_path()) {
        Ok(file) => Some(get_names_from_file(&file)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::get_names_from_file;

    extern crate tempfile;

    use std::io::{Write, Read, Seek, SeekFrom};

    #[test]
    fn empty_file() {
        let tmpfile = tempfile::tempfile().unwrap();
        let empty_vector: Vec<String> = vec![];
        assert_eq!(get_names_from_file(&tmpfile), empty_vector);
    }

    fn some_channel_blocks() {
        let mut tmpfile = tempfile::tempfile().unwrap();
        let result = vec!["one two", "three four", "five six"];
        for item in result.iter() {
            tmpfile.write_all(format!("\n[{}]\n", item).as_bytes()).unwrap();
            tmpfile.write_all("\
	SERVICE_ID = 4164
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
	DELIVERY_SYSTEM = DVBT".as_bytes()).unwrap();
        }
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(get_names_from_file(&tmpfile), result);
    }

}
