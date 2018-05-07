[![Travis-CI](https://travis-ci.org/Me-TV/Me-TV.svg?branch=master)](https://travis-ci.org/Me-TV/Me-TV)
[![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.txt)

# Me TV – it's TV for me computer

## A GTK+ client for watching DVB.

*NB This is a rewrite of Me TV for GTK+3 and GStreamer using the Rust programming language and the gtk-rs and
gstreamer-rs bindings.*

Me TV is a digital television (DVB – digital video broadcast) viewer cased on GTK+3 and GStreamer.

## Reporting Issues

[Me TV Home Page on Launchpad](http://launchpad.net/me-tv) remains the home page for Me TV versions 1.x and
2.x. This GitHub repository is for development of version 3.x – to gain access to the
variety of CI resources, and the like, which work with Git (especially GitHub, but also GitLab and BitBucket) but not
with Bazaar and Launchpad. So, if you have questions, bug reports, issues, etc. on Me TV versions 1.x and
2.x, please go to the [Me TV Home Page on Launchpad](http://launchpad.net/me-tv).

Repository forking, bug reports, and pull requests, for this rewrite (version 3.x) should all happen here on
GitHub.

## Building

Assuming you have taken a clone of this repository, or downloaded a release tarball, you will need to compile
the project.  This being a Rust program all build is handled using Cargo.

    cargo build

will create a debug build, add the `--release` option to get a release build.

Of course there is always:

    cargo run

to build and run. Again the `--release` option is available – but see below about creating the channels
configuration file, which must happen before you can watch television channels using Me TV.

If you want to install, then as an example:

    cargo install --root=$HOME/Built

which will install the executable to $HOME/Built/bin.

## Setting up for watching DVB

It is assumed you have GTK+3 3.10.0 or later and GStreamer 1.12.0 or later installed. It is also assumed you
have the executable dvbv5-scan installed, but this is not totally necessary as there are workarounds.

Before being able to watch a television channel using Me TV you must have a channels file for the transmitter you
are getting DVB signal from. This channels file is $HOME/.config/gstreamer-1.0/dvb-channels.conf.

A way of creating this file from a running Me TV is available using the menu on the application window, currently it
requires dvbv5-scan.

To have the channels file available before executing Me TV you can run dvbv5-scan manually. For example:

    dvbv5-scan --output=~/.config/gstreamer-1.0/dvb-channels.conf /usr/share/dvb/dvb-t/uk-CrystalPalace

on a Debian system (Fedora puts the transmitter files in a different place) will do the right thing of you
live in the Crystal Palace transmitter region in the UK. I suspect people will want to scan on their local
transmitter, in this case you should replace the uk-CrystalPalace with the name appropriate for the location
you are when you run Me TV.

If you do not have dvb5-scan installed then best advice is to install it, preferably using the Linux
distribution package management. If that is not possible then dvbscan or w_scan can be used to create the
needed file, but it must be in v5 format, not v3 format.

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
