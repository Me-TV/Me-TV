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

use glib;

use gstreamer as gst;

use crate::control_window::Message;

#[derive(Debug)]
pub struct EPGEvent {
    pub service_id: u16,
    pub event_id: u16,
    pub start_time: gst::DateTime,
    pub duration: u32,
}

impl EPGEvent {

    pub fn new(service_id: u16, event_id: u16, start_time: gst::DateTime, duration: u32) -> EPGEvent {
        EPGEvent{
            service_id,
            event_id,
            start_time,
            duration,
        }
    }

}

unsafe impl Send for EPGEvent {}
unsafe impl Sync for EPGEvent {}

pub fn run(mut to_cw: glib::Sender<Message>, from_gstreamer: std::sync::mpsc::Receiver<EPGEvent>) {
    loop {
        let event = from_gstreamer.recv();
        println!("EPG Manager received: {:?}", &event);
    }
}
