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

use std::cell::Cell;
use std::sync::Mutex;

use lazy_static::lazy_static;

use gtk;
// Can't use:
//
//use gtk::prelude::*;
//
// since it leads to a resolution problem, so have to explicitly list the bits needed.
use gtk::AboutDialogExt;
use gtk::DialogExt;
use gtk::WidgetExt;
use gtk::GtkWindowExt;

use gdk_pixbuf::PixbufLoader;
use gdk_pixbuf::PixbufLoaderExt;

lazy_static! {
    static ref ABOUT: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
}

fn create() -> gtk::AboutDialog {
    let about = gtk::AboutDialog::new();
    let mut authors = Vec::<&str>::new();
    authors.push("Michael Lamothe <michael.lamothe@gmail.com>");
    authors.push("Russel Winder <russel@winder.org.uk>");
    let mut documentors = Vec::<&str>::new();
    about.set_authors(&authors);
    about.set_comments(Some("A digital television (DVB) viewer using GTK+3 and GStreamer."));
    about.set_copyright(Some("Copyright © 2010–2011  Michael Lamothe <michael.lamothe@gmail.com>\nCopyright © 2014, 2016–2019  Russel Winder <russel@winder.org.uk>"));
    about.set_documenters(&documentors);
    about.set_license(Some("This program is licenced under GNU General Public Licence (GPL) version 3."));
    let loader = PixbufLoader::new();
    loader.write(include_bytes!("resources/images/uk.org.winder.me-tv.png")).unwrap();
    loader.close().unwrap();
    let image = loader.get_pixbuf().unwrap();
    about.set_logo(Some(&image));
    about.set_translator_credits(Some(""));
    about.set_version(Some(env!("CARGO_PKG_VERSION")));
    about
}

/// Present an about dialog in a non-modal way, but only if one is not already displaying.
pub fn present(parent: Option<&gtk::ApplicationWindow>) {
    if let Ok(active) = ABOUT.lock() {
        if ! active.get() {
            let dialog = create();
            dialog.set_transient_for(parent);
            dialog.connect_response(move |d, _| {
                if let Ok(active) = ABOUT.lock() {
                    d.destroy();
                    active.set(false);
                }
            });
            dialog.show();
            active.set(true);
        }
    }
}
