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

use std::rc::Rc;
use std::time::{Duration, Instant};

// use glib;
use glib::prelude::*;
use gdk;
use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use channel_names::encode_to_mrl;
use control_window_button::ControlWindowButton;
use gstreamer_engine::GStreamerEngine;
use metvcomboboxtext::{MeTVComboBoxText, MeTVComboBoxTextExt};
use preferences;

/// When in fullscreen mode this will hold the last time there was mouse movement
/// or key press activity. It is used to provide a timeout for hiding the fullscreen
/// control bar. In window mode this value should be None.
static mut LAST_ACTIVITY_TIME: Option<Instant> = None;

pub struct FrontendWindow {
    control_window_button: Rc<ControlWindowButton>,
    window: gtk::Window,
    pub close_button: gtk::Button, // ControlWindowButton instance needs access to this.
    fullscreen_button: gtk::Button,
    volume_adjustment: gtk::Adjustment,
    volume_button: gtk::VolumeButton,
    pub channel_selector: MeTVComboBoxText, // ControlWindowButton instance needs access to this.
    fullscreen_toolbar: gtk::Toolbar,
    fullscreen_unfullscreen_button: gtk::Button,
    fullscreen_volume_button: gtk::VolumeButton,
    pub fullscreen_channel_selector: MeTVComboBoxText, // ControlWindowButton instance needs access to this.
    inhibitor: u32,
    pub engine: GStreamerEngine, // ControlWindowButton instance needs access to this.
}

impl FrontendWindow {

