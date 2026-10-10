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

## [0.7.0] - 2026-10-10

### Added

- A Windows 11 version (64-bit), installed with a setup program. The `p12`
  command-line tool is only included in the Linux version for now.
- The user guide covers installing on Linux and Windows, where the app keeps its files
  on each, and Windows' rule that only one program at a time can use a MIDI port.

## [0.6.0] - 2026-10-10

### Added

- Help → Licences shows P12 Librarian's licence and the full licences of the libraries,
  fonts and themes it includes. The Debian package also installs them in
  `/usr/share/doc/p12-librarian/`.

### Changed

- P12 Librarian is now freeware: free to use, including professionally, but not to
  redistribute or modify. Earlier versions were MIT-licensed. You're asked to accept
  the disclaimer once more, as it refers to the new licence.
- The About box links to the website, qutrit.se, instead of the source code.

## [0.5.0] - 2026-10-10

### Added

- An Undo button (Ctrl+Z) in the status bar after every write, rename, initialize or
  Layer B copy. It writes the previous contents back from the automatic backup.
- File → Restore points… lists every automatic backup with the slots it holds, the
  names they had and what is on the synth now, and restores it, opens it in the File
  pane, shows it in its folder or deletes it.
- Rearranging: drag a patch onto another slot in the Prophet 12 pane to swap the two
  (several selected patches swap pair by pair). Ctrl+drag copies instead.

### Changed

- Dragging within the Prophet 12 pane now swaps patches instead of copying; hold Ctrl
  to copy as before.
- Status messages after a change are shorter and no longer show the backup's full path.

### Fixed

- Two changes within the same second no longer share one backup file name, which made
  the second backup replace the first.

## [0.4.0] - 2026-10-10

### Added

- Auditioning a patch now also plays a note, middle C (C3) for one second by default, so
  you hear it without touching the keyboard. Space auditions the selected patch.
- File → Preferences… (Ctrl+,) sets the audition note, its length and MIDI channel, or
  turns the note off. The channel is read from the synth automatically.
- `p12 identify` also shows the synth's MIDI channel.
- Keyboard navigation: arrow keys, Home/End and Page Up/Down move through the list
  (with Shift to extend the selection), Left/Right switch panes, Ctrl+Page Up/Down
  switch banks, Ctrl+A selects all, Enter writes the selected file patches and Delete
  initializes the selected slots (both ask first). The active pane is outlined.
- An option to audition each patch as you move to it with the arrow keys.

### Changed

- Switching bank or searching starts the list at the top.

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

[Unreleased]: https://github.com/QutritLogic/P12-Librarian/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/QutritLogic/P12-Librarian/releases/tag/v0.3.0
