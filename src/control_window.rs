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
use std::process;
use std::rc::Rc;

use futures;
use futures::prelude::*;
use futures::channel::mpsc::Receiver;

use gio;
use gio::prelude::*;
use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

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
    pub fn new(application: &gtk::Application, message_channel: Receiver<Message>) -> Rc<ControlWindow> {
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
                    "Should display the EPG window."
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
        let context = glib::MainContext::ref_thread_default();
        context.spawn_local({
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
///  If the transmitter files are not present this function will do nothing.
///
/// Currently try to use dvbv5-scan to create the file, or if it isn't present, try dvbscan or w_scan.
fn ensure_channel_file_present(control_window: &Rc<ControlWindow>) {
    match  transmitter_dialog::present(Some(&control_window.window)) {
        Some(path_to_transmitter_file) => {
            //  TODO Turn this into a dialog that follows the GNOME HIG. Probably best to create a custom dialog.
            let dialog = gtk::MessageDialog::new(
                Some(&control_window.window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Info,
                gtk::ButtonsType::OkCancel,   // TODO This button type is discourage by the GNOME HIG, incorrect button placements.
                "Run dvbv5-scan, this may take a while.");
            let return_code = dialog.run();
            if return_code == 0 {  // TODO  what is the response ID for OK and for Cancel?
                let context = glib::MainContext::ref_thread_default();
                context.block_on(
                    futures::future::lazy({
                        let p_t_t_f = path_to_transmitter_file.clone();
                        let d = dialog.clone();
                        move |_| {
                            let output = process::Command::new("dvbv5-scan")
                                .arg("-o")
                                .arg(channels_file_path())
                                .arg(p_t_t_f)
                                .output();
                            // TODO Show some form of activity during the scanning.
                            d.destroy();
                            output
                        }
                    }).then({
                        let c_w = control_window.clone();
                        move |output| {
                            match output {
                                Ok(_) => c_w.update_channels_store(),
                                Err(error) => display_an_error_dialog(Some(&c_w.window), &format!("dvbv5-scan failed to generate a file.\n{:?}", error)),
                            };
                            futures::future::ok::<(), ()>(())
                        }
                    })
                ).unwrap();
            } else {
                dialog.destroy();
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
            if let Some(target_channel) = preferences::get_default_channel() {
                if target_channel.is_empty() {
                    display_an_error_dialog(Some(&c_w_b.control_window.window), "The default channel is the empty string and cannot be tuned to.");
                } else {
                    if let Some(iterator) = control_window.channel_names_store.get_iter_first() {
                        loop {
                            if let Some(channel_name) = control_window.channel_names_store.get_value(&iterator, 0).get::<String>() {
                                if target_channel == channel_name {
                                    match control_window.channel_names_store.get_path(&iterator) {
                                        Some(mut tree_path) => {
                                            let index = tree_path.get_indices_with_depth()[0];
                                            c_w_b.channel_selector.set_active(index);
                                            c_w_b.frontend_button.set_active(true);
                                        },
                                        None => panic!("Failed to get the path of the iterator."),
                                    }
                                    break;
                                }
                            }
                            if !control_window.channel_names_store.iter_next(&iterator) {
                                display_an_error_dialog(Some(&c_w_b.control_window.window), "The default channel could not be found for immediate TV display.");
                                break;
                            }
                        }
                    }
                }
            }
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
