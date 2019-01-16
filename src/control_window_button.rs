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

use std::cell::RefCell;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use channel_names::encode_to_mrl;
use control_window::ControlWindow;
use dialogs::display_an_error_dialog;
use frontend_manager::FrontendId;
use frontend_window::FrontendWindow;
use metvcomboboxtext::{MeTVComboBoxText, MeTVComboBoxTextExt};
use preferences;

/// A `ControlWindowButton` is a `gtk::Box` but there is no inheritance so use
/// a bit of composition.
#[derive(Debug)]
pub struct ControlWindowButton {
    pub control_window: Rc<ControlWindow>, // FrontendWindow instance needs access to this.
    pub frontend_id: FrontendId, // ControlWindow instance needs access to this for searching.
    pub widget: gtk::Box, // ControlWindow instance needs access to this for packing.
    pub frontend_button: gtk::ToggleButton, // FrontendWindow needs access to this.
    pub channel_selector: MeTVComboBoxText, // FrontendWindow needs read access to this.
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
        let frontend_button = gtk::ToggleButton::new_with_label(
            format!("adaptor{}\nfrontend{}", frontend_id.adapter, frontend_id.frontend).as_ref()
        );
        let channel_selector = MeTVComboBoxText::new_and_set_model(&control_window.channel_names_store);
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
        self.channel_selector.set_active(0);
        if let Some(ref frontend_window) = *self.frontend_window.borrow() {
            frontend_window.channel_selector.set_active(0);
            frontend_window.fullscreen_channel_selector.set_active(0);
        }
    }

    /// Set the state of the channel control widgets.
    fn set_channel_index(&self, channel_index: u32) {
        let current = self.channel_selector.get_active().unwrap();
        if current != channel_index {
            self.channel_selector.set_active(channel_index);
            if let Some(ref frontend_window) = *self.frontend_window.borrow() {
                frontend_window.channel_selector.set_active(channel_index);
                frontend_window.fullscreen_channel_selector.set_active(channel_index);
            }
        }
    }

    /// Toggle the button.
    ///
    /// This function is called after the change of state of the frontend_button.
    fn toggle_button(control_window_button: &Rc<ControlWindowButton>) { // Used in control_window.rs
        if control_window_button.frontend_button.get_active() {
            if control_window_button.control_window.is_channels_store_loaded() {
                let frontend_window = match FrontendWindow::new(&control_window_button) {
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
        let status = control_window_button.frontend_button.get_active();
        if let Some(ref frontend_window) = *control_window_button.frontend_window.borrow() {
            if status {
                frontend_window.engine.stop();
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

}
