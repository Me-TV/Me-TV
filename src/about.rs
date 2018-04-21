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

fn create() -> gtk::AboutDialog {
    let about = gtk::AboutDialog::new();
    let mut authors = Vec::<&str>::new();
    authors.push("Michael Lamothe <michael.lamothe@gmail.com>");
    authors.push("Russel Winder <russel@winder.org.uk>");
    let mut documentors = Vec::<&str>::new();
    about.set_authors(&authors);
    about.set_comments("Me TV is a Digital Television (DVB) viewer for GTK+3.");
    about.set_copyright("Copyright © 2010–2011  Michael Lamothe <michael.lamothe@gmail.com>\nCopyright © 2014, 2016–2018  Russel Winder <russel@winder.org.uk>");
    about.set_documenters(&documentors);
    about.set_license("This program is licenced under GNU General Public Licence (GPL) version 3.");
    let loader = PixbufLoader::new();
    loader.write(include_bytes!("resources/images/me-tv.png")).unwrap();
    loader.close().unwrap();
    let image = loader.get_pixbuf().unwrap();
    about.set_logo(Some(&image));
    about.set_name("Me TV");
    about.set_translator_credits("");
    about.set_version(env!("CARGO_PKG_VERSION"));
    about
}

pub fn present(parent: Option<&gtk::ApplicationWindow>) {
    // TODO Is there a way to create this once so as to show/hide instead of this create/destroy?
    // TODO lazy_static appears not to be able to handle the GTK stuff –
    // TODO gobject_sys::GObject does not support std::marker::Sync
    let dialog = create();
    dialog.set_transient_for(parent);
    dialog.run();
    dialog.destroy();
}
