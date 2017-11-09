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

use std::cell::RefCell;
use std::sync::mpsc::Receiver;

use gio;
use gio::prelude::*;
use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

use about;
use frontend_manager::{FrontendId, Message};
use frontend_window;
use channel_names;

struct ControlWindow {
    window: gtk::ApplicationWindow,
    main_box: gtk::Box,
    frontends_box: gtk::Box,
    label: gtk::Label,
    channel_names: Vec<String>,
    default_channel_name: String,
}

struct ControlWindowButton {
    widget: gtk::Box,
    frontend_button: gtk::ToggleButton,
    channel_selector: gtk::ComboBoxText,
    frontend_window: Option<frontend_window::FrontendWindow>,
}

thread_local!(
static CONTROL_WINDOW: RefCell<Option<ControlWindow>> = RefCell::new(None)
);

/// Constructor (obviously :-). Creates the window to hold the widgets representing the
/// frontends available. It is assumed this is called in the main thread that then runs the
/// GTK event loop.
pub fn create_and_attach(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Me TV");
    window.set_border_width(5);
    let header_bar = gtk::HeaderBar::new();
    header_bar.set_title("Me TV");
    header_bar.set_show_close_button(true);
    let menu_button = gtk::MenuButton::new();
    menu_button.set_image(&gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into()));
	let menu_builder = gtk::Builder::new_from_string(include_str!("resources/control_window_menu.xml"));
	let window_menu = menu_builder.get_object::<gio::Menu>("control_window_menu").expect("Could not create the control window menu.");
    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate({
        let w = window.clone();
        move |_, _| about::present(Some(&w))
    });
    window.add_action(&about_action);
    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate({
        let a = application.clone();
        move |_, _| a.quit()
    });
    window.add_action(&quit_action);
	menu_button.set_menu_model(&window_menu);
	header_bar.pack_end(&menu_button);
	window.set_titlebar(&header_bar);
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let label = gtk::Label::new("\n\nNo frontends available.");
    let frontends_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    main_box.add(&label);
    window.add(&main_box);
    window.show_all();
    gtk::Inhibit(true);
    let mut channel_names = channel_names::get_names();
    let default_channel_name = channel_names[0].clone();
    channel_names.sort();
    let rv = ControlWindow{
        window,
        main_box,
        frontends_box,
        label,
        channel_names,
        default_channel_name,
    };
    CONTROL_WINDOW.with(|global|{
        *global.borrow_mut() = Some(rv);
    });
}

/// Construct a new button representing an available front end.
///
/// The adapter and frontend numbers for the label for a toggle button that is used
/// to start and stop a frontend window displaying the stream for that frontend. Below
/// is a drop down list button to select the channel to tune the front end to.
///
/// This function is executed in the GTK event loop thread.
fn new_control_window_button(fei: FrontendId, channel_names: &Vec<String>, default_channel_name: &String) -> ControlWindowButton {
    let frontend_button = gtk::ToggleButton::new_with_label(format!("adaptor{}\nfrontend{}", fei.adapter, fei.frontend).as_ref());
    let channel_selector = gtk::ComboBoxText::new();
    for (i, name) in channel_names.iter().enumerate() {
        channel_selector.append_text(name);
        if name == default_channel_name {
            channel_selector.set_active(i as i32);
        }
    }
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
    widget.set_border_width(5);
    widget.add(&frontend_button);
    widget.add(&channel_selector);
    ControlWindowButton {
        widget,
        frontend_button,
        channel_selector,
        frontend_window: None,
    }
}

/// Add a new frontend. This function is executed in the GTK event loop thread.
fn handle_add_frontend(fei: FrontendId) {
    println!("handle_add_frontend executed.");
    CONTROL_WINDOW.with(|global| {
        if let Some(ref mut x) = *global.borrow_mut() {
            if x.main_box.get_children()[0] == x.label {
                x.main_box.remove(&x.label);
                x.main_box.add(&x.frontends_box);
            }
            let control_window_button = new_control_window_button(fei, &x.channel_names, &x.default_channel_name);
            control_window_button.frontend_button.connect_toggled({
                move |feb|{
                    //let few = frontend_window::new();
                }
            });
            x.frontends_box.add(&control_window_button.widget);
            x.window.show_all();
        }
    })
}

/// Remove a frontend. This function is executed in the GTK event loop thread.
fn handle_remove_adapter(id: u16) {
    println!("handle_remove_adapter executed.");
    CONTROL_WINDOW.with(|global|{
        if let Some(ref mut x) = *global.borrow_mut() {
            // Do remove.
            if x.frontends_box.get_children().is_empty() {
                x.main_box.remove(&x.frontends_box);
                x.main_box.add(&x.label);
            }
            x.window.show_all();
        }
    })
}

/// The function, running in it's own thread, that receives messages from elsewhere and
/// schedules events on the GTK event loop thread to realise the requests.
pub fn message_listener(from_fem: Receiver<Message>) {
    loop {
        match from_fem.recv() {
            Ok(r) => {
                match r {
                    Message::FrontendAppeared{fei} => {
                        println!("Frontend adapter{}:frontend{} appeared.", fei.adapter, fei.frontend);
                        glib::idle_add(move ||{handle_add_frontend(fei.clone()); glib::Continue(false)});
                    },
                    Message::AdapterDisappeared{id} => {
                        println!("Adapter {} disappeared.", id);
                        glib::idle_add( move||{handle_remove_adapter(id); glib::Continue(false)});
                    },
                }
            },
            Err(_) => {println!("Control Window Listener has stopped.");},
        }
    }
}
