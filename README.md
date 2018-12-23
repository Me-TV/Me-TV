[![Travis-CI](https://travis-ci.org/Me-TV/Me-TV.svg?branch=master)](https://travis-ci.org/Me-TV/Me-TV)
[![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![Download](https://api.bintray.com/packages/russel/generic/Me_TV/images/download.svg?version=v3.0.6)
](https://bintray.com/russel/generic/Me_TV/v3.0.6/link)

# Me TV – it's TV for me computer

Me TV is a DVB – digital video broadcast – viewer based on GTK+3 and GStreamer.

*NB This is a rewrite of Me TV for GTK+3 and GStreamer using the Rust programming language and the gtk-rs and
gstreamer-rs bindings.*

The home page for previous versions of Me TV is [Me TV Home Page on Launchpad](http://launchpad.net/me-tv).

## Dependencies

To run Me TV, it is assumed you have the following installed:

1. GTK+3 3.18.0 or later.
2. GStreamer 1.12.0 or later for Me TV 3.0.x, or GStreamer 1.14.5 or later for Me TV 3.1.x or later.
3. The base, good, and bad GStreamer plugins – currently the DVB plugin is in the bad package; it started
there for historical reasons, and although it is not now bad, it is good, it has never migrated to the good
package. So to run Me TV on Debian you need the packages gstreamer1.0-plugins-base,
gstreamer1.0-plugins-good, and gstreamer1.0-plugins-bad. On Debian the GStreamer GTK and OpenGL plugins have
been separated out, so you will also need to install gstreamer1.0-gtk3 and gstreamer1.0-gl.

For building Me TV you need the development packages installed, not just the library packages. So on Debian
the libgtk-3-dev and the libgstreamer1.0-dev packages.

## Building (and Installing)

You need to either take a clone of this repository, or download a release tarball.  You will then need to
build the executable. This being a Rust program all build is handled using Cargo – though there is a
Meson/Ninja build for those who wish to use that, Rust and Cargo are still needed though, the Meson/Ninja
build just manages use of Cargo. Rust and Cargo may be packaged for your operating system, but most people
use [Rustup](https://rustup.rs/) to install Rust and Cargo so as to stay up to date with both.

Once you have Rust and Cargo installed, by whatever means, then in the directory of the Me TV project,
typing the command:

    cargo build

will create a debug build, add the `--release` option to the command line to get a release build.

Of course there is always the option of typing:

    cargo run

to build and also run the program if the build is successful. Again the `--release` option is available on
this command line – but see below about creating the channels configuration file, which must happen before
you can watch television using Me TV.

If you want to make use of the Meson/Ninja build you will need Meson and Ninja installed. Usually this is by
installing the relevant packages.  You will need to clone the Me TV Git repository or download a release
tarball just as above. If you then cd into the directory, and run:

    mkdir Build
    cd Build
    meson --prefx=$HOME/Built ..
    ninja

a build of Me TV using Cargo will happen. The `prefix` option to the meson command is only needed if you are
not going to install Me TV to the usual place. Once the build is complete:

    ninja install

will install Me TV.

## Setting up for watching DVB

Before being able to watch a television channel using Me TV, you must have a channels file for the
transmitter you are getting DVB signal from. This channels file is
$HOME/.config/gstreamer-1.0/dvb-channels.conf. It is assumed this file is in DVBv5 format, DVBv3 format
files will cause an error.

A way of creating this file from a running Me TV is available using the menu on the application window,
currently it requires the executable _dvbv5-scan_ be installed. On Debian Sid this is in package
_dvb-tools_, whereas on Fedora Rawhide it is in the package _v4l-utils_ – for some reason Debian splits out
the DVB tools from the V4L utils, whereas Fedora keeps them all together. To create the channels file you
will not only need _dvbv5-scan_ installed but also the transmitter data files. These are in the package
_dtv-scan-tables_ on both Debian and Fedora. However Debian installs them to /usr/share/dvb/dvb-t/ whereas
Fedora installs them to /usr/share/dvbv5/dvb-t/.

To have the channels file available before executing Me TV you can run _dvbv5-scan_ manually. For example:

    dvbv5-scan --output=~/.config/gstreamer-1.0/dvb-channels.conf /usr/share/dvb/dvb-t/uk-CrystalPalace

on a Debian system (Fedora puts the transmitter files in a slightly different place) will do the right thing
of you live in the Crystal Palace transmitter region in the UK. I suspect people will want to scan on their
local transmitter, in which case you should replace the uk-CrystalPalace with the name appropriate for the
location you are when you run Me TV.

If you do not have _dvb5-scan_ installed then best advice is to install it, preferably using the Linux
distribution package management. If that is not possible then _dvbscan_ or _w\_scan_ can be used to create
the needed file, but it must be in DVBv5 format, not DVBv3 format.

## Using Me TV

When started the initial screen of Me TV shows the frontends available or a message if none are
connected. It should show any new ones as they are connected – and remove ones as they are removed. Each
frontend button is a toggle button for the associated display for that frontend. Clicking on a frontend
button will start a new frame, tune the frontend to the channel that is shown, and start playing the
channel. Channels can be changed and there is a full screen capability.

Hopefully the UI is intuitive and gives a good UX. If not please feel free to submit an issue.

## Recording

The main Me TV program is a GUI for watching TV. With it come two command line programs,
_me-tv-record_ and _me-tv-schedule_. _me-tv-record_ records a named channel for a given period
to a named MPEG-4 file. The created files can be watched using Glide or Totem (or any other
viewer program that can play MPEG-4 files). _me-tv-schedule_ is a command line program for
setting up an execution of me-tv-record, i.e. it schedules recording a given channel for a given
duration outputting to a given file, starting at a given time in the future.

## NB

Me TV has, to date, only been tested for DVB-T and DVB-T2, none of the other formats (DVB-C, DVB-S, DVB-S2,
ATSC) have been tested. They should work, but… actual reports of success are needed before any claims can be
made.

## Licence

This code is licenced under GPLv3. [![Licence](https://www.gnu.org/graphics/gplv3-127x51.png)](https://www.gnu.org/licenses/gpl-3.0.en.html)
