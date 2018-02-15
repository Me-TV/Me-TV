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

pub fn create_frontend_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Me TV");
    let header_bar = gtk::HeaderBar::new();
    header_bar.set_title("Me TV");
    header_bar.set_show_close_button(false);
    let menu_button = gtk::MenuButton::new();
    menu_button.set_image(&gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into()));
    let fullscreen_menu_button = gtk::MenuButton::new();
    fullscreen_menu_button.set_image(&gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into()));
    let builder = gtk::Builder::new_from_string(include_str!("resources/frontend_window_menu.xml"));
    let window_menu = builder.get_object::<gio::Menu>("window_menu").expect("Could not create the frontend window menu.");
    let fullscreen_action = gio::SimpleAction::new("fullscreen", None);
    fullscreen_action.connect_activate({
        let w = window.clone();
        move |_, _| { if is_fullscreen(&w) { w.unfullscreen(); } else { w.fullscreen(); } }
    });
    window.add_action(&fullscreen_action);
    menu_button.set_menu_model(&window_menu);
    header_bar.pack_end(&menu_button);
    window.set_titlebar(&header_bar);
    window
}

fn is_fullscreen(window: &gtk::ApplicationWindow) -> bool {
	false // window.get_state() & gdk::WINDOW_STATE_FULLSCREEN
}
