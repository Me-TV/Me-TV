/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017–2019  Russel Winder
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

use std::rc::Rc;
use std::time::{Duration, Instant};

// use glib;
use glib::prelude::*;
use gdk;
use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use crate::channel_names::encode_to_mrl;
use crate::control_window_button::ControlWindowButton;
use crate::epg_manager::EPGEvent;
use crate::gstreamer_engine::GStreamerEngine;
use crate::metvcomboboxtext::{MeTVComboBoxText, MeTVComboBoxTextExt};
use crate::preferences;

/// In fullscreen mode this holds the last time there was mouse movement
/// or key press activity: it is used to provide a timeout for hiding the fullscreen
/// control bar. In window mode this value should always be None.
//  NB  This is only ever accessed using the GUI thread so no multi-threading
//  protection needed. Rust does though require all access to be labelled unsafe.
static mut LAST_ACTIVITY_TIME: Option<Instant> = None;

/// A frontend window for rendering a television or radio channel.
#[derive(Debug)]
pub struct FrontendWindow {
    control_window_button: Rc<ControlWindowButton>,
    window: gtk::Window,
    pub close_button: gtk::Button, // ControlWindowButton instance needs access to this.
    fullscreen_button: gtk::Button,
    volume_adjustment: gtk::Adjustment,
    pub volume_button: gtk::VolumeButton,  // ControlWindowButton instance uses this.
    pub channel_selector: MeTVComboBoxText, // ControlWindowButton instance needs access to this.
    fullscreen_toolbar: gtk::Toolbar,
    fullscreen_unfullscreen_button: gtk::Button,
    fullscreen_volume_button: gtk::VolumeButton,
    pub fullscreen_channel_selector: MeTVComboBoxText, // ControlWindowButton instance needs access to this.
    inhibitor: u32,
    pub engine: GStreamerEngine, // ControlWindowButton instance needs access to this.
}

