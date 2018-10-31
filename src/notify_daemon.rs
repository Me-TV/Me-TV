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

use std::sync::mpsc::channel;

use futures::channel::mpsc::Sender;

use notify::{Watcher, RecursiveMode, RawEvent, op, raw_watcher};

use regex::Regex;

use frontend_manager::{FrontendId, Message};

// TODO Find out how to fix this problem of notification of the frontendX files..
//
// Experimental evidence from Fedora Rawhide indicates that some USB DVB devices
// (the newer ones) do not give an event for creating the frontend file.
// Some USB devices do not give an event for the demux device. All seem to give events
// for the dvr and net devices. All file removes are notified.

/// Ensure the name is adaptorXXX /frontendYYY where XXX and YYY are pure numeric,
///and return a `FrontendId` based on these numbers.
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

/// The function that drives the inotify daemon.
pub fn run(mut to_cw: Sender<Message>) {
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
                                to_cw.try_send(Message::FrontendAppeared{fei: fei}).unwrap();
                            }
                        }
                    },
                    op::REMOVE => {
                        let path = path.to_str().unwrap();
                        if path.contains("dvb") && path.contains("adapter") && path.contains("frontend") {
                            if let Some(fei) = frontend_id_from(&path) {
                                to_cw.try_send(Message::FrontendDisappeared{fei: fei}).unwrap();
                            }
                        }
                    },
                    _ => {},
                }
            },
            Ok(event) => println!("notify_daemon: broken event: {:?}", event),
            Err(e) => println!("notify_daemon: watch error: {:?}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn check_frontend_id_from_with_correct_structure(adapter: u16, frontend: u16) -> bool {
            Some(FrontendId{adapter: adapter, frontend: frontend}) == frontend_id_from(&format!("/dev/dvb/adapter{}/frontend{}", adapter, frontend))
        }
    }

    quickcheck! {
        fn check_frontend_id_from_with_incorrect_structure(prefix: String, postfix: String, adapter: u16, frontend: u16) -> bool {
            None == frontend_id_from(&format!("{}/adapter{}/frontend{}{}", prefix, adapter, frontend, postfix))
         }
    }

}
