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

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use channel_names::encode_to_mrl;
use control_window::ControlWindow;
use frontend_manager::{FrontendId, TuningId};
use frontend_window::FrontendWindow;

use comboboxtext_extras::ComboBoxTextExtras;

/// A `ControlWindowButton` is a `gtk::Box` but there is no inheritance so use
/// a bit of composition.
pub struct ControlWindowButton {
    control_window: Rc<ControlWindow>,
    pub tuning_id: TuningId,
    pub widget: gtk::Box,
    frontend_button: gtk::ToggleButton,
    channel_selector: gtk::ComboBoxText,
    inhibitor: Cell<u32>,
    frontend_window: RefCell<Option<Rc<FrontendWindow>>>,
}

impl ControlWindowButton {
    /// Construct a new button representing an available front end.
    ///
    /// The adapter and frontend numbers for the label for a toggle button that is used
    /// to start and stop a frontend window displaying the stream for that frontend. Below
    /// is a drop down list button to select the channel to tune the front end to.
    ///
    /// This function is executed in the GTK event loop thread.
    pub fn new(control_window: &Rc<ControlWindow>, fei: FrontendId) -> Rc<ControlWindowButton> {
        let tuning_id = TuningId {
            frontend: fei,
            channel: RefCell::new(None),
        };
        let frontend_button = gtk::ToggleButton::new_with_label(
            format!("adaptor{}\nfrontend{}", tuning_id.frontend.adapter, tuning_id.frontend.frontend).as_ref()
        );
        let channel_selector = gtk::ComboBoxText::new();
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.pack_start(&frontend_button, true, true, 0);
        widget.pack_start(&channel_selector, true, true, 0);
        let cwb = Rc::new(ControlWindowButton {
            control_window: control_window.clone(),
            tuning_id,
            widget,
            frontend_button,
            channel_selector,
            inhibitor: Cell::new(0),
            frontend_window: RefCell::new(None),
        });
        cwb.fill_channel_list(&control_window);
        cwb.channel_selector.connect_changed({
            let c_w_b = cwb.clone();
            move |_| Self::on_channel_changed(&c_w_b, &c_w_b.channel_selector.get_active_text().unwrap())
        });
        cwb.frontend_button.connect_toggled({
            let c_w_b = cwb.clone();
            move |_| {
                if let Some(ref channel_names) = *c_w_b.control_window.channel_names.borrow() {
                    if channel_names.len() > 0 {
                        Self::toggle_button(&c_w_b);
                    }
                } else {
                    let dialog = gtk::MessageDialog::new(
                        Some(&c_w_b.control_window.window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Info,
                        gtk::ButtonsType::Ok,
                        "No channel file, so no channel list, so cannot play a channel.");
                    dialog.run();
                    dialog.destroy();
                    //button.set_active(false); // TODO causes the reissuing of the signal. :-(
                }
            }
        });
        cwb
    }

    /// Transfer the list of channel names held by the control window into the selector box and set the default.
    pub fn fill_channel_list(&self, control_window: &Rc<ControlWindow>) {
        match *control_window.channel_names.borrow() {
            Some(ref channel_names) => {
                for name in channel_names {
                    self.channel_selector.append_text(&name);
                };
                if let Some(ref default_channel_name) = *control_window.default_channel_name.borrow() {
                    self.tuning_id.channel.replace(Some(default_channel_name.clone()));
                    self.set_label(default_channel_name);
                }
            },
            None => {
                self.channel_selector.insert_text(0, "No channels file.");
                self.channel_selector.set_active(0);
            },
        }
    }

    /// Set the state of the channel control widgets.
    fn set_label(&self, channel_name: &String) {
        let current = &self.tuning_id.channel;
        let name = match *current.borrow() {
            Some(ref n) => n.clone(),
            None => "".to_string(),
        };
        if name != *channel_name {
            let previous_name = current.replace(Some(channel_name.clone()));
            assert_eq!(previous_name.unwrap(), name);
        }
        self.channel_selector.set_active_text(channel_name.as_ref());
        if let Some(ref frontend_window) = *self.frontend_window.borrow() {
            frontend_window.channel_selector.set_active_text(channel_name.as_ref());
            frontend_window.fullscreen_channel_selector.set_active_text(channel_name.as_ref());
        }
    }

    /// Toggle the button.
    ///
    /// This function is called after the change of state of the frontend_button.
    fn toggle_button(control_window_button: &Rc<ControlWindowButton>) {
        let application = control_window_button.control_window.window.get_application().unwrap();
        if control_window_button.frontend_button.get_active() {
            if let Some(ref channel_name) = *control_window_button.tuning_id.channel.borrow() {
                assert!((*control_window_button.frontend_window.borrow()).is_none());
                let frontend_window = Rc::new(FrontendWindow::new(
                    &application,
                    &control_window_button.control_window.channel_names,
                    &control_window_button.channel_selector.get_active_text().unwrap()));
                frontend_window.close_button.connect_clicked({
                    let button = control_window_button.frontend_button.clone();
                    move |_| button.set_active(!button.get_active())
                });
                frontend_window.volume_adjustment.connect_value_changed({
                    let f_w = frontend_window.clone();
                    move |_| f_w.engine.set_volume(f_w.volume_adjustment.get_value())
                });
                frontend_window.channel_selector.connect_changed({
                    let c_w_b = control_window_button.clone();
                    let f_w = frontend_window.clone();
                    move |_| Self::on_channel_changed(&c_w_b, &f_w.channel_selector.get_active_text().unwrap())
                });
                frontend_window.fullscreen_channel_selector.connect_changed({
                    let c_w_b = control_window_button.clone();
                    let f_w = frontend_window.clone();
                    move |_| Self::on_channel_changed(&c_w_b, &f_w.fullscreen_channel_selector.get_active_text().unwrap())
                });
                match control_window_button.frontend_window.replace(Some(frontend_window)) {
                    Some(_) => panic!("Inconsistent state of frontend,"),
                    None => {},
                };
                if control_window_button.inhibitor.get() == 0 {
                    if let Some(ref frontend_window) = *control_window_button.frontend_window.borrow() {
                        control_window_button.inhibitor.set(application.inhibit(&frontend_window.window, gtk::ApplicationInhibitFlags::SUSPEND, "Me TV inhibits when playing a channel."));
                        frontend_window.window.show();
                        frontend_window.engine.set_mrl(&encode_to_mrl(channel_name));
                        frontend_window.engine.play();
                    }
                } else {
                    panic!("Inconsistent state. Inhibitor error.");
                }
            }
        } else {
            if control_window_button.inhibitor.get() != 0 {
                application.uninhibit(control_window_button.inhibitor.get());
                control_window_button.inhibitor.set(0);
                if let Some(ref frontend_window) = *control_window_button.frontend_window.borrow() {
                    frontend_window.window.hide();
                    frontend_window.engine.stop();
                }
                match control_window_button.frontend_window.replace(None) {
                    Some(_) => {},
                    None => panic!("Inconsistent state of frontend,"),
                }
            } else {
                panic!("Inconsistent state. Inhibitor error.");
            }
        }
    }

    /// Callback for an observed channel change.
    fn on_channel_changed(control_window_button: &Rc<ControlWindowButton>, channel_name: &String) {
        let status = control_window_button.frontend_button.get_active();
        if let Some(ref frontend_window) = *control_window_button.frontend_window.borrow() {
            if status {
                frontend_window.engine.stop();
            }
            control_window_button.set_label(channel_name);
            frontend_window.engine.set_mrl(&encode_to_mrl(channel_name));
            if status {
                // TODO Must handle not being able to tune to a channel better than panicing.
                frontend_window.engine.play();
            }
        }
        control_window_button.tuning_id.channel.replace(Some(channel_name.clone()));
    }

}
