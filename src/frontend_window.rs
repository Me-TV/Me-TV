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

use gdk;
use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use channel_names::encode_to_mrl;
use control_window_button::ControlWindowButton;
use gstreamer_engine::GStreamerEngine;
use metvcomboboxtext::{MeTVComboBoxText, MeTVComboBoxTextExt};

pub struct FrontendWindow {
    control_window_button: Rc<ControlWindowButton>,
    window: gtk::ApplicationWindow,
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

    pub fn new(control_window_button: &Rc<ControlWindowButton>) -> Rc<FrontendWindow> {
        let application = control_window_button.control_window.window.get_application().unwrap();
        let engine = GStreamerEngine::new(&application);
        let window = gtk::ApplicationWindow::new(&application);
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
        fullscreen_button.connect_clicked({
            let w = window.clone();
            move |_| { w.fullscreen(); }
        });
        let channel_selector = MeTVComboBoxText::new_with_core_model(&control_window_button.control_window.channel_names_store);
        channel_selector.set_active(control_window_button.channel_selector.get_active());
        channel_selector.connect_changed({
            let c_w_b = control_window_button.clone();
            move |channel_selector| ControlWindowButton::on_channel_changed(&c_w_b, channel_selector.get_active())
        });
        let volume_adjustment = gtk::Adjustment::new(0.2, 0.0, 1.0, 0.01, 0.05, 0.0);
        // Call back defined at the end of this function.
        let volume_button = gtk::VolumeButton::new();
        let fullscreen_toolbar_builder = gtk::Builder::new_from_string(include_str!("resources/frontend_window_fullscreen_toolbar.glade.xml"));
        let fullscreen_toolbar = fullscreen_toolbar_builder.get_object::<gtk::Toolbar>("fullscreen_control_toolbar").unwrap();
        let fullscreen_unfullscreen_button = fullscreen_toolbar_builder.get_object::<gtk::Button>("fullscreen_unfullscreen_button").unwrap();
        fullscreen_unfullscreen_button.connect_clicked({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_| {
                f_t.hide();
                w.unfullscreen();
            }
        });
        let fullscreen_volume_button = fullscreen_toolbar_builder.get_object::<gtk::VolumeButton>("fullscreen_volume_button").unwrap();
        let mut fullscreen_channel_selector: MeTVComboBoxText = fullscreen_toolbar_builder.get_object::<gtk::ComboBox>("fullscreen_channel_selector").unwrap();
        fullscreen_channel_selector.set_with_core_model(&control_window_button.control_window.channel_names_store);
        fullscreen_channel_selector.set_active(control_window_button.channel_selector.get_active());
        fullscreen_channel_selector.connect_changed({
            let c_w_b = control_window_button.clone();
            move |f_c_s| ControlWindowButton::on_channel_changed(&c_w_b, f_c_s.get_active())
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
        fullscreen_toolbar.set_halign(gtk::Align::Baseline);
        fullscreen_toolbar.set_valign(gtk::Align::Start);
        fullscreen_toolbar.hide();
        video_overlay.add_overlay(&fullscreen_toolbar);
        video_overlay.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        video_overlay.connect_motion_notify_event({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_, _| {
                if w.get_window().unwrap().get_state().intersects(gdk::WindowState::FULLSCREEN) {
                    f_t.show();
                }
                gtk::timeout_add_seconds(5, {
                    let ft = f_t.clone();
                    move || {
                        ft.hide();
                        Continue(false)
                    }
                });
                Inhibit(false)
            }
        });
        window.add(&video_overlay);
        window.add_events(gdk::EventMask::KEY_PRESS_MASK.bits() as i32);
        window.connect_key_press_event({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_, key| {
                if key.get_keyval() == gdk::enums::key::Escape {
                    f_t.hide();
                    w.unfullscreen();
                }
                Inhibit(false)
            }
        });
        engine.set_mrl(&encode_to_mrl(&control_window_button.channel_selector.get_active_text().unwrap()));
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
        frontend_window
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
