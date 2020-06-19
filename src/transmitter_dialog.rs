/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018, 2019  Russel Winder
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

use std::fs;
use std::path;

use gtk;
use gtk::prelude::*;

use crate::dialogs::display_an_error_dialog;
use crate::dvb;
use crate::preferences;

struct TransmitterSelector {
    transmitter: gtk::ComboBoxText,
    dialog: gtk::Dialog,
}

/// Return the path to the directory of transmitter files if present.
/// On Fedora DVBT/DVBT2 files are in /usr/share/dvbv5/dvb-t
/// On Debian DVBT/DVBT2 files are in /usr/share/dvb/dvb-t
fn dvbt_transmitter_files_directory_path() -> Option<path::PathBuf> {
    let mut path = path::PathBuf::new();
    path.push("/usr");
    path.push("share");
    path.push("dvbv5");
    if ! path.is_dir() {
        path.pop();
        path.push("dvb");
    }
    path.push(match preferences::get_delivery_system() {
        dvb::DeliverySystem::ATSC => "atsc",
        dvb::DeliverySystem::DVBC_ANNEX_A => "atsc",
        dvb::DeliverySystem::DVBC_ANNEX_B => "dvb-c",
        dvb::DeliverySystem::DVBT => "dvb-t",
        dvb::DeliverySystem::DVBT2 => "dvb-t",
        dvb::DeliverySystem::ISDBT => "isdb-t",
    });
    if path.is_dir() { Some(path) }
    else { None }
}

/// Create a dialog to allow the user to select the transmitter file they wish to
/// generate a channels file for – if a list of transmitter files is present in the
/// directory presented as the location of them.
fn create(parent: Option<&gtk::ApplicationWindow>, transmitter_files_directory_path: &path::Path) -> Option<TransmitterSelector> {
    let dialog = gtk::Dialog::with_buttons(
        Some("Me TV Transmitter Chooser"),
        parent,
        gtk::DialogFlags::MODAL,
        &[],
    );
    let label = gtk::Label::new(Some("Select the transmitter\nyou get signal from."));
    let transmitter = gtk::ComboBoxText::new();
    let mut transmitter_files = match fs::read_dir(transmitter_files_directory_path) {
        Ok(iterator) => iterator.map(|item| item.unwrap().file_name().to_str().unwrap().to_string()).collect::<Vec<String>>(),
        Err(_) => return None
    };
    transmitter_files.sort();
    for name in transmitter_files {
        transmitter.append_text(&name)
    }
    transmitter.set_active(Some(0));
    let content_area = dialog.get_content_area();
    content_area.pack_start(&label, false, false, 10);
    // TODO Make the ComboBoxText more easily scrollable?
    content_area.pack_start(&transmitter, false, false, 10);
    dialog.show_all();
    Some(TransmitterSelector {
        transmitter,
        dialog,
    })
}

/// Present a dialog to the user to allow them to select the transmitter file to
/// use to scan to create a channels file.
///
/// Returns an `Option` with the path on success.
///
/// If there are problems finding a transmitter file, tell the user via message dialog
/// and return `None`.
pub fn present(parent: Option<&gtk::ApplicationWindow>) -> Option<path::PathBuf> {
    match dvbt_transmitter_files_directory_path() {
        Some(transmitter_files_directory_path) =>  match create(parent, transmitter_files_directory_path.as_path()) {
            Some(dialog) => {
                dialog.dialog.run();
                let mut path = transmitter_files_directory_path;
                path.push(dialog.transmitter.get_active_text().unwrap().as_str());
                unsafe { dialog.dialog.destroy(); }
                Some(path)
            },
            None => {
                display_an_error_dialog(parent, "There appear to be no transmitter files,\nperhaps the dtv-scan-tables package is not correctly installed.");
                None
            }
        },
        None => {
            display_an_error_dialog(parent, "There appear to be no transmitter files directory ,\nperhaps the dtv-scan-tables package is not installed.");
            None
        }
    }
}
