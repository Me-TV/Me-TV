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
use gstreamer_mpegts::EITDescriptor;

use crate::control_window::Message;

#[derive(Debug)]
pub struct EPGEventMessage {
    pub service_id: u16,
    pub event_id: u16,
    pub start_time: gst::DateTime,
    pub duration: u32,
    pub descriptors: Vec<EITDescriptor>
}

impl EPGEventMessage {

    pub fn new(service_id: u16, event_id: u16, start_time: gst::DateTime, duration: u32, descriptors: Vec<EITDescriptor>) -> EPGEventMessage {
        EPGEventMessage {
            service_id,
            event_id,
            start_time,
            duration,
            descriptors,
        }
    }

}

unsafe impl Send for EPGEventMessage {}
unsafe impl Sync for EPGEventMessage {}

pub fn run(mut to_cw: glib::Sender<Message>, from_gstreamer: std::sync::mpsc::Receiver<EPGEventMessage>) {
    loop {
        let event = from_gstreamer.recv();
        //println!("EPG Manager received: {:?}", &event);
        //
        // What is the best data structure for the EPG? The rendering will
        // be by channel number and date/time, so these seem to be the
        // indexes needed. The question is how to structure the indexes.
        // The issue is whether date/time first then channel number.
        //
    }
}
