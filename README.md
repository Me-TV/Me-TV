[![Travis-CI](https://travis-ci.org/Me-TV/Me-TV.svg?branch=master)](https://travis-ci.org/Me-TV/Me-TV)
[![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.txt)

# Me TV – it's TV for me computer

## A GTK+ client for watching and recording DVB.

*NB This is a rewrite of Me TV for GTK+3 and GStreamer using the Rust programming language and the gtk-rs and
gstreamer-rs bindings.*

Me TV is a digital television (DVB – digital video broadcast) viewer based on GTK+3 and GStreamer.

## Reporting Issues

[Me TV Home Page on Launchpad](http://launchpad.net/me-tv) remains the home page for Me TV versions 1.x and
2.x. This GitHub repository is for development and release of version 3.x – to gain access to the
variety of CI resources, and the like, which work with Git (especially GitHub, but also GitLab and BitBucket) but not
with Bazaar and Launchpad. So, if you have questions, bug reports, issues, etc. on Me TV versions 1.x and
2.x, please go to the [Me TV Home Page on Launchpad](http://launchpad.net/me-tv).

Anything to do with Me TV 3 such as repository forking, bug reports, pull requests, etc. should happen here
on GitHub.

## Building

_It is assumed you have GTK+3 3.10.0 or later and GStreamer 1.12.0 or later already installed._

_For building Me TV you need the development packages installed, not just the library packages. So on Debian the
libgtk-3-dev and the libgstreamer1.0-dev packages._

You need to either take a clone of this repository, or download a release tarball.  You will then need to
build the executable. This being a Rust program all build is handled using Cargo. Rust and Cargo may be
packaged for your operating system, but most people use [Rustup](https://rustup.rs/) to install Rust and
Cargo so as to stay up to date with both. Once you have Rust and Cargo installed, by whatever means, then in
the directory of the Me TV project, typing the command:

    cargo build

will create a debug build, add the `--release` option to the command line to get a release build.

Of course there is always the option of typing:

    cargo run

to build and also run the program if the build is successful. Again the `--release` option is available on
this command line – but see below about creating the channels configuration file, which must happen before
you can watch television using Me TV.

If you want to install, then as an example:

    cargo install --root=$HOME/Built

which will install the executable to $HOME/Built/bin.

## Setting up for watching DVB

_In order for GStreamer to work at run time, you have to have many of the plugins installed. There are base,
good, bad, and ugly plugin packages. Currently the DVB plugin is in the bad package – it started there for
historical reasons, and although it is not now bad, it is good, it has never migrated to the good
package. So to run Me TV on Debian you need the packages gstreamer1.0-plugins-base,
gstreamer1.0-plugins-good, and gstreamer1.0-plugins-bad._

Before being able to watch a television channel using Me TV, you must have a channels file for the
transmitter you are getting DVB signal from. This channels file is
$HOME/.config/gstreamer-1.0/dvb-channels.conf. It is assumed this file is in DVBv5 format, DVBv3 format
files will cause an error.

A way of creating this file from a running Me TV is available using the menu on the application window,
currently it requires the executable _dvbv5-scan_ be installed. On Debian Sid this is in package
_dvb-tools_, whereas on Fedora Rawhide it is in the package _v4l-utils_ – for some reason Debian splits out
the DVB tools from the V4L utils, whereas Fedora keeps them all together. To create the channels file you
will not only need _dvbv5-scan_ installed but also the transmitter data files. These are in the package
_dvb-scan-tables_ on both Debian and Fedora. However Debian installs them to /usr/share/dvb/dvb-t/ whereas
Fedora installs them to /usr/share/dvbv5/dvb-t/.

To have the channels file available before executing Me TV you can run _dvbv5-scan_ manually. For example:

    dvbv5-scan --output=~/.config/gstreamer-1.0/dvb-channels.conf /usr/share/dvb/dvb-t/uk-CrystalPalace

on a Debian system (remember Fedora puts the transmitter files in a slightly different place) will do the
right thing of you live in the Crystal Palace transmitter region in the UK. I suspect people will want to
scan on their local transmitter, in which case you should replace the uk-CrystalPalace with the name
appropriate for the location you are when you run Me TV.

If you do not have _dvb5-scan_ installed then best advice is to install it, preferably using the Linux
distribution package management. If that is not possible then _dvbscan_ or _w\_scan_ can be used to create
the needed file, but it must be in DVBv5 format, not DVBv3 format.

## Using Me TV

When started the initial screen of Me TV shows the frontends available or a message if none are connected. It
should show any new ones as they are connected – and remove ones as they are removed. Each frontend button
is a toggle button for the associated display for that frontend. Clicking on a frontend button will start a
new frame, tune the frontend to the channel that is shown, and start playing the channel. Channels can be
changed and there is a full screen capability.

Hopefully the UI is intuitive and gives a good UX. If not please feel free to submit an issue.

## NB

Me TV has, to date, only been tested for DVB-T and DVB-T2, none of the other formats (DVB-C, DVB-S, DVB-S2,
ATSC) have been tested. They should work, but…

## Licence

This code is licenced under GPLv3. [![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.txt)
