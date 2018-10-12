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

use std::process::Command;

//use gio;
use gio::prelude::*;
use glib;
use glib::prelude::*;
use gtk;
use gtk::prelude::*;

use gst;
use gst::prelude::*;

use send_cell::SendCell;

use dialogs::display_an_error_dialog;
use preferences;

// Cannot use GL stuff on Nouveau, so it is important to know if this is running on a Nouveau system.
// There is likely a much easier, and quicker, way of making this test.
fn is_using_nouveau() -> bool {
    let lsmod_output = Command::new("lsmod").output().unwrap().stdout;
    let lsmod_output = String::from_utf8(lsmod_output).unwrap();
    lsmod_output.contains("nouveau")
}

pub struct GStreamerEngine {
    playbin: gst::Element,
    video_element: gst::Element,
    pub video_widget: gtk::Widget, // FrontendWindow uses this for the overlay.
}

impl GStreamerEngine {

    pub fn new(application: &gtk::Application) -> GStreamerEngine {
        let playbin = gst::ElementFactory::make("playbin", "playbin").expect("Failed to create playbin element");
        let bus = playbin.get_bus().unwrap();
        // The compiler cannot determine that the bus watch callback will be executed by the same thread that
        // the gtk::Application object is created with, which must be the case, and so fails to compile unless we
        // use a SendCell.
        let application_clone = SendCell::new(application.clone());
        let application_clone_for_bus_watch = SendCell::new(application.clone());
        bus.add_watch(move |_, msg| {
            let application_for_bus_watch = application_clone_for_bus_watch.borrow();
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    display_an_error_dialog(
                        Some(&application_for_bus_watch.get_windows()[0]),
                        "There was an end of stream in the GStreamer system"
                    );
                },
                gst::MessageView::Error(error) => {
                    display_an_error_dialog(
                        Some(&application_for_bus_watch.get_windows()[0]),
                        &(String::from("There was an error reported on the GStreamer bus.\n\n") + &format!("{}", error.get_error()))
                    );
                },
                _ => (),
            };
            glib::Continue(true)
        });
        let create_non_gl_element_and_widget = || {
            match gst::ElementFactory::make("gtksink", None) {
                Some(sink) =>{
                    let widget = sink.get_property("widget").expect("Could not get 'widget' property.");
                    (Some(sink), widget.get::<gtk::Widget>())
                },
                None => {
                    display_an_error_dialog(
                        Some(&application_clone.borrow().get_windows()[0]),
                        "Could not create a 'gtksink'\n\nIs the gstreamer1.0-gtk3 package installed?"
                    );
                    (None, None)
                }
            }
        };
        let (video_element, video_widget) = if is_using_nouveau() || !preferences::get_use_opengl() {
            create_non_gl_element_and_widget()
        } else {
            match gst::ElementFactory::make("gtkglsink", None) {
                Some(gtkglsink) => {
                    match gst::ElementFactory::make("glsinkbin", None) {
                        Some(glsinkbin) => {
                            glsinkbin.set_property("sink", &gtkglsink.to_value()).expect("Could not set 'sink'property.");
                            let widget = gtkglsink.get_property("widget").expect("Could not get 'widget' property.");
                            (Some(glsinkbin), widget.get::<gtk::Widget>())
                        },
                        None => {
                            display_an_error_dialog(
                                Some(&application_clone.borrow().get_windows()[0]),
                                "Could not create a 'glsinkbin'\n\nIs the gstreamer1.0-gl package installed?."
                            );
                            (None, None)
                        }
                    }
                },
                None => create_non_gl_element_and_widget()
            }
        };
        if video_element.is_none() || video_widget.is_none() {
            display_an_error_dialog(
                Some(&application_clone.borrow().get_windows()[0]),
                "Since the GStreamer system could not be initialised\nMe TV cannot work as required and so is quitting."
            );
            application_clone.borrow().quit(); // TODO Is it right to quit at this point?
            assert_eq!("Why has the application not quit?", "");
            // TODO Why is this not quitting but terminating due to teh assertion failure?
        }
        let engine = GStreamerEngine {
            playbin,
            video_element: video_element.expect("'video_element' is not None so this cannot happen."),
            video_widget: video_widget.expect("'video_widget is not None, this cannot happen."),
        };
        engine.playbin.set_property("video-sink", &engine.video_element.to_value()).expect("Could not set 'video-sink' property");
        engine.video_element.set_property("force-aspect-ratio", &true.to_value()).expect("Could not set 'force-aspect-ration' property");
        engine.set_subtitles_showing(false);
        engine
    }

    pub fn set_mrl(&self, mrl: &str) {
        self.playbin.set_property("uri", &mrl).expect("Could not set URI on playbin.");
    }

    pub fn pause(&self) {
        let (rv, state, _pending) = self.playbin.get_state(gst::CLOCK_TIME_NONE);
        assert_ne!(rv, gst::StateChangeReturn::Failure);
        if state == gst::State::Playing {
            let rv = self.playbin.set_state(gst::State::Paused);
            assert_ne!(rv, gst::StateChangeReturn::Failure);
        }
    }

    pub fn play(&self) {
        let rv = self.playbin.set_state(gst::State::Playing);
        assert_ne!(rv,  gst::StateChangeReturn::Failure, "could not set play state, try 'GST_DEBUG=3 me-tv' to see why.");
    }

    pub fn stop(&self) {
        let rv = self.playbin.set_state(gst::State::Null);
        assert_ne!(rv,  gst::StateChangeReturn::Failure);
    }

    pub fn get_volume(&self) -> f64 {
        self.playbin.get_property("volume").unwrap().get().unwrap()
    }

    pub fn set_volume(&self, value: f64) {
        self.playbin.set_property("volume", &value).unwrap();
    }

    pub fn set_mute_state(&self, mute: &bool) {
        self.playbin.set_property("mute", mute).unwrap();
    }

    pub fn get_subtitles_showing(&self) -> bool {
        let flags = self.playbin.get_property("flags").unwrap();
        let flags_class = glib::FlagsClass::new(flags.type_()).unwrap();
        flags_class.is_set_by_nick(&flags,"text")
    }

    pub fn set_subtitles_showing(&self, state: bool) {
        let flags = self.playbin.get_property("flags").unwrap();
        let flags_class = glib::FlagsClass::new(flags.type_()).unwrap();
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