    pub fn new(control_window_button: &Rc<ControlWindowButton>) -> Result<Rc<FrontendWindow>, ()> {
        let application = control_window_button.control_window.window.get_application().unwrap();
        let engine = match GStreamerEngine::new(&application, &control_window_button.frontend_id) {
            Ok(engine) => engine,
            Err(_) => { return Err(()); },
        };
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("Me TV");
        window.set_default_size(480, 270);
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(false);
        let close_button = gtk::Button::new();
        close_button.set_image(&gtk::Image::new_from_icon_name("window-close-symbolic", gtk::IconSize::Button.into()));
        close_button.connect_clicked({
            let button = control_window_button.frontend_button.clone();
            move |_| button.set_active(!button.get_active())
        });
        let fullscreen_button = gtk::Button::new();
        fullscreen_button.set_image(&gtk::Image::new_from_icon_name("view-fullscreen-symbolic", gtk::IconSize::Button.into()));
        // Can only set the fullscreen_button actions after fullscreen_toolbar has been defined.
        let channel_selector = MeTVComboBoxText::new_and_set_model(&control_window_button.control_window.channel_names_store);
        channel_selector.set_active(control_window_button.channel_selector.get_active());
        channel_selector.connect_changed({
            let c_w_b = control_window_button.clone();
            move |channel_selector| ControlWindowButton::on_channel_changed(&c_w_b, channel_selector.get_active())
        });
        let volume_adjustment = gtk::Adjustment::new(0.2, 0.0, 1.0, 0.01, 0.05, 0.0);
        // Cannot clone engine so have to wait for construction of the frontend window
        // to be able to define the action associated with the volume_adjustment.
        let volume_button = gtk::VolumeButton::new();
        let fullscreen_toolbar_builder = gtk::Builder::new_from_string(include_str!("resources/frontend_window_fullscreen_toolbar.glade.xml"));
        let fullscreen_toolbar = fullscreen_toolbar_builder.get_object::<gtk::Toolbar>("fullscreen_control_toolbar").unwrap();
        fullscreen_toolbar.set_halign(gtk::Align::Baseline);
        fullscreen_toolbar.set_valign(gtk::Align::Start);
        fullscreen_toolbar.hide();
        fullscreen_button.connect_clicked({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_| {
                unsafe {
                    match LAST_ACTIVITY_TIME {
                        Some(_) => panic!("Last activity time should have been None."),
                        None => LAST_ACTIVITY_TIME = Some(Instant::now()),
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
        let fullscreen_unfullscreen_button = fullscreen_toolbar_builder.get_object::<gtk::Button>("fullscreen_unfullscreen_button").unwrap();
        fullscreen_unfullscreen_button.connect_clicked({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_| {
                unsafe {
                    LAST_ACTIVITY_TIME = None;
                }
                f_t.hide();
                w.unfullscreen();
                show_cursor(&w.clone().upcast::<gtk::Widget>());
            }
        });
        let fullscreen_volume_button = fullscreen_toolbar_builder.get_object::<gtk::VolumeButton>("fullscreen_volume_button").unwrap();
        fullscreen_volume_button.add_events(gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::KEY_PRESS_MASK);
        fullscreen_volume_button.connect_key_press_event({
            let f_t = fullscreen_toolbar.clone();
            // TODO This never appears to get called.
            move |v_b, _| {
                println!("fullscreen volume button key press");
                show_toolbar_and_add_timeout(&v_b.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        fullscreen_volume_button.connect_motion_notify_event({
            let f_t = fullscreen_toolbar.clone();
            // TODO This appears to work
            move |v_b, _| {
                println!("fullscreen volume button mouse motion");
                show_toolbar_and_add_timeout(&v_b.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        let mut fullscreen_channel_selector = fullscreen_toolbar_builder.get_object::<MeTVComboBoxText>("fullscreen_channel_selector").unwrap();
        fullscreen_channel_selector.set_new_model(&control_window_button.control_window.channel_names_store);
        fullscreen_channel_selector.set_active(control_window_button.channel_selector.get_active());
        fullscreen_channel_selector.connect_changed({
            let c_w_b = control_window_button.clone();
            move |f_c_s| ControlWindowButton::on_channel_changed(&c_w_b, f_c_s.get_active())
        });
        fullscreen_channel_selector.add_events(gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::KEY_PRESS_MASK);
        fullscreen_channel_selector.connect_key_press_event({
            let f_t = fullscreen_toolbar.clone();
            // TODO this never appears to be called.
            move |c_b, _| {
                println!("fullscreen channel selector key press");
                show_toolbar_and_add_timeout(&c_b.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        fullscreen_channel_selector.connect_motion_notify_event({
            let f_t = fullscreen_toolbar.clone();
            // TODO this never appears to be called.
            move |c_b, _| {
                println!("fullscreen channel selector mouse move");
                show_toolbar_and_add_timeout(&c_b.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        let volume = volume_adjustment.get_value();
        volume_button.set_value(volume);
        fullscreen_volume_button.set_value(volume);
        volume_button.set_adjustment(&volume_adjustment);
        fullscreen_volume_button.set_adjustment(&volume_adjustment);
        header_bar.pack_end(&close_button);
        header_bar.pack_end(&fullscreen_button);
        header_bar.pack_end(&volume_button);
        header_bar.pack_start(&channel_selector);
        header_bar.show_all();
        window.set_titlebar(&header_bar);
        let video_overlay = gtk::Overlay::new();
        video_overlay.add(&engine.video_widget);
        video_overlay.show_all();
        video_overlay.add_overlay(&fullscreen_toolbar);
        window.add(&video_overlay);
        window.add_events(gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::KEY_PRESS_MASK);
        window.connect_key_press_event({
            let f_t = fullscreen_toolbar.clone();
            move |a_w, key| {
                if key.get_keyval() == gdk::enums::key::Escape {
                    if a_w.get_window().unwrap().get_state().intersects(gdk::WindowState::FULLSCREEN) {
                        unsafe {
                            LAST_ACTIVITY_TIME = None;
                        }
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
        window.connect_motion_notify_event({
            let f_t = fullscreen_toolbar.clone();
            move |a_w, _| {
                if_fullscreen_show_toolbar_and_add_timeout(&a_w.clone().upcast::<gtk::Widget>(), &f_t);
                Inhibit(false)
            }
        });
        let channel_name = control_window_button.channel_selector.get_active_text().unwrap();
        engine.set_mrl(&encode_to_mrl(&channel_name));
        preferences::set_last_channel(channel_name, true);
        engine.play();
        window.show();
        let application = control_window_button.control_window.window.get_application().unwrap();
        let inhibitor = application.inhibit(
            &window,
            gtk::ApplicationInhibitFlags::SUSPEND | gtk::ApplicationInhibitFlags::IDLE,
            "Me TV inhibits when playing a channel."
        );
        if inhibitor == 0 {
            println!("Warning: could not set inhibitor.");
        }
        let frontend_window = Rc::new(FrontendWindow {
            control_window_button: control_window_button.clone(),
            window,
            close_button,
            fullscreen_button,
            volume_adjustment: volume_adjustment.clone(),
            volume_button,
            channel_selector,
            fullscreen_toolbar,
            fullscreen_unfullscreen_button,
            fullscreen_volume_button,
            fullscreen_channel_selector,
            inhibitor,
            engine,
        });
        volume_adjustment.connect_value_changed({
            let f_w = frontend_window.clone();
            move |v_a| f_w.engine.set_volume(v_a.get_value())
        });
        Ok(frontend_window)
    }

    pub fn stop(&self) {
        if self.inhibitor  != 0 {
            let application = self.window.get_application().unwrap();
            application.uninhibit(self.inhibitor);
        } else {
            println!("Warning: inhibitor was not set.");
        }
        self.window.hide();
        self.engine.stop();
    }

}

fn hide_cursor(widget: &gtk::Widget) {
    widget.get_window().unwrap().set_cursor(Some(&gdk::Cursor::new_from_name(&widget.get_display().unwrap(), "none")));
}

fn show_cursor(widget: &gtk::Widget) {
    widget.get_window().unwrap().set_cursor(None);
}

fn if_fullscreen_show_toolbar_and_add_timeout(w: &gtk::Widget, t: &gtk::Toolbar) {
    if w.get_window().unwrap().get_state().intersects(gdk::WindowState::FULLSCREEN) {
        show_toolbar_and_add_timeout(w, t);
    }
}

fn show_toolbar_and_add_timeout(w: &gtk::Widget, t: &gtk::Toolbar) {
    show_cursor(&w);
    t.show();
    unsafe {
        LAST_ACTIVITY_TIME = Some(Instant::now());
    }
}
