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

#[cfg(not(test))]
use std::thread;

#[cfg(not(test))]
use clap;

#[cfg(not(test))]
use gio::prelude::*;
//#[cfg(not(test))]
//use gtk::prelude::*;

#[cfg(not(test))]
use gst;

mod about;
mod channel_names;
mod control_window;
mod control_window_button;
mod dialogs;
mod dvb;
mod frontend_manager;
mod frontend_window;
mod gstreamer_engine;
pub mod input_event_codes; // Make this module public to avoid all the unused warnings.
mod metvcomboboxtext;
mod preferences;
mod preferences_dialog;
mod remote_control;
mod transmitter_dialog;

#[cfg(not(test))]
fn main() {
    preferences::init();
    /*
     *  As at 2018-12-26 gtk-rs seems not to allow connecting to the GTK+ handle_local_options signal,
     *  though it does now allow connecting to the GTK+ command_line signal. Thus gtk-rs still does not
     *  provide a way of processing options locally using the GTK+ mechanisms. Since we want to be able to
     *  process command line prior to the primary instance being created and/or communicated with, we just
     *  use the clap package. C++/gtkmm and D/GtkD do this right, it is a pity Rust/gtk-rs does not.
     *
     *  This is not a huge burden since clap is needed anyway for me-tv-record and me-tv-schedule, the two
     *  command line programs that come with Me TV for doing recording.
     */
    let cli_matches = clap::App::new("Me TV")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A Digital Television (DVB) viewer using GTK+3 and GStreamer.")
        .arg(clap::Arg::with_name("no_gl")
            .long("no-gl")
            .help("Do not try to use OpenGL."))
        .get_matches();
    if cli_matches.is_present("no_gl") {
        preferences::set_use_opengl(false, false);
    }
    gst::init().unwrap();
    let application = gtk::Application::new(Some("uk.org.russel.me-tv"), gio::ApplicationFlags::empty()).expect("Application creation failed");
    glib::set_application_name("Me TV");
    application.connect_startup(move |app| {
        let (to_control_window, from_manager) = glib::MainContext::channel::<control_window::Message>(glib::PRIORITY_DEFAULT);
        //  TODO This variable is no longer used since the application menu was
        //    removed, but the `ControlWindow` instance must be created at this time.
        //    Or is there a better way of doing this?
        let _control_window = control_window::ControlWindow::new(&app, from_manager);
        thread::spawn({
            let t_c_w = to_control_window.clone();
            move ||{ frontend_manager::run(t_c_w); }
        });
    });
    // Get a glib-gio warning if activate is not handled.
    application.connect_activate(move |_| { });
    application.run(&[]);
}
