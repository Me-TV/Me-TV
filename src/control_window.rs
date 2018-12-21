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
use std::cell::{Cell, RefCell};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process;
use std::rc::Rc;
use std::thread;

use futures;
use futures::prelude::*;

use gio;
use gio::prelude::*;
use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

use tempfile;

use channel_names::{channels_file_path, get_names};
use control_window_button::ControlWindowButton;
use dialogs::display_an_error_dialog;
use frontend_manager::{FrontendId, Message};
use preferences;
use transmitter_dialog;

/// A `ControlWindow` is an `gtk::ApplicationWindow` but there is no inheritance
/// so use a bit of composition.
pub struct ControlWindow {
    pub window: gtk::ApplicationWindow, // main.rs needs this for putting application menus dialogues over this window.
    main_box: gtk::Box,
    frontends_box: gtk::Box,
    label: gtk::Label,
    pub channel_names_store: gtk::ListStore, // Used by ControlWindowButton and FrontendWindow.
    channel_names_loaded: Cell<bool>,
    control_window_buttons: RefCell<Vec<Rc<ControlWindowButton>>>,
}

impl ControlWindow {
    /// Constructor (obviously :-). Creates the window to hold the widgets representing the
    /// frontends available. It is assumed this is called in the main thread that then runs the
    /// GTK event loop.
    pub fn new(application: &gtk::Application, message_channel: futures::channel::mpsc::Receiver<Message>) -> Rc<ControlWindow> {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("Me TV");
        window.set_border_width(10);
        window.connect_delete_event({
            let a = application.clone();
            move |_, _| {
                a.quit();
                Inhibit(false)
            }
        });
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title("Me TV");
        header_bar.set_show_close_button(true);
        let menu_button = gtk::MenuButton::new();
        menu_button.set_image(&gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into()));
        let menu_builder = gtk::Builder::new_from_string(include_str!("resources/control_window_menu.xml"));
        let window_menu = menu_builder.get_object::<gio::Menu>("control_window_menu").unwrap();
        let epg_action = gio::SimpleAction::new("epg", None);
        window.add_action(&epg_action);
        let channels_file_action = gio::SimpleAction::new("create_channels_file", None);
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
        let control_window = Rc::new(ControlWindow {
            window,
            main_box,
            frontends_box,
            label,
            channel_names_store: gtk::ListStore::new(&[String::static_type()]),
            channel_names_loaded: Cell::new(false),
            control_window_buttons: RefCell::new(Vec::new()),
        });
        control_window.update_channels_store();
        epg_action.connect_activate({
            let c_w = control_window.clone();
            move |_, _| {
                display_an_error_dialog(Some(&c_w.window), if c_w.control_window_buttons.borrow().is_empty() {
                    "No frontends, so no EPG."
                } else {
                    "Should display the EPG window."  // TODO Get the EPG window working.
                });
            }
        });
        channels_file_action.connect_activate({
            let c_w = control_window.clone();
            move |_, _| {
                if c_w.control_window_buttons.borrow().is_empty() {
                    display_an_error_dialog(Some(&c_w.window),"No frontends, so no tuning possible.");
                } else {
                    ensure_channel_file_present(&c_w);
                }
            }
        });
        glib::MainContext::ref_thread_default().spawn_local({
            let c_w = control_window.clone();
            message_channel.for_each(move |message| {
                match message {
                    Message::FrontendAppeared { fei } => add_frontend(&c_w, &fei),
                    Message::FrontendDisappeared { fei } => remove_frontend(&c_w, &fei),
                }
                Ok(())
            }).map(|_| ())
        });
        control_window
    }

    /// Transfer the list of channel names held by the control window into the selector box and set the default.
    pub fn update_channels_store(&self) {
        self.channel_names_store.clear();
        match get_names() {
            Some(mut channel_names) => {
                channel_names.sort();
                for name in channel_names {
                    self.channel_names_store.insert_with_values(None, &[0], &[&name]);
                };
                self.channel_names_loaded.set(true);
            },
            None => {
                self.channel_names_store.insert_with_values(None, &[0], &[&"No channels file."]);
                self.channel_names_loaded.set(false);
            }
        }
        for button in self.control_window_buttons.borrow().iter() {
            button.reset_active_channel();
        }
    }

    pub fn is_channels_store_loaded(&self) -> bool { self.channel_names_loaded.get() }

}

