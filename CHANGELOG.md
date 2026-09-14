# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A Windows installer (`rumpus_setup-x64.exe`) that can install for all users or for the
  current user only, offers Start Menu and desktop shortcuts, and can make Rumpus the default
  player for `.mid` and `.midi` files. Setup refuses to install when Windows MIDI Services is
  unavailable on the PC
- Rumpus runs as a single instance: files opened from Explorer or a second command line are added
  to the playlist of the running window, which comes to the front. They start playing only when
  nothing is playing
- An application icon and version information in the executable

### Changed

- The Device menu looks for MIDI outputs every time it opens, so the Refresh Output Devices
  item and its F5 shortcut are gone
