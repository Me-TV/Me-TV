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

use std::cell::RefCell;

use gdk;
use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use gstreamer_engine::GStreamerEngine;

pub struct FrontendWindow {
    pub window: gtk::ApplicationWindow,
    pub video_overlay: gtk::Overlay,
    pub close_button: gtk::Button,
    fullscreen_button: gtk::Button,
    pub volume_adjustment: gtk::Adjustment,
    volume_button: gtk::VolumeButton,
    pub channel_selector: gtk::ComboBoxText,
    pub fullscreen_toolbar: gtk::Toolbar,
    fullscreen_unfullscreen_button: gtk::Button,
    fullscreen_volume_button: gtk::VolumeButton,
    pub fullscreen_channel_selector: gtk::ComboBoxText,
    pub engine: GStreamerEngine,
}

impl FrontendWindow {

    pub fn new(application: &gtk::Application, channel_names: &RefCell<Option<Vec<String>>>) -> FrontendWindow {
        let engine = GStreamerEngine::new(&application);
        let window = gtk::ApplicationWindow::new(application);
        let video_overlay = gtk::Overlay::new();
        let header_bar = gtk::HeaderBar::new();
        let close_button = gtk::Button::new();
        let fullscreen_button = gtk::Button::new();
        let channel_selector = gtk::ComboBoxText::new();
        let volume_adjustment = gtk::Adjustment::new(0.2, 0.0, 1.0, 0.01, 0.05, 0.0);
        let volume_button = gtk::VolumeButton::new();
        let fullscreen_toolbar_builder = gtk::Builder::new_from_string(include_str!("resources/frontend_window_fullscreen_toolbar.glade.xml"));
        let fullscreen_toolbar = fullscreen_toolbar_builder.get_object::<gtk::Toolbar>("fullscreen_control_toolbar").unwrap();
        let fullscreen_unfullscreen_button = fullscreen_toolbar_builder.get_object::<gtk::Button>("fullscreen_unfullscreen_button").unwrap();
        let fullscreen_volume_button = fullscreen_toolbar_builder.get_object::<gtk::VolumeButton>("fullscreen_volume_button").unwrap();
        let fullscreen_channel_selector = fullscreen_toolbar_builder.get_object::<gtk::ComboBoxText>("fullscreen_channel_selector").unwrap();
        window.set_title("Me TV");
        window.set_default_size(480, 270);
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(false);
        close_button.set_image(&gtk::Image::new_from_icon_name("window-close-symbolic", gtk::IconSize::Button.into()));
        // close_button action added by caller of this function.
        fullscreen_button.set_image(&gtk::Image::new_from_icon_name("view-fullscreen-symbolic", gtk::IconSize::Button.into()));
        fullscreen_button.connect_clicked({
            let w = window.clone();
            move |_| { w.fullscreen(); }
        });
        fullscreen_unfullscreen_button.connect_clicked({
            let w = window.clone();
            let f_t = fullscreen_toolbar.clone();
            move |_| {
                f_t.hide();
                w.unfullscreen();
            }
        });
        match *channel_names.borrow() {
            Some(ref channel_names) => {
                for name in channel_names {
                    channel_selector.append_text(&name);
                    fullscreen_channel_selector.append_text(&name);
                }
            },
            None => {
                channel_selector.append_text("No channels file.");
                fullscreen_channel_selector.append_text("No channels file.");
            },
        }
        // Channel selector callbacks set in calling function.
        let volume = volume_adjustment.get_value();
        volume_button.set_value(volume);
        fullscreen_volume_button.set_value(volume);
        volume_button.set_adjustment(&volume_adjustment);
        fullscreen_volume_button.set_adjustment(&volume_adjustment);
        // Adjustment callback set in calling function.
        header_bar.pack_end(&close_button);
        header_bar.pack_end(&fullscreen_button);
        header_bar.pack_end(&volume_button);
        header_bar.pack_start(&channel_selector);
        header_bar.show_all();
        window.set_titlebar(&header_bar);
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
                Inhibit(true)
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
        FrontendWindow {
            window,
            video_overlay,
            close_button,
            fullscreen_button,
            volume_adjustment,
            volume_button,
            channel_selector,
            fullscreen_toolbar,
            fullscreen_unfullscreen_button,
            fullscreen_volume_button,
            fullscreen_channel_selector,
            engine,
        }
    }

}
