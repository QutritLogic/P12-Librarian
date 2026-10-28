# Changelog

All notable changes to P12 Librarian are listed here, newest first.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project uses [Semantic Versioning](https://semver.org/): changes that break existing files or
behaviour raise the first number (1.0.0 → 2.0.0), new features the middle one (1.0.0 → 1.1.0)
and bug fixes the last one (1.0.0 → 1.0.1). Before 1.0, new features and breaking changes
raised the middle number and bug fixes the last one.

Each release lists its changes under these headings, as needed: **Added** (new
features), **Changed** (changes to existing behaviour), **Fixed** (bug fixes),
**Removed** (features taken out) and **Security**.

## [Unreleased]

### Changed

- P12 Librarian is now open source under the MIT License, replacing the freeware licence. The source code is on
  GitHub: <https://github.com/QutritLogic/P12-Librarian>. The disclaimer names the new licence, so the app asks
  to accept it again.

## [1.0.0] - 2026-10-28

### Changed

- The user guide's introduction lists every feature, and its shortcut table includes Ctrl+drag (copy) and
  Ctrl+Shift+A (Appearance).

### Fixed

- A failed tag or settings command can no longer make later tag changes fail as well.

## [0.11.0] - 2026-10-23

### Added

- Move and shift on the synth: dropping patches within the Prophet 12 pane now asks whether to swap them or to
  move them to the drop slot, with the patches in between moving up or down to close the gap. U1–U4 count as one
  long row of slots. The dialog shows what goes where and roughly how long it takes, and remembers the choice.
  Backed up, verified and undoable like a swap.
- Holding a dragged patch over a bank tab opens that bank, so patches can be dropped in another bank.
- Bank sheets: File → Export bank sheet… saves banks read from the synth, or the File pane's list, as a PDF to
  print (A4, one bank per page with slot, name, Layer B and favourites) or as CSV for a spreadsheet (with tags).

## [0.10.0] - 2026-10-18

### Added

- Sortable columns: click a heading to sort by favourite, slot, name, Layer B or comparison; click again for the
  other way round, a third time for the list's own order. Writing selected patches to consecutive slots, and
  Save selected…, follow the order shown.
- File → Open recent lists the last eight `.syx` files opened or saved in the File pane.
- The panes can be resized by dragging the gap between them; double-click it for equal widths. The width is
  remembered.
- The header shows the synth's model and OS version, e.g. “Prophet 12 Keyboard · OS 1.4.2”. `p12 identify` also
  shows the OS version this way.
- Reconnecting: the app notices when the synth is unplugged or switched off, and connects again by itself when
  it comes back (unless you clicked Disconnect).

### Fixed

- The In and Out labels in the header could wrap onto two lines at 110 % zoom and above.

## [0.9.0] - 2026-10-18

### Added

- Edit `.syx` files without a synth: start a new list (Ctrl+N), drag patches into it from the Prophet 12 pane,
  add another file's patches, reorder by dragging, remove (Delete), renumber to consecutive User slots, and
  save (Ctrl+S) or save as (Ctrl+Shift+S). A `*` marks unsaved changes, and the app asks before they're lost.
- Compare: a column in both panes showing, by sound, whether each patch is the same, differs, is elsewhere or
  is missing on the other side, with a summary above the File pane.
- Duplicates: a ×2 badge after a name when the same sound is in the pane more than once.
- Search words `is:dup`, `is:same`, `is:differs` and `is:missing`.

### Fixed

- Opening a file that can't be read no longer empties the File pane.

## [0.8.2] - 2026-10-11

### Fixed

- The search box in the File pane could stick out past the right edge of the window.

## [0.8.1] - 2026-10-11

### Fixed

- Copying a patch into Layer B of a tagged or favourite patch made it lose its tags and favourite mark.

## [0.8.0] - 2026-10-11

### Added

- Favourites: click the ☆ next to a patch, or press F, to mark it; the ★ button next to each search box shows
  only favourites.
- Tags: press T, or right-click → Tags…, to tag the selected patches. Tags show after the name; click one, or
  search for `#tag`, to find the patches that have it. Tags and favourites belong to the sound, so they stay
  with a patch when it's moved, renamed or saved to a file. They're kept in `tags.json` next to the settings.
- Re-read from synth (R, or right-click) reads the selected slots again, e.g. after saving an edit on the
  synth, and says which had changed. When a tagged or favourite slot has changed on the synth since it was last
  read, the new version keeps its tags and favourite mark.
- The version number is shown at the bottom right of the main window.

### Changed

- A cleaner About box, with a photo of a Prophet 12 in Qutrit's studio.

## [0.7.1] - 2026-10-11

### Added

- An AppImage for other 64-bit Linux distributions (Fedora 40, Arch, openSUSE Tumbleweed and
  others from 2024 on). It runs without installing and has no `p12` command-line tool.

### Fixed

- On Linux, after opening Appearance with Ctrl+Shift+A, dragging a patch to another slot on
  the synth offered to copy it (the Write dialog) instead of swapping the two, until the
  app was restarted. Ctrl is now read from the keyboard itself when a patch is dropped.
- The user guide named Debian 12, Ubuntu 22.04 and Linux Mint 21 as supported, but the app
  needs Ubuntu 24.04, Linux Mint 22, Debian 13 or newer.

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

[Unreleased]: https://github.com/QutritLogic/P12-Librarian/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.11.0...v1.0.0
[0.11.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.8.2...v0.9.0
[0.8.2]: https://github.com/QutritLogic/P12-Librarian/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/QutritLogic/P12-Librarian/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/QutritLogic/P12-Librarian/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/QutritLogic/P12-Librarian/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/QutritLogic/P12-Librarian/releases/tag/v0.3.0
