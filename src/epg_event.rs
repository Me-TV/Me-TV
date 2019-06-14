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


use gstreamer as gst;

#[derive(Debug)]
pub struct EPGEvent {
    pub service_id: u16,
    pub event_id: u16,
    pub start_time: gst::DateTime,
    pub duration: u32,
}

impl EPGEvent {

    pub fn new() -> EPGEvent {
        EPGEvent{
            service_id: 0,
            event_id: 0,
            start_time: gst::DateTime::new_now_local_time(),
            duration: 0,
        }
    }

}

unsafe impl Send for EPGEvent {}
