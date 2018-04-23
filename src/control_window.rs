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
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use gio;
use gio::prelude::*;
use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

use channel_names;
use control_window_button::ControlWindowButton;
use frontend_manager::{FrontendId, Message};
use transmitter_dialog;

/// A `ControlWindow` is an `gtk::ApplicationWindow` but there is no inheritance
/// so use a bit of composition.
pub struct ControlWindow {
    pub window: gtk::ApplicationWindow,
    main_box: gtk::Box,
    frontends_box: gtk::Box,
    label: gtk::Label,
    control_window_buttons: RefCell<Vec<Rc<ControlWindowButton>>>,
    pub channel_names: RefCell<Option<Vec<String>>>,
    pub default_channel_name: RefCell<Option<String>>,
}

thread_local!(
pub static CONTROL_WINDOW: RefCell<Option<Rc<ControlWindow>>> = RefCell::new(None)
);

impl ControlWindow {

    /// Constructor (obviously :-). Creates the window to hold the widgets representing the
    /// frontends available. It is assumed this is called in the main thread that then runs the
    /// GTK event loop.
    pub fn new(application: &gtk::Application) -> Rc<ControlWindow> {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("Me TV");
        window.set_border_width(10);
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(true);
        window.connect_delete_event({
            let a = application.clone();
            move |_, _| {
                a.quit();
                Inhibit(true)
            }
        });
        let menu_button = gtk::MenuButton::new();
        menu_button.set_image(&gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into()));
        let menu_builder = gtk::Builder::new_from_string(include_str!("resources/control_window_menu.xml"));
        let window_menu = menu_builder.get_object::<gio::Menu>("control_window_menu").unwrap();
        let epg_action = gio::SimpleAction::new("epg", None);
        epg_action.connect_activate(
            move |_, _| {
               CONTROL_WINDOW.with(|global| {
                   if let Some(ref control_window) = *global.borrow_mut() {
                       let message = if (*control_window.control_window_buttons.borrow()).len() >0 {
                           "Should display the EPG window."
                       } else {
                           "No frontends, so no EPG."
                       };
                       let dialog = gtk::MessageDialog::new(
                           Some(&control_window.window),
                           gtk::DialogFlags::MODAL,
                           gtk::MessageType::Info,
                           gtk::ButtonsType::Ok,
                           message
                       );
                       dialog.run();
                       dialog.destroy();
                   }
               });
            }
        );
        window.add_action(&epg_action);
        let channels_file_action = gio::SimpleAction::new("create_channels_file", None);
        channels_file_action.connect_activate(
            move |_, _| {
                CONTROL_WINDOW.with(|global| {
                    if let Some(ref control_window) = *global.borrow_mut() {
                        if (*control_window.control_window_buttons.borrow()).len() >0 {
                            ensure_channel_file_present(&control_window.window, true).expect("Something went wrong creating the channels file");
                        } else {
                            let dialog = gtk::MessageDialog::new(
                                Some(&control_window.window),
                                gtk::DialogFlags::MODAL,
                                gtk::MessageType::Info,
                                gtk::ButtonsType::Ok,
                                "No frontends, so no tuning possible.");
                            dialog.run();
                            dialog.destroy();
                        }
                    }
                })
            }
        );
        window.add_action(&channels_file_action);
        menu_button.set_menu_model(&window_menu);
        header_bar.pack_end(&menu_button);
        window.set_titlebar(&header_bar);
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new("\nNo frontends available.\n");
        let frontends_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        main_box.pack_start(&label, true, true, 0);
        window.add(&main_box);
        window.show_all();
        let control_window_buttons: RefCell<Vec<Rc<ControlWindowButton>>> = RefCell::new(Vec::new());
        let mut channel_names = channel_names::get_names();
        let default_channel_name = match channel_names {
            Some(ref mut vector) => {
                let result = Some(vector[0].clone());
                vector.sort();
                result
            },
            None => None,
        };
        let control_window = Rc::new(ControlWindow {
            window,
            main_box,
            frontends_box,
            label,
            control_window_buttons,
            channel_names: RefCell::new(channel_names),
            default_channel_name: RefCell::new(default_channel_name),
        });
        let rv = control_window.clone();
        CONTROL_WINDOW.with(|global| {
            *global.borrow_mut() = Some(control_window);
        });
        rv
    }

}

