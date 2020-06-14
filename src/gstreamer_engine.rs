/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017–2020  Russel Winder
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
use std::rc::Rc;

//use gio;
//use gio::prelude::*;
use glib;
use glib::prelude::*;
//use glib::translate::*;
use gtk;
use gtk::prelude::*;

use gst;
use gst::prelude::*;

use gst_mpegts;

use fragile::Fragile;

use crate::control_window_button::ControlWindowButton;
use crate::dialogs::display_an_error_dialog;
use crate::epg_manager;
use crate::preferences;

/// Is nouveau the device driver?
///
/// Cannot use GL stuff on Nouveau, so it is important to know if this is running on a Nouveau
/// system.
// There is likely a much easier, and quicker, way of making this test.
fn is_using_nouveau() -> bool {
    let lsmod_output = Command::new("lsmod").output().unwrap().stdout;
    let lsmod_output = String::from_utf8(lsmod_output).unwrap();
    lsmod_output.contains("nouveau")
}

/// The GStreamer elements and GTK+ widgets that are the bits that do the work of rendering the
/// television or radio channel.
#[derive(Debug)]
pub struct GStreamerEngine {
    playbin: gst::Element,
    video_element: gst::Element,
    pub video_widget: gtk::Widget, // FrontendWindow uses this for the overlay.
}

impl GStreamerEngine {

