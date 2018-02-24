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

use gtk;

use glib::FlagsClass;

use gstreamer;
use gstreamer::prelude::*;

pub struct GStreamerEngine {
    playbin: gstreamer::Element,
    video_element: gstreamer::Element,
    pub video_widget: gtk::Widget,
}

impl GStreamerEngine {

    pub fn new() -> GStreamerEngine {
        let playbin = gstreamer::ElementFactory::make("playbin", "playbin").expect("Failed to create playbin element");
        let (video_element, video_widget) = if let Some(gtkglsink) = gstreamer::ElementFactory::make("gtkglsink", None) {
            let glsinkbin = gstreamer::ElementFactory::make("glsinkbin", None).unwrap();
            glsinkbin
                .set_property("sink", &gtkglsink.to_value())
                .unwrap();
            let widget = gtkglsink.get_property("widget").unwrap();
            (glsinkbin, widget.get::<gtk::Widget>().unwrap())
        } else {
            let sink = gstreamer::ElementFactory::make("gtksink", None).unwrap();
            let widget = sink.get_property("widget").unwrap();
            (sink, widget.get::<gtk::Widget>().unwrap())
        };
        GStreamerEngine {
            playbin,
            video_element,
            video_widget,
        }
    }

    fn on_bus_message() {}

    pub fn set_mrl(&self, mrl: &str) {
        self.playbin.set_property("uri", &mrl).expect("Could not set URI on playbin.");
    }

    pub fn pause(&self) {
        let rv = self.playbin.set_state(gstreamer::State::Paused);
        assert_eq!(rv,  gstreamer::StateChangeReturn::Success);
    }

    pub fn play(&self) {
        let rv = self.playbin.set_state(gstreamer::State::Playing);
        assert_eq!(rv,  gstreamer::StateChangeReturn::Success);
    }

    pub fn stop(&self) {
        let rv = self.playbin.set_state(gstreamer::State::Null);
        assert_eq!(rv,  gstreamer::StateChangeReturn::Success);
    }

    pub fn get_volume(&self) -> f32 {
        self.playbin.get_property("volume").unwrap().get().unwrap()
    }

    pub fn set_volume(&self, value: &f32) {
        self.playbin.set_property("volume", value).unwrap();
    }

    pub fn set_mute_state(&self, mute: &bool) {
        self.playbin.set_property("mute", mute).unwrap();
    }

    pub fn get_subtitles_showing(&self) -> bool {
        let flags = self.playbin.get_property("flags").unwrap();
        let flags_class = FlagsClass::new(flags.type_()).unwrap();
        flags_class.is_set_by_nick(&flags,"text")
    }

    pub fn set_subtitles_showing(&self, state: bool) {
        let flags = self.playbin.get_property("flags").unwrap();
        let flags_class = FlagsClass::new(flags.type_()).unwrap();
        let flags_builder = flags_class.builder_with_value(flags).unwrap();
        let flags = if state {
            flags_builder.set_by_nick("text")
        } else {
            flags_builder.unset_by_nick("text")
        }
            .build()
            .unwrap();
        self.playbin.set_property("flags", &flags).unwrap();
    }

}
