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

use gdk;
//use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use gstreamer_engine::GStreamerEngine;

pub struct FrontendWindow {
    pub window: gtk::ApplicationWindow,
    pub close_button: gtk::Button,
    pub fullscreen_button: gtk::Button,
    pub volume_adjustment: gtk::Adjustment,
    pub volume_button: gtk::VolumeButton,
    pub channel_selector: gtk::ComboBoxText,
}

impl FrontendWindow {

    pub fn new(application: &gtk::Application, channel_names: &Vec<String>, engine: &GStreamerEngine) -> FrontendWindow {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("Me TV");
        window.set_default_size(480, 270);
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(false);
        let close_button = gtk::Button::new();
        close_button.set_image(&gtk::Image::new_from_icon_name("window-close-symbolic", gtk::IconSize::Button.into()));
        // close_button action added by caller of this function.
        let fullscreen_button = gtk::Button::new();
        fullscreen_button.set_image(&gtk::Image::new_from_icon_name("view-fullscreen-symbolic", gtk::IconSize::Button.into()));
        fullscreen_button.connect_clicked({
            let w = window.clone();
            move |_| { w.fullscreen(); }
        });
        let channel_selector = gtk::ComboBoxText::new();
        for (_, name) in channel_names.iter().enumerate() {
            channel_selector.append_text(name);
        }
        // With C++ had to set the volume of the button before setting the adjustment
        // to get the right icons.
        let volume_adjustment = gtk::Adjustment::new(0.2, 0.0, 1.0, 0.01, 0.05, 0.0);
        let volume = volume_adjustment.get_value();
        let volume_button = gtk::VolumeButton::new();
        volume_button.set_value(volume);
        volume_button.set_adjustment(&volume_adjustment);
        // Adjustment callback set in calling function.
        header_bar.pack_end(&close_button);
        header_bar.pack_end(&fullscreen_button);
        header_bar.pack_end(&volume_button);
        header_bar.pack_start(&channel_selector);
        window.set_titlebar(&header_bar);
        window.add(&engine.video_widget);
        window.add_events(gdk::EventMask::KEY_PRESS_MASK.bits() as i32);
        window.connect_key_press_event({
            let w = window.clone();
            move |_, key| {
                if key.get_keyval() == gdk::enums::key::Escape {
                    w.unfullscreen();
                }
                Inhibit(false)
            }
        });
        FrontendWindow {
            window,
            close_button,
            fullscreen_button,
            volume_adjustment,
            volume_button,
            channel_selector,
        }
    }

}
