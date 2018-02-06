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
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use gio;
use gio::prelude::*;
use glib;
//use glib::prelude::*;
use gtk;
use gtk::prelude::*;

use about;
use channel_names;
use frontend_manager::{FrontendId, TuningId, Message};
use frontend_window::FrontendWindow;

use comboboxtext_extras::ComboBoxTextExtras;

/// A `ControlWindow` is an `gtk::ApplicationWindow` but there is no inheritance
/// so use a bit of composition.
pub struct ControlWindow {
    pub window: gtk::ApplicationWindow,
    main_box: gtk::Box,
    frontends_box: gtk::Box,
    label: gtk::Label,
    control_window_buttons: Vec<Rc<ControlWindowButton>>,
    channel_names: Vec<String>,
    default_channel_name: String,
}

/// A `ControlWindowButton` is a `gtk::Box` but there is no inheritance so use
/// a bit of composition.
struct ControlWindowButton {
    control_window: Rc<ControlWindow>,
    tuning_id: TuningId,
    widget: gtk::Box,
    frontend_button: gtk::ToggleButton,
    channel_selector: gtk::ComboBoxText,
    inhibitor: u32,
    frontend_window: Rc<FrontendWindow>,
}

thread_local!(
static CONTROL_WINDOW: RefCell<Option<Rc<ControlWindow>>> = RefCell::new(None)
);

impl ControlWindow {
    /// Constructor (obviously :-). Creates the window to hold the widgets representing the
    /// frontends available. It is assumed this is called in the main thread that then runs the
    /// GTK event loop.
    pub fn new(application: &gtk::Application) -> Rc<ControlWindow> {
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
        // Temporary place holders. Need some better things in this menu.
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
        // End temporary place holder.
        menu_button.set_menu_model(&window_menu);
        header_bar.pack_end(&menu_button);
        window.set_titlebar(&header_bar);
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new("No frontends available.");
        let frontends_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        main_box.pack_start(&label, true, true, 0);
        window.add(&main_box);
        window.show_all();
        let control_window_buttons: Vec<Rc<ControlWindowButton>> = Vec::new();
        let mut channel_names = channel_names::get_names();
        let default_channel_name = channel_names[0].clone();
        channel_names.sort();
        let control_window = Rc::new(ControlWindow {
            window,
            main_box,
            frontends_box,
            label,
            control_window_buttons,
            channel_names,
            default_channel_name,
        });
        let rv = control_window.clone();
        CONTROL_WINDOW.with(|global| {
            *global.borrow_mut() = Some(control_window);
        });
        rv
    }
}

impl ControlWindowButton {
    /// Construct a new button representing an available front end.
    ///
    /// The adapter and frontend numbers for the label for a toggle button that is used
    /// to start and stop a frontend window displaying the stream for that frontend. Below
    /// is a drop down list button to select the channel to tune the front end to.
    ///
    /// This function is executed in the GTK event loop thread.
    fn new(control_window: &Rc<ControlWindow>, fei: FrontendId, channel_names: &Vec<String>, default_channel_name: &String) -> Rc<ControlWindowButton> {
        let tuning_id = TuningId{frontend: fei, channel: default_channel_name.clone()};
        let frontend_button = gtk::ToggleButton::new_with_label(format!("adaptor{}\nfrontend{}", tuning_id.frontend.adapter, tuning_id.frontend.frontend).as_ref());
        let channel_selector = gtk::ComboBoxText::new();
        for (_, name) in channel_names.iter().enumerate() {
            channel_selector.append_text(name);
        }
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.pack_start(&frontend_button, true, true, 0);
        widget.pack_start(&channel_selector, true, true, 0);
        let frontend_window = FrontendWindow::new(&control_window.window.get_application().unwrap());
        let cwb = Rc::new(ControlWindowButton {
            control_window: control_window.clone(),
            tuning_id: tuning_id.clone(),
            widget,
            frontend_button: frontend_button.clone(),
            channel_selector,
            inhibitor: 0,
            frontend_window,
        });
        cwb.set_label(&tuning_id.channel);
        frontend_button.connect_toggled({
            let c_w_b = cwb.clone();
            move |feb| {
                c_w_b.toggle_button();
            }
        });
        cwb
    }

    /// Set the state of the control window button.
    fn set_label(&self, channel_name: &String) {
        self.channel_selector.set_active_text(self.tuning_id.channel.as_ref());
    }

    /// Toggle the button.
    fn toggle_button(&mut self) {
        println!("Frontend window button toggled.");
        CONTROL_WINDOW.with(|global|{
             if let Some(ref control_window) = *global.borrow() {
                 let app = control_window.window.get_application().unwrap();
                 if self.frontend_button.get_active() {
                     println!("Active");
                     if self.inhibitor == 0 {
                         println!("Activating inactive window.");
                         self.inhibitor = app.inhibit(&self.frontend_window.window, gtk::ApplicationInhibitFlags::SUSPEND, "Me Tv inhibits when playing a channel.");
                     } else {
                         println!("Window being activated is already active.");
                     }
                 } else {
                     println!("Inactive");
                     if self.inhibitor != 0 {
                         println!("Deactivating active window.");
                         app.uninhibit(self.inhibitor);
                         self.inhibitor = 0;
                     } else {
                         println!("Window being deactivated is not active.");
                     }
                 }
             }
         });
    }
}

/// Add a new frontend to this control window.
fn add_frontend(control_window: &Rc<ControlWindow>, fei: FrontendId) {
    if control_window.main_box.get_children()[0] == control_window.label {
        control_window.main_box.remove(&control_window.label);
        control_window.main_box.pack_start(&control_window.frontends_box, true, true, 0);
    }
    let control_window_button = ControlWindowButton::new(control_window, fei, &control_window.channel_names, &control_window.default_channel_name);
    control_window.frontends_box.pack_start(&control_window_button.widget, true, true, 0);
    control_window.control_window_buttons.push(control_window_button);
    control_window.window.show_all();
}

/// Remove all the frontends of an adaptor from this control window.
fn remove_adapter(control_window: &Rc<ControlWindow>, id: u16) {
    // Do remove: must remove the ControlWindowButton widget  from ControlWindow frontend box
    // and the ControlWindowButton object from the Vec of object in the ControlWindow.
    if control_window.frontends_box.get_children().is_empty() {
        control_window.main_box.remove(&control_window.frontends_box);
        control_window.main_box.pack_start(&control_window.label, true, true, 0);
    }
    control_window.window.show_all();
}

/// Put a call on the GTK event loop to add a new frontend.
fn handle_add_frontend(fei: FrontendId) {
    println!("handle_add_frontend executed.");
    CONTROL_WINDOW.with(|global| {
        if let Some(ref mut control_window) = *global.borrow_mut() {
            add_frontend(control_window, fei);
        }
    })
}

/// Put a call on the GTK event loop to remove all the frontends of an adaptor.
fn handle_remove_adapter(id: u16) {
    println!("handle_remove_adapter executed.");
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
