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

use std::fs;
use std::sync::mpsc::Sender;

use inotify::{
    EventMask,
    Inotify,
    WatchMask,
};

use frontend_manager::{
    DVB_BASE_PATH,
};

/// Messages that can be sent from here to the frontend manager.
pub enum Message {
    AdapterAppeared{id: u16},
    AdapterDisappeared{id: u16},
}

/// Ensure the name is adaptorXXX where XXX is pure numeric, return the
/// integer value of XXX.
fn return_adapter_number(name: &str) -> Result<u16, ()> {
    let matches: Vec<&str> = name.split("adapter").collect();
    if matches.len() == 2 {
        match u16::from_str_radix(matches.get(1).unwrap(), 10) {
            Ok(r) => Ok(r),
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}

/// Set up the inotify watch on the existing DVB directory.
fn set_watch_on_dvb(inotify: &mut Inotify, to_fem: &Sender<Message>) {
    inotify
        .add_watch(DVB_BASE_PATH, WatchMask::CREATE | WatchMask::DELETE | WatchMask::DELETE_SELF)
        .expect("Could not set up the watch on the DVB directory." );
    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events.");
        for event in events {
            if event.mask.contains(EventMask::CREATE) {
                if event.mask.contains(EventMask::ISDIR) {
                    let new_adapter_number = return_adapter_number(event.name.unwrap().to_str().unwrap()).unwrap();
                    to_fem.send(Message::AdapterAppeared {id: new_adapter_number}).unwrap();
                }
            } else if event.mask.contains(EventMask::DELETE) {
                if event.mask.contains(EventMask::ISDIR) {
                    let removed_adapter_number = return_adapter_number(event.name.unwrap().to_str().unwrap()).unwrap();
                    to_fem.send(Message::AdapterDisappeared {id: removed_adapter_number }).unwrap();
                }
            } else if event.mask.contains(EventMask::DELETE_SELF) {
                // If this directory is going, it is fairly certain all the adapters
                // have been removed already.
                return;
            }
        }
    }}

/// Set up the inotify watch on the /dev/directory as there is currently no DVB directory.
fn set_watch_on_dev(inotify: &mut Inotify, to_fem: &Sender<Message>) {
    inotify
        .add_watch("/dev", WatchMask::CREATE)
        .expect("Could not set up the watch on the /dev directory.");
    let mut buffer = [0u8; 4096];
    let mut dvb_present = false;
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events.");
        for event in events {
            if event.mask.contains(EventMask::ISDIR) {
                if event.name.unwrap() == "dvb" {
                    // TODO is it right to assume 0 here?
                    to_fem.send(Message::AdapterAppeared {id: 0}).unwrap();
                    dvb_present = true;
                }
            }
        }
        if dvb_present {
            set_watch_on_dvb(inotify, to_fem);
            dvb_present = false;
        }
    }
}

/// The function that drives the inotify daemon.
pub fn run(to_fem: Sender<Message>) {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    match fs::metadata(DVB_BASE_PATH) {
        Ok(_)  => set_watch_on_dvb(&mut inotify, &to_fem),
        Err(_) => set_watch_on_dev(&mut inotify, &to_fem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn check_return_adapter_number_with_correct_structure(id: u16) -> bool {
            Ok(id) == return_adapter_number(&("adapter".to_string() + &id.to_string()))
        }
    }

    quickcheck! {
        fn check_return_adapter_number_with_incorrect_structure(prefix: String, id: u16) -> bool {
            Err(()) == return_adapter_number(&(prefix + &id.to_string()))
        }
    }

}
