// Copyright (C) 2019 Russel Winder <russel@winder.org.uk>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gst;

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
