/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2017–2020  Russel Winder
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

use gtk;
use gtk::prelude::*;

use crate::channels_data::encode_to_mrl;
use crate::control_window::ControlWindow;
use crate::dialogs::display_an_error_dialog;
use crate::frontend_manager::FrontendId;
use crate::frontend_window::FrontendWindow;
use crate::input_event_codes;
use crate::metvcombobox::{MeTVComboBox, MeTVComboBoxExt};
use crate::preferences;
use crate::remote_control::TargettedKeystroke;

/// A `ControlWindowButton` is a `gtk::Box` but there is no inheritance so use
/// a bit of composition.
#[derive(Debug)]
pub struct ControlWindowButton {
    pub control_window: Rc<ControlWindow>, // FrontendWindow instance needs access to this.
    pub frontend_id: FrontendId, // ControlWindow instance needs access to this for searching.
    pub widget: gtk::Box, // ControlWindow instance needs access to this for packing.
    pub frontend_button: gtk::ToggleButton, // FrontendWindow needs access to this.
    pub channel_selector: MeTVComboBox, // FrontendWindow needs read access to this.
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
    pub fn new(control_window: &Rc<ControlWindow>, fei: &FrontendId) -> Rc<ControlWindowButton> {
        let frontend_id = fei.clone();
        let frontend_button = gtk::ToggleButton::with_label(
            format!("adaptor{}\nfrontend{}", frontend_id.adapter, frontend_id.frontend).as_ref()
        );
        let channel_selector = MeTVComboBox::new_and_set_model(&control_window.channels_data_store);
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.pack_start(&frontend_button, true, true, 0);
        widget.pack_start(&channel_selector, true, true, 0);
        let control_window_button = Rc::new(ControlWindowButton {
            control_window: control_window.clone(),
            frontend_id,
            widget,
            frontend_button,
            channel_selector,
            frontend_window: RefCell::new(None),
        });
        control_window_button.reset_active_channel();
        control_window_button.channel_selector.connect_changed({
            let c_w_b = control_window_button.clone();
            move |_| Self::on_channel_changed(&c_w_b, c_w_b.channel_selector.get_active().unwrap())
        });
        control_window_button.frontend_button.connect_toggled({
            let c_w_b = control_window_button.clone();
            move |_| {
                if c_w_b.control_window.is_channels_store_loaded() {
                    Self::toggle_button(&c_w_b);
                } else {
                    display_an_error_dialog(Some(&c_w_b.control_window.window), "No channel file, so no channel list, so cannot play a channel.");
                }
            }
        });
        control_window_button
    }

    /// Set the active channel to 0.
    pub fn reset_active_channel(&self) {  // Used in control_window.rs
        self.channel_selector.set_active(Some(0));
        if let Some(ref frontend_window) = *self.frontend_window.borrow() {
            frontend_window.channel_selector.set_active(Some(0));
            frontend_window.fullscreen_channel_selector.set_active(Some(0));
        }
    }

    /// Set the state of all the channel control widgets.
    fn set_channel_index(&self, channel_index: u32) {
        let current = self.channel_selector.get_active().unwrap();
        if current != channel_index {
            self.channel_selector.set_active(Some(channel_index));
        }
        if let Some(ref frontend_window) = *self.frontend_window.borrow() {
            let fe_current = frontend_window.channel_selector.get_active().unwrap();
            if fe_current != channel_index {
                frontend_window.channel_selector.set_active(Some(channel_index));
            }
            let fs_fe_current = frontend_window.fullscreen_channel_selector.get_active().unwrap();
            if fs_fe_current != channel_index {
                frontend_window.fullscreen_channel_selector.set_active(Some(channel_index));
            }
        }
    }

    /// Toggle the button.
    ///
    /// This function is called after the change of state of the frontend_button.
    fn toggle_button(control_window_button: &Rc<ControlWindowButton>) { // Used in control_window.rs
        if control_window_button.frontend_button.get_active() {
            if control_window_button.control_window.is_channels_store_loaded() {
                let frontend_window = match FrontendWindow::new(control_window_button.clone()) {
                    Ok(frontend_window) => frontend_window,
                    Err(_) => {
                        display_an_error_dialog(Some(&control_window_button.control_window.window), "Could not create a frontend window, most likely because\na GStreamer engine could not be created.");
                        return;
                    },
                };
                match control_window_button.frontend_window.replace(Some(frontend_window)) {
                    Some(_) => panic!("Inconsistent state of frontend,"),
                    None => {},
                };
            }
            // TODO Should there be an else activity here?
        } else {
            match control_window_button.frontend_window.replace(None) {
                Some(ref frontend_window) => frontend_window.stop(),
                None => panic!("Inconsistent state of frontend,"),
            }
        }
    }