impl FrontendWindow {
    pub fn new(control_window_button: Rc<ControlWindowButton>) -> Result<Rc<FrontendWindow>, ()> {
        let engine = match GStreamerEngine::new(control_window_button.clone()) {
            Ok(engine) => engine,
            Err(_) => { return Err(()); },
        };
        let window = {
            let w = gtk::Window::new(gtk::WindowType::Toplevel);
            w.set_title("Me TV");
            w.set_default_size(480, 270);
            w
        };
        window.connect_delete_event({
            let button = control_window_button.frontend_button.clone();
            move |_, _| {
                button.set_active(!button.get_active());
                Inhibit(false)
            }
        });
        let close_button = {
            let c_b = gtk::Button::new();
            c_b.set_image(Some(&gtk::Image::new_from_icon_name(Some("window-close-symbolic"), gtk::IconSize::Button.into())));
            c_b.connect_clicked({
                let button = control_window_button.frontend_button.clone();
                move |_| button.set_active(!button.get_active())
            });
            c_b
        };
        let fullscreen_button = {
            let f_b = gtk::Button::new();
            f_b.set_image(Some(&gtk::Image::new_from_icon_name(Some("view-fullscreen-symbolic"), gtk::IconSize::Button.into())));
            // Can only set the fullscreen_button actions after fullscreen_toolbar has been defined.
            f_b
        };
        let volume_adjustment = gtk::Adjustment::new(0.2, 0.0, 1.0, 0.01, 0.05, 0.0);
        // Cannot clone engine so have to wait for construction of the frontend window
        // to be able to define the action associated with the volume_adjustment.
        let volume_button = gtk::VolumeButton::new();
        let channel_selector = {
            let c_s = MeTVComboBoxText::new_and_set_model(&control_window_button.control_window.channel_names_store);
            c_s.set_active(control_window_button.channel_selector.get_active());
            c_s.connect_changed({
                let c_w_b = control_window_button.clone();
                move |channel_selector| ControlWindowButton::on_channel_changed(&c_w_b, channel_selector.get_active().unwrap())
            });
            c_s
        };
        let header_bar = {
            let h_b = gtk::HeaderBar::new();
            h_b.set_title(Some("Me TV"));
            h_b.set_show_close_button(false);  // TODO Why have a special close button instead of the standard one?
            h_b.pack_end(&close_button);
            h_b.pack_end(&fullscreen_button);
            h_b.pack_end(&volume_button);
            h_b.pack_start(&channel_selector);
            h_b.show_all();
            h_b
        };
        window.set_titlebar(Some(&header_bar));
        let fullscreen_toolbar_builder = gtk::Builder::new_from_string(include_str!("resources/frontend_window_fullscreen_toolbar.glade.xml"));
        let fullscreen_toolbar = {
            let f_t = fullscreen_toolbar_builder.get_object::<gtk::Toolbar>("fullscreen_control_toolbar").unwrap();
            f_t.set_halign(gtk::Align::Baseline);
            f_t.set_valign(gtk::Align::Start);
            f_t.hide();
            f_t
        };
        fullscreen_button.connect_clicked({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_| {
                unsafe {
                    match LAST_ACTIVITY_TIME {
                        Some(_) => panic!("Last activity time should have been None."),
                        None => set_timeout(Some(Instant::now())),
                    }
                }
                hide_cursor(&w.clone().upcast::<gtk::Widget>());
                w.fullscreen();
                gtk::timeout_add_seconds(2, {
                    let ww = w.clone();
                    let ft = f_t.clone();
                    move || {
                        unsafe {
                            match LAST_ACTIVITY_TIME {
                                Some(last_activity_time) => {
                                    if Instant::now().duration_since(last_activity_time) > Duration::from_secs(5) {
                                        hide_cursor(&ww.clone().upcast::<gtk::Widget>());
                                        ft.hide();
                                    }
                                    Continue(true)
                                },
                                None => Continue(false)
                            }
                        }
                    }
                });
            }
        });
        let fullscreen_unfullscreen_button = {
            let f_u_b = fullscreen_toolbar_builder.get_object::<gtk::Button>("fullscreen_unfullscreen_button").unwrap();
            f_u_b.connect_clicked({
                let w = window.clone();
                let f_t = fullscreen_toolbar.clone();
                move |_| {
                    set_timeout(None);
                    f_t.hide();
                    w.unfullscreen();
                    show_cursor(&w.clone().upcast::<gtk::Widget>());
                }
            });
            f_u_b
        };
        let fullscreen_volume_button = {
            let f_v_b = fullscreen_toolbar_builder.get_object::<gtk::VolumeButton>("fullscreen_volume_button").unwrap();
            //  TODO Mouse clicks on the + and - icons seem to create timeouts, moving the mouse
            //    creates a timeout, but grabbing the slider and moving it appears not to cause a timeout.
            f_v_b.connect_event_after(|_, _| { add_timeout(); });
            f_v_b.get_popup().unwrap().connect_event_after(|_, _| { add_timeout(); });
            f_v_b
        };
        let fullscreen_channel_selector = {
            let mut f_c_s = fullscreen_toolbar_builder.get_object::<MeTVComboBoxText>("fullscreen_channel_selector").unwrap();
            f_c_s.set_new_model(&control_window_button.control_window.channel_names_store);
            f_c_s.set_active(control_window_button.channel_selector.get_active());
            f_c_s.connect_changed({
                let c_w_b = control_window_button.clone();
                move |f_c_s| ControlWindowButton::on_channel_changed(&c_w_b, f_c_s.get_active().unwrap())
            });
            //
            // TODO There appear to be no 'event-after' events posted for a ComboBox or it's child.
            //
            f_c_s.connect_event_after(|_, ev| {
                add_timeout();
                unsafe {
                    println!("Adding timeout from f_c_s: {:?}, {:?}, {:?}", Instant::now(), ev.get_event_type(), LAST_ACTIVITY_TIME);
                };
            });
            f_c_s.get_child().unwrap().connect_event_after(|_, ev| {
                add_timeout();
                unsafe {
                    println!("Adding timeout from f_c_s child: {:?}, {:?}, {:?}", Instant::now(), ev.get_event_type(), LAST_ACTIVITY_TIME);
                };
            });
            /*
            f_c_s.connect_realize(|f| {
                println!("============  f_c_s realized.");
                if let Some(w) = f.get_window() {
                    println!("======== got a window.");
                    let events = w.get_events();
                    w.set_events(events | gdk::EventMask::KEY_PRESS_MASK | gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::SCROLL_MASK);
                    f.connect_event_after(|_, ev| {
                        add_timeout();
                        unsafe {
                            println!("Adding timeout from f: {:?}, {:?}, {:?}", Instant::now(), ev.get_event_type(), LAST_ACTIVITY_TIME);
                        };
                    });
                }
            });
            if let Some(c) = f_c_s.get_child() {
                println!("====== Got the f_c_s child.");
                c.connect_realize(|cc| {
                    println!("============ f_c_s child got realized");
                    if let Some(cc_w) = cc.get_window() {
                        println!("====== Got a f_c_s child window.");
                        let events = cc_w.get_events();
                        cc_w.set_events(events | gdk::EventMask::ALL_EVENTS_MASK);
                        cc.connect_event_after(|_, ev| {
                            add_timeout();
                            unsafe {
                                println!("Adding timeout from f_c_s child: {:?}, {:?}, {:?}", Instant::now(), ev.get_event_type(), LAST_ACTIVITY_TIME);
                            };
                        });
                    }
                });
            };
            */
            f_c_s
        };
        let volume = volume_adjustment.get_value();
        volume_button.set_value(volume);
        fullscreen_volume_button.set_value(volume);
        volume_button.set_adjustment(&volume_adjustment);
        fullscreen_volume_button.set_adjustment(&volume_adjustment);
        let video_overlay = {
            let v_o = gtk::Overlay::new();
            v_o.add(&engine.video_widget);
            v_o.show_all();
            v_o.add_overlay(&fullscreen_toolbar);
            v_o
        };
        window.add(&video_overlay);
        window.add_events(gdk::EventMask::KEY_PRESS_MASK);
        window.connect_key_press_event({
            let f_t = fullscreen_toolbar.clone();
            move |a_w, key| {
                if key.get_keyval() == gdk::enums::key::Escape {
                    if a_w.get_window().unwrap().get_state().intersects(gdk::WindowState::FULLSCREEN) {
                        set_timeout(None);
                        f_t.hide();
                        a_w.unfullscreen();
                        show_cursor(&a_w.clone().upcast::<gtk::Widget>());
                    }
                } else {
                    if_fullscreen_show_toolbar_and_add_timeout(&a_w.clone().upcast::<gtk::Widget>(), &f_t);
                }
                Inhibit(false)
            }
        });
        window.add_events(gdk::EventMask::POINTER_MOTION_MASK);
        window.connect_motion_notify_event({
            let f_t = fullscreen_toolbar.clone();
            move |a_w, _| {
                if_fullscreen_show_toolbar_and_add_timeout(&a_w.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        let channel_name = control_window_button.channel_selector.get_active_text().unwrap();
        engine.set_mrl(&encode_to_mrl(&channel_name));
        engine.play();
        preferences::set_last_channel(channel_name, true);
        window.show();
        let inhibitor = control_window_button.control_window.window.get_application().unwrap().inhibit(
            Some(&window),
            gtk::ApplicationInhibitFlags::SUSPEND | gtk::ApplicationInhibitFlags::IDLE,
            Some("Me TV inhibits when playing a channel."),
        );
        if inhibitor == 0 {
            println!("Warning: could not set inhibitor.");
        }
        let frontend_window = Rc::new(FrontendWindow {
            control_window_button: control_window_button.clone(),
            window,
            close_button,
            fullscreen_button,
            volume_adjustment,
            volume_button,
            channel_selector,
            fullscreen_toolbar,
            fullscreen_unfullscreen_button,
            fullscreen_volume_button,
            fullscreen_channel_selector,
            inhibitor,
            engine,
        });
        frontend_window.volume_adjustment.connect_value_changed({
            let f_w = frontend_window.clone();
            move |v_a| f_w.engine.set_volume(v_a.get_value())
        });
        Ok(frontend_window)
    }

    pub fn stop(&self) {
        if self.inhibitor  != 0 {
            let application = self.control_window_button.control_window.window.get_application().unwrap();
            application.uninhibit(self.inhibitor);
        } else {
            println!("Warning: inhibitor was not set.");
        }
        self.window.hide();
        self.engine.stop();
    }
}

fn hide_cursor(widget: &gtk::Widget) {
    if let Some(window) = widget.get_window() {
        window.set_cursor(gdk::Cursor::new_from_name(&widget.get_display().unwrap(), "none").as_ref());
    }
}

fn show_cursor(widget: &gtk::Widget) {
    if let Some(window) = widget.get_window() {
       window.set_cursor(None::<&gdk::Cursor>);
    }
}

fn if_fullscreen_show_toolbar_and_add_timeout(w: &gtk::Widget, t: &gtk::Toolbar) {
    if w.get_window().unwrap().get_state().intersects(gdk::WindowState::FULLSCREEN) {
        show_toolbar_and_add_timeout(w, t);
    }
}

fn show_toolbar_and_add_timeout(w: &gtk::Widget, t: &gtk::Toolbar) {
    show_cursor(&w);
    t.show();
    add_timeout();
}

fn add_timeout() {
    unsafe {
        set_timeout(match LAST_ACTIVITY_TIME {
            Some(last_activity_time) => {
                let current_time = Instant::now();
                Some(if last_activity_time < current_time { current_time } else { last_activity_time })
            },
            None =>  Some(Instant::now()),
        });
    }
}

fn set_timeout(time: Option<Instant>) {
    unsafe {
        LAST_ACTIVITY_TIME = time;
    }
}