/// Ensure that the GStreamer dvbsrc channels file is present.
/// If the argument is `false` then exit if the file is present or try to create it if it isn't.
/// If the argument is `true` then always try to recreate it.
///
/// Currently try to use dvbv5-scan to create the file, or if it isn't present, try dvbscan or w_scan.
fn ensure_channel_file_present(control_window: &gtk::ApplicationWindow, force_rewrite: bool) -> Result<String, String> {
    let mut channels_file_path = channel_names::channels_file_path();
    let channels_file_path = channels_file_path.as_path();
    if !force_rewrite & channels_file_path.exists() {
        return Ok("Channels file exists.".to_string());
    }
    let path_to_transmitter_file = transmitter_dialog::present(Some(&control_window));
    match Command::new("dvbv5-scan")
        .arg("-o")
        .arg(channel_names::channels_file_path())
        .arg(path_to_transmitter_file)
        .output() {
        Ok(_) => {
            Ok("Should be all done now.".to_string())
        },
        Err(_) => Err("Scan command failed.".to_string())
    }
}

/// Add a new frontend to this control window.
fn add_frontend(control_window: &Rc<ControlWindow>, fei: FrontendId) {
    if control_window.main_box.get_children()[0] == control_window.label {
        control_window.main_box.remove(&control_window.label);
        control_window.main_box.pack_start(&control_window.frontends_box, true, true, 0);
    }
    let control_window_button = ControlWindowButton::new(control_window, fei);
    control_window.frontends_box.pack_start(&control_window_button.widget, true, true, 0);
    control_window.control_window_buttons.borrow_mut().push(control_window_button);
    control_window.window.show_all();
}

/// Remove all the frontends of an adaptor from this control window.
fn remove_adapter(control_window: &Rc<ControlWindow>, id: u16) {
    //
    // TODO Get this working.
    // Must remove the ControlWindowButton widget  from ControlWindow frontend box
    // and the ControlWindowButton object from the Vec of object in the ControlWindow.
    //
    if control_window.frontends_box.get_children().is_empty() {
        control_window.main_box.remove(&control_window.frontends_box);
        control_window.main_box.pack_start(&control_window.label, true, true, 0);
    }
    control_window.window.show_all();
}

/// Put a call on the GTK event loop to add a new frontend.
fn handle_add_frontend(fei: FrontendId) {
    CONTROL_WINDOW.with(|global| {
        if let Some(ref mut control_window) = *global.borrow_mut() {
            add_frontend(control_window, fei);
        }
    })
}

/// Put a call on the GTK event loop to remove all the frontends of an adaptor.
fn handle_remove_adapter(id: u16) {
    CONTROL_WINDOW.with(|global|{
        if let Some(ref mut control_window) = *global.borrow_mut() {
            remove_adapter(control_window, id);
        }
    })
}

/// This function, running in it's own thread, receives messages from elsewhere and
/// schedules events on the GTK event loop thread to realise the requests.
pub fn message_listener(from_fem: Receiver<Message>) {
    loop {
        match from_fem.recv() {
            Ok(message) => {
                match message {
                    Message::FrontendAppeared{fei} => {
                        glib::idle_add(move ||{ handle_add_frontend(fei.clone()); glib::Continue(false) });
                    },
                    Message::AdapterDisappeared{id} => {
                        glib::idle_add( move||{ handle_remove_adapter(id); glib::Continue(false) });
                    },
                }
            },
            Err(_) => { println!("Control Window Listener has stopped."); },
        }
    }
}
