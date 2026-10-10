# Changelog

All notable changes to P12 Librarian are listed here, newest first.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project uses [Semantic Versioning](https://semver.org/). While the version starts with 0,
new features and breaking changes raise the middle number (0.3.0 → 0.4.0) and bug fixes
raise the last one (0.3.0 → 0.3.1).

Each release lists its changes under these headings, as needed: **Added** (new
features), **Changed** (changes to existing behaviour), **Fixed** (bug fixes),
**Removed** (features taken out) and **Security**.

## [Unreleased]

### Added

- Auditioning a patch now also plays a note, middle C (C3) for one second by default, so
  you hear it without touching the keyboard. Space auditions the selected patch.
- File → Preferences… (Ctrl+,) sets the audition note, its length and MIDI channel, or
  turns the note off. The channel is read from the synth automatically.
- `p12 identify` also shows the synth's MIDI channel.

## [0.3.0] - 2026-10-09

The first release of P12 Librarian, a patch librarian for the Sequential (Dave Smith
Instruments) Prophet 12.

### Added

- Back up, browse, search, audition, rename, initialize and restore programs over MIDI
  SysEx, including copying patches into Layer B of an existing program.
- Every write saves the affected slots to `~/Documents/Prophet 12 Backups/` first and
  reads each written program back to verify it. The Factory banks are never written.
- A disclaimer that the app asks you to accept before first use (Help → Disclaimer,
  `p12 disclaimer`).
- Eleven colour themes and four fonts, previewed and chosen in View → Appearance…
  (Ctrl+Shift+A).
- A PDF user guide (F1) and the `p12` command-line tool.
- The Debian package `p12-librarian` for Linux Mint and Ubuntu (amd64).

[Unreleased]: https://github.com/QutritLogic/P12-Librarian/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/QutritLogic/P12-Librarian/releases/tag/v0.3.0
