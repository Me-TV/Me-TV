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

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use std::os::unix::fs::FileTypeExt;

/// A struct to represent the identity of a specific frontend currently
/// available on the system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendId {
    pub adapter: u16,
    pub frontend: u16,
}

///  A struct to represent a tuning of a frontend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuningId {
    pub frontend: FrontendId,
    pub channel: RefCell<Option<String>>,
}

/// An enumeration of all the message types that  can be sent by
/// the frontend manager.
pub enum Message {
    FrontendAppeared{fei: FrontendId},
    FrontendDisappeared{fei: FrontendId},
}

/// The path in the filesystem to the DVB related special files.
pub fn dvb_base_path() -> PathBuf { PathBuf::from("/dev/dvb") }

/// Return the path to the adapter director for a given adapter.
pub fn adapter_path(id: u16) -> PathBuf {
    let mut result = dvb_base_path();
    result.push("adapter".to_string() + &id.to_string());
    result
}

/// Return the path to the special file for a given frontend.
pub fn frontend_path(fei: &FrontendId) -> PathBuf {
    let mut result = adapter_path(fei.adapter);
    result.push("frontend".to_string() + &fei.frontend.to_string());
    result
}

/// Return the path to the special file of the demux for a given frontend.
pub fn demux_path(fei: &FrontendId) -> PathBuf {
    let mut result = adapter_path(fei.adapter);
    result.push("demux".to_string() + &fei.frontend.to_string());
    result
}

/// Return the path to the special file of the data for a given frontend.
pub fn dvr_path(fei: &FrontendId) -> PathBuf {
    let mut result = adapter_path(fei.adapter);
    result.push("dvr".to_string() + &fei.frontend.to_string());
    result
}

/// Process a newly present adapter to inform the control window of all the frontends
/// newly accessible.
fn add_frontends(to_cw: &Sender<Message>, id: u16) {
    let mut fei = FrontendId{adapter: id, frontend: 0};
    loop {
        // TODO Is it worth doing the check for special file or just check for existence.
        let path = frontend_path(&fei);
        match fs::metadata(&path) {
            Ok(m) => {
                // NB m.is_file() is false for special files. :-(
                // Assume the special devices were are dealing with are
                // character devices not block devices.
                if m.file_type().is_char_device() {
                    to_cw.send(Message::FrontendAppeared{fei: fei.clone()}).unwrap();
                }
            },
            Err(error) => break,
        };
        fei.frontend += 1;
    }
}

/// Search for any adapters already installed on start of the application
pub fn search_and_add_adaptors(to_cw: &Sender<Message>) {
    if dvb_base_path().is_dir() {
        let mut adapter_number = 0;
        loop {
            if adapter_path(adapter_number).is_dir() {
                add_frontends(to_cw, adapter_number);
            } else {
                break;
            }
            adapter_number += 1;
        }
    }
}

/// The entry point for the thread that is the front end manager process.
pub fn run(from_in: Receiver<Message>, to_cw: Sender<Message>) {
    search_and_add_adaptors(&to_cw);
    loop {
        match from_in.recv() {
            Ok(r) => {
                match r {
                  Message::FrontendAppeared{fei} => {
                      to_cw.send(Message::FrontendAppeared{fei}).unwrap();
                  },
                  Message::FrontendDisappeared{fei} => {
                      to_cw.send(Message::FrontendDisappeared{fei}).unwrap();
                  },
                }
            },
            Err(_) => {
                println!("Frontend Manager got an Err, notify end of channel must have dropped.");
                break;
            },
        }
    }
    println!("Frontend Manager terminated.");
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn adapter_path_is_correct(id: u16) -> bool {
            adapter_path(id).to_str().unwrap() == format!("/dev/dvb/adapter{}", id)
        }
    }

    quickcheck! {
        fn frontend_path_is_correct(a: u16, f: u16) -> bool {
            frontend_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/frontend{}", a, f)
        }
    }

    quickcheck! {
        fn demux_path_is_correct(a: u16, f: u16) -> bool {
            demux_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/demux{}", a, f)
        }
    }

    quickcheck! {
        fn dvr_path_is_correct(a: u16, f: u16) -> bool {
            dvr_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/dvr{}", a, f)
        }
    }

}
