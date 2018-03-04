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

use control_window::ControlWindow;
use frontend_manager::{FrontendId, TuningId};
use frontend_window::FrontendWindow;
use gstreamer_engine::GStreamerEngine;

use comboboxtext_extras::ComboBoxTextExtras;

/// A `ControlWindowButton` is a `gtk::Box` but there is no inheritance so use
/// a bit of composition.
pub struct ControlWindowButton {
    control_window: Rc<ControlWindow>,
    tuning_id: TuningId,
    pub widget: gtk::Box,
    frontend_button: gtk::ToggleButton,
    channel_selector: gtk::ComboBoxText,
    inhibitor: Cell<u32>,
    frontend_window: FrontendWindow,
    engine: GStreamerEngine,
}

impl ControlWindowButton {
    /// Construct a new button representing an available front end.
    ///
    /// The adapter and frontend numbers for the label for a toggle button that is used
    /// to start and stop a frontend window displaying the stream for that frontend. Below
    /// is a drop down list button to select the channel to tune the front end to.
    ///
    /// This function is executed in the GTK event loop thread.
    pub fn new(control_window: &Rc<ControlWindow>, fei: FrontendId, channel_names: &Vec<String>, default_channel_name: &String) -> Rc<ControlWindowButton> {
        let tuning_id = TuningId { frontend: fei, channel: RefCell::new(default_channel_name.clone()) };
        let frontend_button = gtk::ToggleButton::new_with_label(format!("adaptor{}\nfrontend{}", tuning_id.frontend.adapter, tuning_id.frontend.frontend).as_ref());
        let channel_selector = gtk::ComboBoxText::new();
        for (_, name) in channel_names.iter().enumerate() {
            channel_selector.append_text(name);
        }
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.pack_start(&frontend_button, true, true, 0);
        widget.pack_start(&channel_selector, true, true, 0);
        let application = control_window.window.get_application().unwrap();
        let engine = GStreamerEngine::new(&application);
        let frontend_window = FrontendWindow::new(&application, &channel_names, &engine);
        let cwb = Rc::new(ControlWindowButton {
            control_window: control_window.clone(),
            tuning_id,
            widget,
            frontend_button,
            channel_selector,
            inhibitor: Cell::new(0),
            frontend_window,
            engine,
        });
        cwb.set_label(&cwb.tuning_id.channel.borrow());
        cwb.frontend_window.close_button.connect_clicked({
            let c_w_b = cwb.clone();
            move |_| {
                let button = &c_w_b.frontend_button;
                button.set_active(! button.get_active())
            }
        });
        cwb.frontend_window.volume_adjustment.connect_value_changed({
            let c_w_b = cwb.clone();
            move |_| {
                c_w_b.engine.set_volume(c_w_b.frontend_window.volume_adjustment.get_value());
            }
        });
        cwb.frontend_button.connect_toggled({
            let c_w_b = cwb.clone();
            move |_| c_w_b.toggle_button()
        });
        cwb.channel_selector.connect_changed({
            let c_w_b = cwb.clone();
            move |_| c_w_b.on_channel_changed(&c_w_b.channel_selector.get_active_text().unwrap())
        });
        cwb.frontend_window.channel_selector.connect_changed({
            let c_w_b = cwb.clone();
            move |_| c_w_b.on_channel_changed(&c_w_b.frontend_window.channel_selector.get_active_text().unwrap())
        });
        cwb.frontend_window.fullscreen_channel_selector.connect_changed({
            let c_w_b = cwb.clone();
            move |_| c_w_b.on_channel_changed(&c_w_b.frontend_window.fullscreen_channel_selector.get_active_text().unwrap())
        });
        cwb
    }

    /// Set the state of the channel control widgets..
    fn set_label(&self, channel_name: &String) {
        let current = &self.tuning_id.channel;
        if *current.borrow() != *channel_name {
            current.replace(channel_name.clone());
        }
        let value = &*current.borrow();
        self.channel_selector.set_active_text(value.as_ref());
        self.frontend_window.channel_selector.set_active_text(value.as_ref());
        self.frontend_window.fullscreen_channel_selector.set_active_text(value.as_ref());
    }

    /// Toggle the button.
    ///
    /// This function is called after the change of state of the frontend_button.
    fn toggle_button(&self) {
        let app = self.control_window.window.get_application().unwrap();
        if self.frontend_button.get_active() {
            if self.inhibitor.get() == 0 {
                self.inhibitor.set(app.inhibit(&self.frontend_window.window, gtk::ApplicationInhibitFlags::SUSPEND, "Me TV inhibits when playing a channel."));
                self.frontend_window.window.show();
                self.engine.set_mrl(&encode_to_mrl(&self.tuning_id.channel.borrow()));
                self.engine.play();

            } else {
                println!("Inconsistent state. Should panic in a nice multithreaded way.");
            }
        } else {
            if self.inhibitor.get() != 0 {
                app.uninhibit(self.inhibitor.get());
                self.inhibitor.set(0);
                self.frontend_window.window.hide();
                self.engine.pause();
            } else {
                println!("Inconsistent state. Should panic in a nice multithreaded way.");
            }
        }
    }

    /// Callback for an observed channel change.
    fn on_channel_changed(&self, channel_name: &String) {
        let status = self.frontend_button.get_active();
        if status {
            self.engine.stop();
        }
        self.set_label(channel_name);
        self.engine.set_mrl(&encode_to_mrl(&self.tuning_id.channel.borrow()));
        if status {
            self.engine.play();
        }
    }

}

/// Encode a string as used for display to one suitable to be an MRL.
fn encode_to_mrl(channel_name: &String) -> String {
    "dvb://".to_owned() + &channel_name.replace(" ", "%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_to_mrl_with_no_spaces() {
        assert_eq!(encode_to_mrl(&"ITV".to_owned()), "dvb://ITV");
    }

    #[test]
    fn test_encode_to_mrl_with_one_space() {
        assert_eq!(encode_to_mrl(&"BBC NEWS".to_owned()), "dvb://BBC%20NEWS");
    }

    #[test]
    fn test_encode_to_mrl_with_two_spaces() {
        assert_eq!(encode_to_mrl(&"BBC One Lon".to_owned()), "dvb://BBC%20One%20Lon");
    }

}
