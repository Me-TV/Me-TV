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

extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate gdk_pixbuf;

extern crate gstreamer as gst;

extern crate inotify;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

//use std::env;
use std::thread;
use std::sync::mpsc::channel;

use gio::prelude::*;
use gtk::prelude::*;

mod about;
mod channel_names;
mod comboboxtext_extras;
mod control_window;
mod control_window_button;
mod frontend_manager;
mod frontend_window;
mod gstreamer_engine;
mod inotify_daemon;

#[cfg(not(test))]
fn main() {
    gst::init().unwrap();
    let application = gtk::Application::new("uk.org.russel.me-tv_rust", gio::ApplicationFlags::empty()).expect("Application creation failed.");
    glib::set_application_name("Me TV");
    /*
    application.connect_startup(|app|{
    });
    */
    application.connect_activate(|app|{
        // It seems that the application menu must be added before creating the control window.
        let menu_builder = gtk::Builder::new_from_string(include_str!("resources/application_menu.xml"));
        let application_menu = menu_builder.get_object::<gio::Menu>("application_menu").expect("Could not construct the application menu.");
        app.set_app_menu(&application_menu);
        let epg_action = gio::SimpleAction::new("EPG", None);
        app.add_action(&epg_action);
        let about_action = gio::SimpleAction::new("about", None);
        app.add_action(&about_action);
        let quit_action = gio::SimpleAction::new("quit", None);
        app.add_action(&quit_action);
        let control_window = control_window::ControlWindow::new(&app);
        epg_action.connect_activate(move |_, _| {});
        about_action.connect_activate(move |_, _| about::present(Some(&control_window.window)));
        quit_action.connect_activate({let a = app.clone(); move |_, _| a.quit()});
        let (to_fem, from_in) = channel::<inotify_daemon::Message>();
        let (to_cw, from_fem) = channel::<frontend_manager::Message>();
        thread::spawn(||{control_window::message_listener(from_fem)});
        thread::spawn(||{frontend_manager::run(from_in, to_cw)});
        thread::spawn(||{inotify_daemon::run(to_fem)});
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
    //let args = env::args().collect::<Vec<_>>();
    //let arguments: Vec<&str> = args.iter().map(String::as_ref).collect();
    //application.run(&arguments);
    // No point in passing arguments until argument processing is available.
    application.run(&[]);
}
