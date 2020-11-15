# Me TV – it's TV for me computer

Me TV is a DVB – digital video broadcast – viewer based on GTK+3 and GStreamer.

GitLab Master [![GitLab build status](https://gitlab.com/Russel/me-tv/badges/master/pipeline.svg)](https://gitlab.com/Russel/me-tv)
&nbsp;&nbsp;
GitLab v3.0.x [![GitLab build status](https://gitlab.com/Russel/me-tv/badges/v3.0.x/pipeline.svg)](https://gitlab.com/Russel/me-tv)

Travis-CI Master: [![Travis-CI](https://travis-ci.org/Me-TV/Me-TV.svg?branch=master)](https://travis-ci.org/Me-TV/Me-TV)
&nbsp;&nbsp;
Travis-CI v3.0.x: [![Travis-CI](https://travis-ci.org/Me-TV/Me-TV.svg?branch=v3.0.x)](https://travis-ci.org/Me-TV/Me-TV)

Licence: [![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
&nbsp;&nbsp;&nbsp;&nbsp;
Download precompiled executables from Bintray:
[![Download](https://api.bintray.com/packages/me-tv/Downloads/Me-TV/images/download.svg)](https://bintray.com/me-tv/Downloads/Me-TV)

*NB This is a rewrite of Me TV for GTK and GStreamer using the Rust programming language and the gtk-rs
and gstreamer-rs bindings.*

The home page for previous versions of Me TV is [Me TV Home Page on
Launchpad](http://launchpad.net/me-tv).

## Dependencies

To run Me TV, it is assumed you have the following installed:

* GTK 3.18.0 or later.
* GStreamer 1.12.0 or later for Me TV 3.0.x, or GStreamer 1.16.0 or later for Me TV 3.1.x or later.
* The needed GStreamer plugins – currently the DVB plugins are in the 'bad' package; they started there for
historical reasons, and although they are not now bad, they are good, they has never migrated to the
'good' package. Some of the other bits and pieces that might get used during use of Me TV are in
the 'ugly' package, so best to install that; the things in there aren't actually ugly, some are
indeed very nice – long story.

For compiling Me TV from source code you need the development packages installed as well as the
library packages.

| OS     | Execution Packages | Development Packages | Notes |
| --- | --- | --- | --- |
| Debian | libgtk-3-0 | libgtk-3-dev | _Ubuntu is based on regular snapshots of Debian Sid and_ |
| (, Ubuntu, | libgstreamer1.0-0 | libgstreamer1.0-dev | _Mint based on regular snapshots of Ubuntu,_ |
| and Mint) | gstreamer1.0-plugins-base | libgstreamer-plugins-base1.0-dev | _so ensuring packages are in_ Debian Sid_ |
| | gstreamer1.0-plugins-good | libgstreamer-plugins-bad1.0-dev | _Debian Sid causes them to appear in_ |
| | gstreamer1.0-plugins-bad | | _Ubuntu and Mint at the next snapshot._ |
| | gstreamer1.0-plugins-ugly | | |
| | gstreamer1.0-libav | | |
| | gstreamer1.0-gtk3 | | |
| | gstreamer1.0-gl | | |
| | | | |
| Fedora | gtk3 | gtk3-devel | |
| | gstreamer1 | gstreamer1-devel | |
| | gstreamer1-plugins-good| gstreamer1-plugins-base-devel | |
| | gstreamer1-plugins-good-extras| gstreamer1-plugins-bad-free-devel | |
| | gstreamer1-plugins-ugly| | |
| | gstreamer1-plugins-bad-free | | |
| | gstreamer1-plugins-bad-free-extras | | |
| | gstreamer1-plugins-base-tools | | |
| | | | |
| OpenSUSE | gtk3 | gtk3-devel-64bit | |
| | gstreamer1 | | |
| | gstreamer1-plugins-good | | |
| | gstreamer1-plugins-ugly| | |
| | gstreamer1-plugins-bad-free | | |
| | gstreamer1-plugins-base-tools | | |
| | | | |

You will *not* need the "development packages" to execute the compiled executables once compiled
or run pre-compiled executables, only the library dependencies are needed for execution.

## Using Pre-compiled Executables

Most users of Me TV will not want to be bothered with trying to build the executables from
source code. Pre-compiled executables are therefore available on Bintray; a set of pre-compiled
executables for each release. The Me TV project page is
[here](https://bintray.com/beta/#/me-tv/Downloads/Me-TV). Go to the release you are interested
in and there is section "Files" that has the pre-compiled executables for that release.

## Building (and Installing) From Source

If you are going to build the executables from source, you need to either take a clone of this
repository, or download a release tarball.  You will then need to build the executable. This
being a Rust program all build is handled using Cargo – though there is a Meson/Ninja build for
those who wish to use that, Rust and Cargo are still needed though, the Meson/Ninja build just
manages use of Cargo. Rust and Cargo may be packaged for your operating system, but most people
use [Rustup](https://rustup.rs/) to install Rust and Cargo so as to stay up to date with both.

Once you have Rust and Cargo installed, by whatever means, then in the directory of the Me TV
project, typing the command:

    cargo build

will create a debug build, add the `--release` option to the command line to get a release
build.

Of course there is always the option of typing:

    cargo run --bin me-tv

to build and also run the program if the build is successful. Again the `--release` option is
available on this command line – but see below about creating the channels configuration file,
which must happen before you can watch television using Me TV.

If you want to make use of the Meson/Ninja build you will need Meson and Ninja
installed. Usually this is by installing the relevant packages.  You will need to clone the Me
TV Git repository or download a release tarball just as above. If you then cd into the
directory, and run:

    mkdir Build
    cd Build
    meson --prefx=$HOME/Built ..
    ninja

a build of Me TV using Cargo will happen. The `prefix` option to the meson command is only
needed if you are not going to install Me TV to the usual place. Once the build is complete:

    ninja install

will install Me TV.

## Setting up for watching DVB

Before being able to watch a television channel using Me TV, you must have a channels file for
the transmitter you are getting DVB signal from. This channels file is
_$HOME/.config/gstreamer-1.0/dvb-channels.conf_. It is assumed this file is in DVBv5 format,
DVBv3 format files will cause an error.

A way of creating this file from a running Me TV is available using the menu on the application
window, currently it requires the executable _dvbv5-scan_ be installed. On Debian Sid this is in
package _dvb-tools_, whereas on Fedora Rawhide it is in the package _v4l-utils_ – for some
reason Debian splits out the DVB tools from the V4L utils, whereas Fedora keeps them all
together. To create the channels file you will not only need _dvbv5-scan_ installed but also the
transmitter data files. These are in the package _dtv-scan-tables_ on both Debian and
Fedora. However Debian installs them to _/usr/share/dvb/dvb-t/_ whereas Fedora installs them to
_/usr/share/dvbv5/dvb-t/_. You will also need to set the correct delivery system for your
area. For example, Europe, Australia, and many other placed use DVB-T, North America uses ATSC.

To have the channels file available before executing Me TV you can run _dvbv5-scan_
manually. For example:

    dvbv5-scan --output=~/.config/gstreamer-1.0/dvb-channels.conf /usr/share/dvb/dvb-t/uk-CrystalPalace

on a Debian system (Fedora puts the transmitter files in a slightly different place) will do the
right thing if you live in the Crystal Palace transmitter region in the UK. I suspect people
will want to scan on their local transmitter, in which case you should replace the
dvb-t/uk-CrystalPalace with the name appropriate for the location you are when you run Me TV.

The Crystal Palace example is for a delivery system using DVB-T, if your area uses a different
delivery system (ATSC, DVB-C, ISDB-T, etc.) the transmitter files are in different directories,
for example /usr/share/dvb/atsc/us-ATSC-center-frequencies-8VSB

If you do not have _dvb5-scan_ installed then best advice is to install it, preferably using the
Linux distribution package management. If that is not possible then _dvbscan_ or _w\_scan_ can
be used to create the needed file, but it must be in DVBv5 format, not DVBv3 format.

## Using Me TV

When started the initial screen of Me TV shows the frontends available or a message if none are
connected. It should show any new ones as they are connected – and remove ones as they are
removed. Each frontend button is a toggle button for the associated display for that
frontend. Clicking on a frontend button will start a new frame, tune the frontend to the channel
that is shown, and start playing the channel. Channels can be changed and there is a full screen
capability.

Hopefully the UI is intuitive and gives a good UX. If not please feel free to submit an issue.

Here is a screenshot of Me TV playing BBC NEWS in one window and AlJazeera Eng in another
window. The USB device was a Hauppauge! WinTV-dualHD hence there are two separate tuners.

![Main Me TV window with one channel window showing BBC NEWS and another channel window showing
AlJazeera Eng](data/screenshots/bbc_news_aljazeera.png)

## Recording

The main Me TV program is a GUI for watching TV. With it come two command line programs:
- _me-tv-record_ records a named channel for a given period to a named MPEG-4 file. The created
files can be watched using Glide or Totem (or any other viewer program that can play MPEG-4
files).
- _me-tv-schedule_ sets up execution of _me-tv-record_ at a given time in the future, i.e. it
schedules recording a given channel for a given duration outputting to a given file, starting at
a given time in the future.

It is not yet possible to start a recording from the Me TV GUI, but things are being planned.

## NB

Me TV 3 has been developed and tested using only DVB-T and DVB-T2, none of the other delivery
systems (DVB-C, DVB-S, DVB-S2, ATSC, ISDB-T) have been tested by the developer. It is believed
all the delivery systems should work, but…  it isn't possible to test non-DVB-T and non-DVB-T2
broadcast in a DVB-T and DVB-T2 broadcast region. However, there is now at least one person
using Me TV 3 in an ATSC area, and it works. Which is good. This still leaves no data on whether
Me TV works with DVB-C, DVB-S, DVB-S2, and ISDB-T, reports of success are awaited.

## Acknowledgements

This project benefits from support by [JetBrains](https://www.jetbrains.com); JetBrains provide
a licence for [CLion](https://www.jetbrains.com/clion/). Although CLion is ostensibly a C and
C++ IDE, there is [IntelliJ Rust](https://intellij-rust.github.io/) which makes CLion an
excellent IDE for [Rust](https://www.rust-lang.org/) code development.

## Licence

This code is licenced under GPLv3.
[![Licence](https://www.gnu.org/graphics/gplv3-127x51.png)](https://www.gnu.org/licenses/gpl-3.0.en.html)