/// Ensure that the GStreamer dvbsrc channels file is present.
///
/// If the transmitter files are not present this function will do nothing.
///
/// Currently try to use dvbv5-scan to create the file, or if it isn't present, try dvbscan or w_scan.
fn ensure_channel_file_present(control_window: &Rc<ControlWindow>) {
    match  transmitter_dialog::present(Some(&control_window.window)) {
        Some(path_to_transmitter_file) => {
            //  TODO Turn this into a dialog that follows the GNOME HIG. Probably best to create a custom dialog.
            let start_dialog = gtk::MessageDialog::new(
                Some(&control_window.window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Info,
                gtk::ButtonsType::OkCancel,   // TODO This button type is discourage by the GNOME HIG, incorrect button placements.
                &format!("Run:\n\n    dvbv5-scan {}\n\n?\n\nYou need to have already closed all open channel viewers for this to work.", path_to_transmitter_file.to_str().unwrap()),
            );
            let response = gtk::ResponseType::from(start_dialog.run());
            start_dialog.destroy();
            if response== gtk::ResponseType::Ok {
                let wait_dialog = gtk::MessageDialog::new(
                    Some(&control_window.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::None,
                    &format!("Running:\n\n    dvbv5-scan {}\n\nThis may take a while.", path_to_transmitter_file.to_str().unwrap())
                );
                wait_dialog.show_all();
                let (sender, mut receiver) = futures::channel::oneshot::channel::<bool>();
                glib::MainContext::ref_thread_default().spawn_local({
                    let c_w = control_window.clone();
                    let w_d = wait_dialog.clone();
                    receiver.map(move |result|{
                        w_d.destroy();
                        if result {
                            c_w.update_channels_store();
                        } else {
                            display_an_error_dialog(Some(&c_w.window), "dvbv5-scan failed to generate a file.");
                        }
                    }).map(|_| ()).map_err(|_| unreachable!())
                });
                thread::spawn({
                    let p_t_t_f = path_to_transmitter_file.clone();
                    move || {
                        let mut temporary_file = tempfile::NamedTempFile::new().expect("Could not create a temporary file.");
                        match process::Command::new("dvbv5-scan")
                            .arg("-o")
                            .arg(&temporary_file.path())
                            .arg(&p_t_t_f)
                            .output() {
                            Ok(_) => {
                                let mut destination = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open(channels_file_path())
                                    .expect("Could not open channels file.");
                                let mut buffer = String::new();
                                temporary_file.read_to_string(&mut buffer).expect("Could not read temporary channels file.");
                                destination.write(&buffer.as_bytes()).expect("Could not write channels file.");
                                sender.send(true).expect("Could not send result for some reason.")
                            },
                            Err(error) => sender.send(false).expect(&format!("Could not send result of error:{}", error)),
                        };
                    }
                });
            }
        },
        None => ()  // User already informed of problem.
    }
}

/// Add a new frontend to this control window.
fn add_frontend(control_window: &Rc<ControlWindow>, fei: &FrontendId) {
    if control_window.main_box.get_children()[0] == control_window.label {
        control_window.main_box.remove(&control_window.label);
        control_window.main_box.pack_start(&control_window.frontends_box, true, true, 0);
    }
    let control_window_button = ControlWindowButton::new(control_window, fei);
    let c_w_b = control_window_button.clone();
    control_window.frontends_box.pack_start(&control_window_button.widget, true, true, 0);
    control_window.control_window_buttons.borrow_mut().push(control_window_button);
    control_window.window.show_all();
    // TODO Why is the FrontendWindow positioned before the ControlWindow when showing  a default channel.
    let first_adapter_number = FrontendId{adapter: 0, frontend: 0};
    if *fei == first_adapter_number {
        if preferences::get_immediate_tv() {
            let tune_to_channel  = |target_channel_name_option: Option<String>|{
                match target_channel_name_option {
                    Some(target_channel_name) => {
                        if target_channel_name.is_empty() {
                            display_an_error_dialog(Some(&c_w_b.control_window.window), "The channel is the empty string and cannot be tuned to.");
                        } else {
                            // TODO What to do if None is returned?
                            if let Some(iterator) = control_window.channel_names_store.get_iter_first() {
                                loop {
                                    if let Some(channel_name) = control_window.channel_names_store.get_value(&iterator, 0).get::<String>() {
                                        if target_channel_name == channel_name {
                                            match control_window.channel_names_store.get_path(&iterator) {
                                                Some(mut tree_path) => {
                                                    let index = tree_path.get_indices_with_depth()[0];
                                                    if index < 0 { panic!("index cannot be a negative integer"); }
                                                    c_w_b.channel_selector.set_active(index as u32);
                                                    c_w_b.frontend_button.set_active(true);
                                                },
                                                None => panic!("Failed to get the path of the iterator."),
                                            }
                                            break;
                                        }
                                    }
                                    if !control_window.channel_names_store.iter_next(&iterator) {
                                        display_an_error_dialog(Some(&c_w_b.control_window.window), &format!("The channel {} could not be found for immediate TV display.", target_channel_name));
                                        break;
                                    }
                                }
                            }
                        }
                    },
                    None => display_an_error_dialog(Some(&c_w_b.control_window.window), "There was no channel to tune to."),
                }
            };
            tune_to_channel(if preferences::get_use_last_channel() { preferences::get_last_channel() } else { preferences::get_default_channel() });
        }
    }
}

/// Remove the frontend from this control window.
fn remove_frontend(control_window: &Rc<ControlWindow>, fei: &FrontendId) {
    let mut remove_index = 0;
    for (index, control_window_button) in control_window.control_window_buttons.borrow().iter().enumerate() {
        if control_window_button.frontend_id == *fei {
            control_window.frontends_box.remove(&control_window_button.widget);
            remove_index = index;
            break;
        }
    }
    control_window.control_window_buttons.borrow_mut().remove(remove_index);
    if control_window.frontends_box.get_children().is_empty() {
        control_window.main_box.remove(&control_window.frontends_box);
        control_window.main_box.pack_start(&control_window.label, true, true, 0);
    }
    control_window.window.show_all();
}