    /// Callback for an observed channel change.
    pub fn on_channel_changed(control_window_button: &Rc<ControlWindowButton>, channel_index: u32) { // Used in frontend_window.rs
        // TODO status is Option<u32> apparently which isn't a great bool value.
        let status = control_window_button.frontend_button.get_active();
        if let Some(ref frontend_window) = *control_window_button.frontend_window.borrow() {
            if status {
                // Do not stop the frontend completely just change what is being displayed on it.
                frontend_window.engine.stop();
                // TODO Need to clear the area in the gtk::DrawingArea or a gtk::GLArea
                //   to avoid keeping the last video frame when it is a switch to radio.
                //   See https://github.com/Me-TV/Me-TV/issues/29
                let w = frontend_window.engine.video_widget.clone();
                match w.clone().downcast::<gtk::DrawingArea>() {
                    Ok(_d) => {
                        // TODO Clear the background area.
                    },
                    Err(_) => {
                        match w.clone().downcast::<gtk::GLArea>() {
                            Ok(g) => {
                                let _c = g.get_context().unwrap();
                                // TODO Clear the background area.
                            },
                            Err(e) => panic!("Widget is neither gtk::DrawingArea or gtk::GLArea: {}", e),
                        }
                    },
                }
                println!("========  Channel changed callback called");
                // TODO Why does changing channel on the FrontendWindow result in three calls here.
            }
            control_window_button.set_channel_index(channel_index);
            let channel_name = control_window_button.channel_selector.get_active_text().unwrap();
            frontend_window.engine.set_mrl(&encode_to_mrl(&channel_name));
            preferences::set_last_channel(channel_name, true);
            if status {
                // TODO Must handle not being able to tune to a channel better than panicking.
                frontend_window.engine.play();
            }
        }
    }

    /// Process a targetted keystroke.
    pub fn process_targetted_keystroke(&self, tk: &TargettedKeystroke) {
        assert_eq!(self.frontend_id, tk.frontend_id);
        match tk.keystroke {
            input_event_codes::KEY_CHANNELUP => {
                if tk.value > 0 {
                    let selector = &self.channel_selector;
                    let index = selector.get_active().unwrap();
                    // TODO Need to stop going beyond the number of channels there are.
                    selector.set_active(Some(index + 1));
                }
            }
            input_event_codes::KEY_CHANNELDOWN => {
                if tk.value > 0 {
                    let selector = &self.channel_selector;
                    let index = selector.get_active().unwrap();
                    if index > 0 {
                        selector.set_active(Some(index - 1));
                    }
                }
            }
            input_event_codes::KEY_VOLUMEUP => {
                if tk.value > 0 {
                    if let Some(ref f_w) = *self.frontend_window.borrow() {
                        let button = &f_w.volume_button;
                        let volume = button.get_value();
                        let adjustment = button.get_adjustment();
                        let increment = adjustment.get_step_increment();
                        let maximum = adjustment.get_upper();
                        let new_volume = volume + increment;
                        if new_volume < maximum {
                            button.set_value(new_volume);
                        } else {
                            button.set_value(maximum);
                        }
                    }
                }
            },
            input_event_codes::KEY_VOLUMEDOWN => {
                if tk.value > 0 {
                    if let Some(ref f_w) = *self.frontend_window.borrow() {
                        let button = &f_w.volume_button;
                        let volume = button.get_value();
                        let adjustment = button.get_adjustment();
                        let increment = adjustment.get_step_increment();
                        let minimum = adjustment.get_lower();
                        let new_volume = volume - increment;
                        if new_volume > minimum {
                            button.set_value(new_volume);
                        } else {
                            button.set_value(minimum);
                        }
                    }
                }
            },
            input_event_codes::KEY_NUMERIC_0 ..= input_event_codes::KEY_NUMERIC_9 => {
                println!("Got an unprocessed numeric keystroke {}, {}", tk.keystroke, tk.value);
                // Remember there is a key down and key up event;
                // tk.value == 1 -> down, tk.value == 0 -> up.
            },
            x => println!("Got an unprocessed keystroke {}", x),
        }
    }

}
