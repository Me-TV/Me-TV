/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017  Russel Winder
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

extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate gdk_pixbuf;

extern crate gstreamer;

extern crate inotify;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use std::env;
use std::thread;
use std::sync::mpsc::channel;

use gio::prelude::*;
use gtk::prelude::*;

mod about;
mod channel_names;
mod control_window;
mod frontend_manager;
mod frontend_window;
mod inotify_daemon;

#[cfg(not(test))]
fn main() {
    gstreamer::init().unwrap();
    let application = gtk::Application::new("uk.org.russel.me-tv", gio::ApplicationFlags::empty()).expect("Application creation failed.");
    glib::set_application_name("Me TV");
    application.connect_startup(|app|{
        let menu_builder = gtk::Builder::new_from_string(include_str!("resources/application_menu.xml"));
        let application_menu = menu_builder.get_object::<gio::Menu>("application_menu").expect("Could not construct the application menu.");
        app.set_app_menu(&application_menu);
        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(move |_, _| about::present(None));
        app.add_action(&about_action);
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate({let a = app.clone(); move |_, _| a.quit()});
        app.add_action(&quit_action);
    });
    application.connect_activate(|app|{
        control_window::create_and_attach(&app);
        let (to_fem, from_in) = channel::<inotify_daemon::Message>();
        thread::spawn(||{inotify_daemon::run(to_fem)});
        let (to_cw, from_fem) = channel::<frontend_manager::Message>();
        thread::spawn(||{frontend_manager::run(from_in, to_cw)});
        thread::spawn(||{control_window::message_listener(from_fem)});
    });
    /*
     * As at 2017-10-14, gtk-rs does not provide access to these signals.

    application.connect_handle_local_options(|app|{
    });
    application.connect_command_line(|app|{
    });
     */
    // Hack to get a &[&str] of the arguments, required by the gtk::Application::run function.
    // cf. https://users.rust-lang.org/t/vec-string-to-str/
    let args = env::args().collect::<Vec<_>>();
    let arguments: Vec<&str> = args.iter().map(String::as_ref).collect();
    application.run(&arguments);
}
