[![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.txt)

# Me TV – it's TV for me computer

## A GTK+ client for watching DVB.

*NB This is a rewrite of Me TV for GTK+3 and GStreamer using the Rust programming language and the gtk-rs and
gstreamer-rs bindings.*

Me TV is a digital television (DVB – digital video broadcast) viewer for GTK-based systems.

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
configuration file, which must happen before you run Me TV.

If you want to install, then as an example:

    cargo install --root=$HOME/Built

which will install the executable to $HOME/Built/bin.

## Setting up for watching DVB

It is assumed people are using GStreamer 1.10.2 or later.

Before running Me TV, you must create a virtual channels file ~/.config/gstreamer-1.0/dvb-channels.conf for
the transmitter you receive DVB signal from. There are many tools for scanning to create a channels file, on
Linux perhaps the most used one is dvbv5-scan.  For example:

    dvbv5-scan --output=~/.config/gstreamer-1.0/dvb-channels.conf /usr/share/dvb/dvb-t/uk-CrystalPalace

on a Linux system will do the right thing of you live in the Crystal Palace transmitter region in the UK. I
suspect people will want to scan on their local transmitter, in this case you should replace the
uk-CrystalPalace with the name appropriate for the location you are when you run Me TV..

## Using Me TV

When started the initial screen of Me TV shows the frontends available. It
should show any new ones as they are connected – and remove ones as they are removed. Each frontend button
is a toggle button for the associated display for that frontend. Clicking on a frontend button will start a
new frame, tune the frontend to the channel that was shown, and start playing the channel. Channels can be
changed and there is a full screen capability.

Hopefully the UI is intuitive and gives a good UX. If not please feel free to submit an issue.

## NB

Me TV has, to date, only been tested for DVB-T and DVB-T2, none of the other formats (DVB-C, DVB-S, DVB-S2,
ATSC) have been tested. They should work, but…

## Licence

This code is licenced under GPLv3. [![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.txt)