    pub fn new(control_window_button: Rc<ControlWindowButton>) -> Result<GStreamerEngine, ()> {
        let playbin = gst::ElementFactory::make("playbin", Some("playbin")).expect("Failed to create playbin element");
        playbin.connect("element-setup",  false, {
            let fei = control_window_button.frontend_id.clone();
            move |values| {
                // values[0] .get::<gst::Element>() is an Option on the playbin itself.
                let element = values[1]
                    .get::<gst::Element>()
                    .expect("Failed to get a handle on the Element being created")
                    .expect("Got None rather than an Some<Element>");
                if let Some(element_factory) = element.get_factory() {
                    if element_factory.get_name() == "dvbsrc" {
                        let adapter_number = element
                            .get_property("adapter")
                            .expect("Could not retrieve adapter number Value")
                            .get::<i32>()
                            .expect("Could not get the i32 value from the adapter number Value")
                            .expect("Got None rather than Some<u32>") as u8;
                        let frontend_number = element
                            .get_property("frontend")
                            .expect("Could not retrieve frontend number Value.")
                            .get::<i32>()
                            .expect("Could not get the i32 value from the frontend number Value")
                            .expect("Got None rather than Some<u32>") as u8;
                        if adapter_number != fei.adapter {
                            element.set_property("adapter", &(fei.adapter as i32).to_value()).expect("Could not set adapter number on dvbsrc element");
                        }
                        if frontend_number != fei.frontend {
                            element.set_property("frontend", &(fei.frontend as i32).to_value()).expect("Could not set frontend number of dvbsrc element");
                        }
                    }
                    else if element_factory.get_name() == "deinterlace" {
                        // Assumption is that we are using non-GL. Should never get here if GL
                        // is being used.
                        let method = element.get_property("method").expect("Could not get method from deinterlace element.");
                        let method_enumvalue = glib::EnumValue::from_value(&method).expect("Could not get EnumValue from Value.");
                        let method_nick = method_enumvalue.get_nick();
                        let enum_class = glib::EnumClass::new(method.type_()).expect("Could not get EnumClass for method.");
                        let new_method_nick = preferences::get_nongl_deinterlace_method().expect("Failed to get nongl_deinterlace_method.");
                        if new_method_nick != "" && new_method_nick != method_nick {
                            let new_method = enum_class.get_value_by_nick(&new_method_nick).expect(&format!("Failed to get new method EnumValue for {}", &new_method_nick));
                            element.set_property("method", &new_method.to_value()).expect("Failed to set method property.");
                        }
                    }
                }
                None
            }
        }).expect("Could not connect a handler to the element-setup signal.");
        let bus = playbin.get_bus().unwrap();
        // The compiler cannot determine that the bus watch callback will be executed by the
        // same thread that the gtk::Application and ControlWindowButtons objects are created
        // with, which must be the case, and so fails to compile unless we use Fragile.
        let application = &control_window_button.control_window.window.get_application().unwrap();
        let application_clone = Fragile::new(application.clone());
        bus.add_watch({
            let application_clone = Fragile::new(application.clone());
            let control_window_button_clone = Fragile::new(control_window_button.clone());
            move |_, msg| {
                let application = application_clone.get();
                let control_window_button = control_window_button_clone.get();
                match msg.view() {
                    gst::MessageView::Element(element) => {
                        if let Some(structure) = element.get_structure() {
                            match structure.get_name() {
                                "cat" => {},
                                "dvb-adapter" => {
                                    // TODO Do we need to process this at all?  It seems there
                                    // is only one of these sent, at the opening of a connection
                                    // to an adapter.
                                },
                                "dvb-frontend-stats" => {
                                    // TODO Do we need to process this at all?  Lots of these
                                    // get sent out, but it is not clear what the benefit of
                                    // processing them is – at least not at this time anyway.
                                },
                                "dvb-read-failure" => {
                                    // TODO What should be done on a read failure?  For now the
                                    // read fails are simply ignored.
                                    println!("********    Got a DVB read failure.");
                                },
                                "eit" => {
                                    if let Some(section) = gst_mpegts::Section::from_element(&element) {
                                        if section.get_section_type() == gst_mpegts::SectionType::Eit {
                                            if let Some(eit) = section.get_eit() {
                                                for event in eit.event_iterator() {
                                                    let event_message = epg_manager::EPGEventMessage::new(
                                                        section.get_subtable_extension(),
                                                        event.get_event_id(),
                                                        event.get_start_time(),
                                                        event.get_duration(),
                                                        event.get_descriptors(),
                                                    );
                                                    control_window_button.control_window.to_epg_manager.send(event_message).unwrap();
                                                }
                                            } else {
                                                //  TODO This seems to happen, and yet it shouldn't.
                                                println!("********    Could not get an EIT from a supposed EIT Section: {:?}", section);
                                                println!("********        Section type: {:?}", section.get_section_type());
                                                println!("********        EIT: {:?}", section.get_eit());
                                            }
                                        } else {
                                            panic!("************  EIT Section is not an EIT Section: {:?}", section);
                                        }
                                    } else {
                                        panic!("************  Could not get a Section from an EIT Section Element: {:?}", element);
                                    }
                                },
                                "GstNavigationMessage" => {},
                                "nit" => {
                                    if let Some(section) = gst_mpegts::Section::from_element(&element) {
                                        if section.get_section_type() == gst_mpegts::SectionType::Nit {
                                            if let Some(nit) = section.get_nit() {
                                                println!("========  Got a NIT {:?}", nit);
                                            }else {
                                                panic!("************    Could not get a NIT from a NIT Section: {:?}", section);
                                            }
                                        } else {
                                            panic!("************  NIT Section is not an NIT Section: {:?}", section);
                                        }
                                    } else {
                                        panic!("************  Could not get a Section from a NIT Section Element: {:?}", element);
                                    }
                                },
                                "pat" => {},
                                "pmt" =>{},
                                "sdt" => {},
                                "section" => {},
                                "tdt" => {},
                                "tot" => {},
                                _ => println!("Unknown Element type: {:?}", element),
                            }
                        } else {
                            panic!("Element has no Structure: {:?}", element);
                        }
                    },
                    gst::MessageView::Eos(..) => {
                        display_an_error_dialog(
                            Some(&application.get_windows()[0]),
                            "There was an end of stream in the GStreamer system"
                        );
                    },
                    gst::MessageView::Error(error) => {
                        display_an_error_dialog(
                            Some(&application.get_windows()[0]),
                            &format!("There was an error reported on the GStreamer bus.\n\n'{}'\n\nBest bet is to close this channel window and start a new one from the control window.", error.get_error())
                        );
                    },
                    _ => (),
                };
                glib::Continue(true)
            }
        }).unwrap();
        let create_non_gl_element_and_widget = || {
            match gst::ElementFactory::make("gtksink", None) {
                Ok(sink) =>{
                    let widget = sink.get_property("widget").expect("Could not get 'widget' property.");
                    (Some(sink), widget.get::<gtk::Widget>().unwrap())
                },
                Err(_) => {
                    display_an_error_dialog(
                        Some(&application_clone.get().get_windows()[0]),
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
                Ok(gtkglsink) => {
                    match gst::ElementFactory::make("glsinkbin", None) {
                        Ok(glsinkbin) => {
                            match gst::ElementFactory::make("gldeinterlace", None) {
                                Ok(gldeinterlace) => {
                                    let flags = playbin.get_property("flags").expect("Could not get 'flags' property.");
                                    let flags_class = glib::FlagsClass::new(flags.type_()).expect("Could not get the flags FlagClass.");
                                    let flags_builder = flags_class.builder_with_value(flags).expect("Could not get the flags FlagsBuilder.");
                                    let flags = flags_builder.unset_by_nick("deinterlace").build().expect("Could not remove 'deinterlace' from the set of elements playbin might use.");
                                    playbin.set_property("flags", &flags).expect("Could not set the new 'flags' value on playbin.");
                                    // TODO A gst::Bin does not have the properties of a video
                                    //   sink and so when the pipeline diagram is drawn there
                                    //   are a lot of missing properties that are trying to be
                                    //   found. Should consider creating a subclass of Bin with
                                    //   all the relevant properties to avoid all the warnings.
                                    let the_bin = gst::Bin::new(None);
                                    the_bin.add(&gldeinterlace).expect("Could not add the gldeinterlace element to the new bin.");
                                    the_bin.add(&gtkglsink).expect("Could not add the gtkglsink to the new bin.");
                                    gldeinterlace.link(&gtkglsink).expect("Could not link the gldeinterlace element to the gtkglsink element.");
                                    let sink_pad = gst::GhostPad::new(
                                        Some("sink"),
                                        &gldeinterlace.get_static_pad("sink").expect("Could not get sink pad of gldeinterlace element.")
                                    ).expect("Could not create ghost pad.");
                                    the_bin.add_pad(&sink_pad).expect("Could not add the sink pad to the bin.");
                                    // Set the deinterlacing method as per the preferences.
                                    let method = gldeinterlace.get_property("method").expect("Could not get method from gldeinterlace element.");
                                    let method_enumvalue = glib::EnumValue::from_value(&method).expect("Could not get EnumValue from Value.");
                                    let method_nick = method_enumvalue.get_nick();
                                    let enum_class = glib::EnumClass::new(method.type_()).expect("Could not get EnumClass for method.");
                                    let new_method_nick = preferences::get_gl_deinterlace_method().expect("Failed to get gl_deinterlace_method.");
                                    if new_method_nick != "" && new_method_nick != method_nick {
                                        let new_method = enum_class.get_value_by_nick(&new_method_nick).expect(&format!("Failed to get new method EnumValue for {}", &new_method_nick));
                                        gldeinterlace.set_property("method", &new_method.to_value()).expect("Failed to set method property.");
                                    }
                                    glsinkbin.set_property("sink", &the_bin.to_value()).expect("Could not set 'sink'property.");
                                },
                                Err(e) => {
                                    display_an_error_dialog(
                                        Some(&application_clone.get().get_windows()[0]),
                                        "Could not create an OpenGL deinterlace element,\ncontinuing without deinterlacing."
                                    );
                                    glsinkbin.set_property("sink", &gtkglsink.to_value()).expect("Could not set 'sink'property.");
                                },
                            };
                            let widget = gtkglsink.get_property("widget").expect("Could not get 'widget' property.");
                            (Some(glsinkbin), widget.get::<gtk::Widget>().unwrap())
                        },
                        Err(_) => {
                            display_an_error_dialog(
                                Some(&application_clone.get().get_windows()[0]),
                                "Could not create a 'glsinkbin element.'\n\nIs the gstreamer1.0-gl package installed?.\n\nContinuing without OpenGL support."
                            );
                            create_non_gl_element_and_widget()
                        },
                    }
                },
                Err(_) => {
                    display_an_error_dialog(
                        Some(&application_clone.get().get_windows()[0]),
                        "Could not create a 'gtkglsink element.'\n\nIs the gstreamer1.0-gl package installed?.\n\nContinuing without OpenGL support."
                    );
                    create_non_gl_element_and_widget()
                },
            }
        };
        if video_element.is_none() || video_widget.is_none() {
            display_an_error_dialog(
                Some(&application_clone.get().get_windows()[0]),
                "Since the GStreamer system could not be initialised\nMe TV cannot work as required and so is quitting."
            );
            Err(())
        } else {
            let engine = GStreamerEngine {
                playbin,
                video_element: video_element.expect("'video_element' is None, this cannot happen."),
                video_widget: video_widget.expect("'video_widget is None, this cannot happen."),
            };
            engine.video_element.set_property("force-aspect-ratio", &true.to_value()).expect("Could not set 'force-aspect-ration' property");
            engine.playbin.set_property("video-sink", &engine.video_element.to_value()).expect("Could not set 'video-sink' property");
            engine.set_subtitles_showing(false);
            Ok(engine)
        }
    }

    pub fn set_mrl(&self, mrl: &str) {
        self.playbin.set_property("uri", &mrl).expect("Could not set URI on playbin.");
    }

    pub fn pause(&self) {
        let (rv, state, _pending) = self.playbin.get_state(gst::CLOCK_TIME_NONE);
        assert_eq!(rv.unwrap(), gst::StateChangeSuccess::Success);
        if state == gst::State::Playing {
            self.playbin.set_state(gst::State::Paused).unwrap();
        }
    }

    pub fn play(&self) {
        if let Err(_) = self.playbin.set_state(gst::State::Playing) {
            display_an_error_dialog(
                Some(&(self.video_widget.get_toplevel().unwrap().downcast::<gtk::Window>().unwrap())),
                "Could not set play state, perhaps the aerial isn't connected?\n\nTry running with 'GST_DEBUG=3 me-tv' for details."
            );
        }
        /*
         * Add writing out the GStreamer pipeline to the event queue, but leave long
         * enough for the pipeline to be formed.
         *
         * Comment out for now.
         * /
        glib::timeout_add_seconds_local(8, {
            let the_bin = self.playbin.clone().downcast::<gst::Bin>().unwrap();
            move || {
                gst::debug_bin_to_dot_file(&the_bin, gst::DebugGraphDetails::all(), "pipeline");
                println!("££££££££  Pipeline diagram drawn.");
                Continue(false)
            }
        });
        / * */
    }

    pub fn stop(&self) {
        self.playbin.set_state(gst::State::Null).unwrap();
    }

    pub fn get_volume(&self) -> f64 {
        self.playbin.get_property("volume").unwrap().get().unwrap().unwrap()
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
