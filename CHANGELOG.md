# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0]

### Added

- `-p` argument to print in addition to logging

### Changed

- Logging now occurs by default, printing is optional and mainly for printing to a log file.
- systemd service example uses `-p` to log to the stdout

## [1.1.0]

### Added

- The ability for a TCP connection to attempt to reconnect whenever there's a disconnect
- Added an example systemd service to the `resources` dir.

### Changed

- `REFRESH_INTERVAL` is now 10000, up from 1000
- Logging is done through `println`, not through the `log` crate.

## [1.0.1]

### Added

- All current changes before the changelog was started