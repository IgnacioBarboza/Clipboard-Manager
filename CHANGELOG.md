# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-25

First public release.

### Added
- Background daemon that records text and images copied on Wayland.
- `clipcrab --menu` picker built on rofi, with image thumbnails.
- History of the last 10 entries, de-duplicated and persisted across restarts
  in `$XDG_STATE_HOME/clipcrab/state.json`.
- Image storage in `$XDG_CACHE_HOME/clipcrab/images`, with orphaned images
  cleaned up on startup.
- Owner-only permissions (0700/0600) for the history and image cache.
- `--help` and `--version` flags.
- systemd user unit in `contrib/`.
