/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2018  Russel Winder
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

struct TransmitterSelector {
    transmitter: gtk::ComboBoxText,
    dialog: gtk::Dialog,
}

/// Return the path to the directory of DVB-T transmitter files.
/// On Fedora /usr/share/dvbv5/dvb-t
/// On Debian /usr/share/dvb/dvb-t
fn dvbt_transmitter_files_path() -> Option<path::PathBuf> {
    let mut path = path::PathBuf::new();
    path.push("/usr");
    path.push("share");
    path.push("dvbv5");
    if ! path.is_dir() {
        path.pop();
        path.push("dvb");
    }
    path.push("dvb-t");
    if path.is_dir() { Some(path) }
    else { None }
}

fn create(parent: Option<&gtk::ApplicationWindow>) -> Option<TransmitterSelector> {
    let transmitter_files_path = match dvbt_transmitter_files_path() {
        Some(transmitter_files_path) => transmitter_files_path,
        None => return None
    };
    let dialog = gtk::Dialog::new_with_buttons(
        Some("Me TV Transmitter Chooser"),
        parent,
        gtk::DialogFlags::MODAL,
        &[],
    );
    let label = gtk::Label::new("Select the transmitter\nyou get signal from.");
    let transmitter = gtk::ComboBoxText::new();
    let mut transmitter_files = match fs::read_dir(transmitter_files_path) {
        Ok(iterator) => iterator.map(|item| item.unwrap().file_name().to_str().unwrap().to_string()).collect::<Vec<String>>(),
        Err(_) => return None
    };
    transmitter_files.sort();
    for name in transmitter_files {
        transmitter.append_text(&name)
    }
    transmitter.set_active(0);
    let content_area = dialog.get_content_area();
    content_area.pack_start(&label, false, false, 10);
    content_area.pack_start(&transmitter, false, false, 10);
    dialog.show_all();
    Some(TransmitterSelector {
        transmitter,
        dialog,
    })
}

pub fn present(parent: Option<&gtk::ApplicationWindow>) -> Option<path::PathBuf> {
    match create(parent) {
        Some(dialog) => {
            dialog.dialog.run();
            let mut path = dvbt_transmitter_files_path().unwrap();
            path.push(dialog.transmitter.get_active_text().unwrap());
            dialog.dialog.destroy();
            Some(path)
        },
        None => {
            let dialog = gtk::MessageDialog::new(
                parent,
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Ok,
                "There appear to be no transmitter files ,\nperhaps the dvb-scan-tables package is not installed."
            );
            dialog.run();
            dialog.destroy();
            None
        }
    }
}
