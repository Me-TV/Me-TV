/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2019  Russel Winder
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
use std::sync::mpsc::channel;

use futures::channel::mpsc::Sender;

use notify::{Watcher, RecursiveMode, RawEvent, op, raw_watcher};

/// An enumeration of all the message types that  can be sent by
/// the remote control notify daemon.
#[derive(Debug)]
pub enum Message {
    RemoteControlAppeared{sys_rc_path: PathBuf},
    RemoteControlDisappeared{sys_rc_path: PathBuf},
}

/// The function that drives the remote control notify daemon.
pub fn run(mut to_cw: Sender<Message>) {
    let (transmit_end, receive_end) = channel();
    let mut watcher = raw_watcher(transmit_end).unwrap();
    watcher.watch("/sys/class/rc", RecursiveMode::Recursive).unwrap();
    loop {
        match receive_end.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie: _cookie}) => {
                match op {
                    op::CREATE => {
                        to_cw.try_send(Message::RemoteControlAppeared{sys_rc_path: path}).unwrap()
                    },
                    op::REMOVE => {
                        to_cw.try_send(Message::RemoteControlDisappeared{sys_rc_path: path}).unwrap()
                    },
                    _ => {},
                }
            },
            Ok(event) => println!("remote_control_notify_daemon: broken event: {:?}", event),
            Err(e) => println!("remote_control_notify_daemon: watch error: {:?}", e),
        }
    }
}
