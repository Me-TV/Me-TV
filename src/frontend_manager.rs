/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017–2019  Russel Winder
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

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

use futures::channel::mpsc::Sender;
use notify::{Watcher, RecursiveMode, RawEvent, op, raw_watcher};
use regex::Regex;

use control_window::Message;
use remote_control;

/// A struct to represent the identity of a specific frontend currently
/// available on the system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendId {
    pub adapter: u8,
    pub frontend: u8,
}

/// The path in the filesystem to the DVB related special files.
pub fn dvb_base_path() -> PathBuf { PathBuf::from("/dev/dvb") }

/// Return the path to the adapter director for a given adapter.
pub fn adapter_path(id: u8) -> PathBuf {
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

/// Search for any adapters already installed on start of the application.
///
/// Inform the GUI and the remote control manager of the presence of
/// any adaptors and frontends.
pub fn add_already_installed_adaptors(to_cw: &mut Sender<Message>) {
    if dvb_base_path().is_dir() {
        let mut adapter_number = 0;
        loop {
            if adapter_path(adapter_number).is_dir() {
                let mut fei = FrontendId{adapter: adapter_number, frontend: 0};
                loop {
                    // TODO Is it worth doing the check for special file or just check for existence.
                    let path = frontend_path(&fei);
                    match fs::metadata(&path) {
                        Ok(m) => {
                            // NB m.is_file() is false for special files. :-(
                            // Assume the special devices were are dealing with are
                            // character devices not block devices.
                            if m.file_type().is_char_device() {
                                to_cw.try_send(Message::FrontendAppeared{fei: fei.clone()}).unwrap();
                            }
                        },
                        Err(_) => break,
                    };
                    fei.frontend += 1;
                }
            } else {
                break;
            }
            adapter_number += 1;
        }
    }
}

// TODO Find out how to fix this problem of notification of the frontendX files..
//
// Experimental evidence from Fedora Rawhide indicates that some USB DVB devices
// (the newer ones) do not give an event for creating the frontend file.
// Some USB devices do not give an event for the demux device. All seem to give events
// for the dvr and net devices. All file removes are notified.

/// Ensure the name is adaptorXXX /frontendYYY where XXX and YYY are pure numeric,
/// and return a `FrontendId` based on these numbers.
fn frontend_id_from(path: &str) -> Option<FrontendId> {
    let regex = Regex::new(r"/dev/dvb/adapter([0-9]+)/frontend([0-9]+)").unwrap();
    if regex.is_match(&path) {
        let captures = regex.captures(path).unwrap();
        let adapter_number = u8::from_str_radix(&captures[1], 10).unwrap();
        let frontend_number= u8::from_str_radix(&captures[2], 10).unwrap();
        Some(FrontendId{adapter: adapter_number, frontend: frontend_number})
    } else {
        None
    }
}

/// The entry point for the thread that is the frontend manager process.
///
/// Distributes "appeared" and "disappeared" messages to the GUI and to the remote
/// control manager whenever an adaptor/frontend state changes.
pub fn run(mut to_cw: Sender<Message>) {
    thread::spawn({
        let tocw = to_cw.clone();
        move || remote_control::run(tocw)
    });
    add_already_installed_adaptors(&mut to_cw);
    let (transmit_end, receive_end) = channel();
    let mut watcher = raw_watcher(transmit_end).unwrap();
    watcher.watch("/dev", RecursiveMode::Recursive).unwrap();
    loop {
        match receive_end.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie: _cookie}) => {
                match op {
                    op::CREATE => {
                        let path = path.to_str().unwrap();
                        // Hack because of the lack of certainty that the frontend notifies.
                        if path.contains("dvb") && path.contains("adapter") && path.contains("dvr") {
                            let path = path.replace("dvr", "frontend");
                            if let Some(fei) = frontend_id_from(&path) {
                                to_cw.try_send(Message::FrontendAppeared{fei: fei.clone()}).unwrap();
                            }
                        }
                    },
                    op::REMOVE => {
                        let path = path.to_str().unwrap();
                        if path.contains("dvb") && path.contains("adapter") && path.contains("frontend") {
                            if let Some(fei) = frontend_id_from(&path) {
                                to_cw.try_send(Message::FrontendDisappeared{fei: fei.clone()}).unwrap();
                            }
                        }
                    },
                    _ => {},
                }
            },
            Ok(event) => println!("frontend_manager::run: broken event: {:?}", event),
            Err(e) => println!("frontend_manager::run: watch error: {:?}", e),
        }
    }
    println!("Frontend Manager terminated.");
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn adapter_path_is_correct(id: u8) -> bool {
            adapter_path(id).to_str().unwrap() == format!("/dev/dvb/adapter{}", id)
        }
    }

    quickcheck! {
        fn frontend_path_is_correct(a: u8, f: u8) -> bool {
            frontend_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/frontend{}", a, f)
        }
    }

    quickcheck! {
        fn demux_path_is_correct(a: u8, f: u8) -> bool {
            demux_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/demux{}", a, f)
        }
    }

    quickcheck! {
        fn dvr_path_is_correct(a: u8, f: u8) -> bool {
            dvr_path(&FrontendId{adapter: a, frontend: f}).to_str().unwrap() == format!("/dev/dvb/adapter{}/dvr{}", a, f)
        }
    }

    quickcheck! {
        fn check_frontend_id_from_with_correct_structure(adapter: u8, frontend: u8) -> bool {
            Some(FrontendId{adapter: adapter, frontend: frontend}) == frontend_id_from(&format!("/dev/dvb/adapter{}/frontend{}", adapter, frontend))
        }
    }

    quickcheck! {
        fn check_frontend_id_from_with_incorrect_structure(prefix: String, postfix: String, adapter: u8, frontend: u8) -> bool {
            None == frontend_id_from(&format!("{}/adapter{}/frontend{}{}", prefix, adapter, frontend, postfix))
         }
    }

}
