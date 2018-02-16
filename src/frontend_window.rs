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

//use gdk;
//use gdk::prelude::*;
use gio;
use gio::prelude::*;
//use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

//use channel_names;

use gstreamer_engine::GStreamerEngine;

pub struct FrontendWindow {
    pub window: gtk::ApplicationWindow,
    pub close_button: gtk::Button,
    pub fullscreen_button: gtk::Button,
    pub channel_selector: gtk::ComboBoxText,
}

impl FrontendWindow {
    pub fn new(application: &gtk::Application, engine: &GStreamerEngine) -> FrontendWindow {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("Me TV");
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(false);
        let close_button = gtk::Button::new();
        close_button.set_image(&gtk::Image::new_from_icon_name("window-close-symbolic", gtk::IconSize::Button.into()));
        // close_button action added by caller of this funciton.
        let fullscreen_button = gtk::Button::new();
        fullscreen_button.set_image(&gtk::Image::new_from_icon_name("view-fullscreen-symbolic", gtk::IconSize::Button.into()));
        fullscreen_button.connect_clicked({
            let w = window.clone();
            move |_| { w.fullscreen(); }
        });
        let channel_selector = gtk::ComboBoxText::new();
        header_bar.pack_end(&close_button);
        header_bar.pack_end(&fullscreen_button);
        header_bar.pack_start(&channel_selector);
        window.set_titlebar(&header_bar);
        FrontendWindow {
            window,
            close_button,
            fullscreen_button,
            channel_selector,
        }
    }
}
