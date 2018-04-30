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

use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};

use notify::{Watcher, RecursiveMode, RawEvent, op, raw_watcher};

use frontend_manager::dvb_base_path;

/// Messages that can be sent from here to the frontend manager.
pub enum Message {
    AdapterAppeared{id: u16},
    AdapterDisappeared{id: u16},
}

/// Ensure the name is adaptorXXX where XXX is pure numeric, return the
/// integer value of XXX.
fn extract_adapter_number(name: &str) -> Option<u16> {
    let matches: Vec<&str> = name.split("adapter").collect();
    if matches.len() == 2 {
        match u16::from_str_radix(matches.get(1).unwrap(), 10) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Set up the notify watch on the existing DVB directory.
fn set_watch_on_dvb(to_fem: &Sender<Message>) {
    let (transmit_end, receive_end) = channel();
    let mut watcher = raw_watcher(transmit_end).unwrap();
    watcher.watch("/dev/dvb", RecursiveMode::NonRecursive).unwrap();
    loop {
        match receive_end.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie: _cookie}) => {
                match op {
                    op::CREATE => {
                        if path.is_dir() {
                            if let Some(created_adapter_number) = extract_adapter_number(path.file_name().unwrap().to_str().unwrap()) {
                                to_fem.send(Message::AdapterAppeared{id: created_adapter_number}).unwrap();
                            }
                        }
                    },
                    op::REMOVE => {
                        if path == PathBuf::from("/dev/dvb") { break; }
                        if let Some(removed_adapter_number) = extract_adapter_number(path.file_name().unwrap().to_str().unwrap()) {
                            to_fem.send(Message::AdapterDisappeared{id: removed_adapter_number}).unwrap();
                        }
                    },
                    _ => {},
                }
            },
            Ok(event) => println!("broken event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

/// Set up the notify watch on the /dev directory as there is currently no DVB directory.
fn set_watch_on_dev(to_fem: &Sender<Message>) {
    let (transmit_end, receive_end) = channel();
    let mut watcher = raw_watcher(transmit_end).unwrap();
    watcher.watch("/dev", RecursiveMode::NonRecursive).unwrap();
    loop {
        match receive_end.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie: _cookie}) => {
                if op == op::CREATE && path == PathBuf::from("/dev/dvb")  && path.is_dir() {
                    to_fem.send(Message::AdapterAppeared {id: 0}).unwrap();
                    set_watch_on_dvb(&to_fem)
                }
            },
            Ok(event) => println!("broken event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

/// The function that drives the inotify daemon.
pub fn run(to_fem: Sender<Message>) {
    if dvb_base_path().is_dir() { set_watch_on_dvb(&to_fem); }
    else { set_watch_on_dev(&to_fem); }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn check_return_adapter_number_with_correct_structure(id: u16) -> bool {
            Some(id) == extract_adapter_number(&("adapter".to_string() + &id.to_string()))
        }
    }

    quickcheck! {
        fn check_return_adapter_number_with_incorrect_structure(prefix: String, id: u16) -> bool {
            None == extract_adapter_number(&(prefix + &id.to_string()))
        }
    }

}
