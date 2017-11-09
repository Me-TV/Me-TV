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

extern crate xdg;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

/// Read the file that the GStreamer dvbsrc plugin uses and extract a list of the channels.
///
/// GStreamer uses the XDG directory structure with, currently, gstreamer-1.0 as its
/// name. The dvbsrc plugin assumes the name dvb-channels.conf. The DVBv5 file format
/// is INI/TOML style: a sequence of blocks, one for each channel, starting with a channel
/// name surrounded by brackets and then a sequence of binding of keys to values each
/// one indented.
pub fn get_names() -> Vec<String> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("gstreamer-1.0").expect("Cannot set XDG prefix.");
    let path = xdg_dirs.find_config_file("dvb-channels.conf").expect("Cannot set XDG path to config file.");
    let file = File::open(path).expect("Cannot open config file.");
    let buf_reader = BufReader::new(file);
    buf_reader
        .lines()
        .filter(|i|{ i.is_ok() })
        .map(|i|{ String::from(i.unwrap().trim()) })
        .filter(|i|{ i.starts_with('[') && i.ends_with(']') })
        .map(|i|{ String::from(i[1..(i.len() - 1)].trim()) })
        .collect()
}

// TODO Need a test or seven for this function.