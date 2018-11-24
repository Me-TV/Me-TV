# ChangeLog

## [Unreleased]

## [3.0.4] - 2018-11-24
### Added
- Add a Meson-based build to help with installation.
- A proper changelog to help users understand the main changes of a release without reading the commit logs.
- Correctly map the adaptor/frontend button of the UI to the hardware.

### Changed
- Improve the way dialogs work during a channel scan.
- When creating a channel list, do not write directly to the target file, but instead work with a temporary
  file and only update the target file on a successful tune.
- Revamp the way the control bar timeout is handled in fullscreen mode.
- Cope with the changes due to the GStreamer move to a GitLab instance.
- Correct which cursor to used when it is shown.
- Limit the user to 256 adaptors, with 256 tuners per adaptor.

## [3.0.3] - 2018-10-23
### Added
- Try to ensure the cursor hides in fullscreen mode.

### Changed
- Try another way of handling the control bar hide timeout in fullscreen mode.
- Update minimum version of GTK+ required to run Me TV to 3.18.0.
- Various changes to ensure continuous integration on Travis-CI works and is useful.

## [3.0.2] - 2018-10-15
### Changed
- Amended the way the preferences dialog looks and behaves.
- Various internal changes to the code that should have no effect for the user.

## [3.0.1] - 2018-10-14
### Added
- Added the "use last channel" immediate play, as well as having "use default channel" immediate play.

### Changed
- Corrected the way the configuration file was used.

## [3.0.0] - 2018-10-13
The initial release of Me TV 3.
