// P12 Librarian — User Guide
// Build: docs/build.sh (needs the `typst` CLI). Output goes to
// app/src-tauri/resources/user-guide.pdf, which the app opens on F1.

#let version = "0.7.1"
#let accent = rgb("#2f6fd6")
#let muted = rgb("#6b7380")
#let soft = rgb("#eef2f8")

#set document(title: "P12 Librarian — User Guide", author: "Qutrit and Claude")
#set page(
  paper: "a4",
  margin: (x: 2.2cm, top: 2.2cm, bottom: 2.4cm),
  numbering: "1",
  footer: context {
    if here().page() > 2 [
      #set text(8.5pt, fill: muted)
      P12 Librarian — User Guide #h(1fr) #counter(page).display()
    ]
  },
)
#set text(font: ("Ubuntu", "DejaVu Sans"), size: 10.5pt, lang: "en")
#set par(justify: false, leading: 0.65em, spacing: 1em)
#show raw: set text(font: ("FiraCode Nerd Font Mono", "Fira Code", "DejaVu Sans Mono"), size: 9.5pt)
#show raw.where(block: true): block.with(fill: soft, inset: 8pt, radius: 4pt, width: 100%)
#set heading(numbering: "1.1")
#show heading.where(level: 1): it => {
  set text(16pt, fill: accent)
  block(above: 2em, below: 1em, it)
}
#show heading.where(level: 2): set text(12.5pt)
#show link: set text(fill: accent)

// A key or key combination.
#let key(..keys) = {
  let cap(k) = box(
    stroke: 0.6pt + muted,
    inset: (x: 3pt, y: 1.5pt),
    outset: (y: 1.5pt),
    radius: 2pt,
    text(8.5pt, font: ("FiraCode Nerd Font Mono", "DejaVu Sans Mono"), k),
  )
  keys.pos().map(cap).join([+])
}

// A button or menu item in the app.
#let ui(label) = text(weight: "bold", label)

// Highlighted boxes.
#let callout(title, color, body) = block(
  fill: color.lighten(88%),
  stroke: (left: 3pt + color),
  inset: (x: 10pt, y: 8pt),
  width: 100%,
  radius: (right: 4pt),
)[#text(weight: "bold", fill: color, title) \ #body]
#let tip(body) = callout("Tip", rgb("#2e9e5b"), body)
#let note(body) = callout("Note", accent, body)
#let warning(body) = callout("Important", rgb("#d0453a"), body)

// A worked example.
#let example(title, body) = block(
  stroke: 0.6pt + accent.lighten(50%),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[#text(weight: "bold", fill: accent)[Example: #title] \ #body]

// ───────────────────────────── Title page ─────────────────────────────

#page(numbering: none)[
  #v(5cm)
  #align(center)[
    #image("../app/src-tauri/icons/icon.png", width: 3.2cm)
    #v(0.8cm)
    #text(26pt, weight: "bold")[P12 Librarian]
    #v(0.2cm)
    #text(15pt, fill: muted)[User Guide]
    #v(1.2cm)
    #text(11pt)[Back up, browse, rename and restore the programs \ of a Prophet 12 over MIDI SysEx]
    #v(1fr)
    #image("../app/src/assets/brand/lockup-on-light.svg", width: 4.2cm)
    #v(0.5cm)
    #text(9.5pt, fill: muted)[
      Version #version · October 2026 \
      Tested with Prophet 12 Keyboard, OS 1.4.2.3
    ]
  ]
]

#page(numbering: none)[
  #outline(title: [Contents], indent: auto, depth: 2)
]
#counter(page).update(1)

// ───────────────────────────── Content ─────────────────────────────

= Introduction

P12 Librarian is a desktop app for managing the programs (patches) stored in a Dave Smith Instruments
Prophet 12. It talks to the synth over MIDI SysEx and lets you:

- *back up* all User banks, or any selection, to standard `.syx` files;
- *browse and search* every program by name;
- *audition* patches from a file without storing them;
- *write* patches to any User slot, including into *Layer B* of an existing program;
- *rename* patches, and *clean up* names typed in capitals on the synth;
- *initialize* slots with the synth's Basic Patch.

Every write is protected the same way: the slots about to change are read and saved first, and every written
program is read back from the synth and compared with what was sent (@safety).

*Use at your own risk.* Writing to the synth replaces the programs stored there, and data can be lost. Keep your
own backups. The app asks you to accept the disclaimer the first time it starts (@disclaimer).

== Tested setup

#table(
  columns: (auto, 1fr),
  stroke: none,
  column-gutter: 1.5em,
  inset: (x: 0pt, y: 3pt),
  [*Synthesizer*], [Prophet 12 Keyboard],
  [*Synth OS*], [1.4.2.3, the final release from DSI],
  [*Connection*], [USB, directly to the computer],
  [*Computers*], [Linux Mint 22.3 (Cinnamon, X11) and Windows 11, both 64-bit],
)

#note[
  The *Prophet 12 Module* is expected to work the same way, but has not been tested. There is no macOS
  version.
]

What was verified on the hardware above, beyond what the manual documents:

- where the program name is stored inside a program (the manual doesn't say);
- that writing, renaming and initializing store exactly what was sent (every write is verified);
- that lowercase letters in names display correctly;
- that the synth's own *Copy A -> B* copies a whole layer, which is what Layer B copying is based on.

= Getting started

== Installing

Download P12 Librarian from #link("https://qutrit.se/p12-librarian/")[qutrit.se/p12-librarian]. Installing a
newer version over an older one keeps your settings and backups.

/ Linux: Ubuntu 24.04, Linux Mint 22, Debian 13 or newer, 64-bit. Install the `.deb` package from a terminal
  in the folder you downloaded it to, then start P12 Librarian from the applications menu:
  ```sh
  sudo apt install ./p12-librarian_<version>_amd64.deb
  ```

/ Other Linux: Fedora 40, Arch, openSUSE Tumbleweed and other 64-bit distributions from 2024 on. Download the
  AppImage, make it executable and start it; nothing is installed. If it doesn't start, install FUSE 2
  (`libfuse2`, on Ubuntu 24.04 `libfuse2t64`).
  ```sh
  chmod +x P12-Librarian_<version>_x86_64.AppImage
  ./P12-Librarian_<version>_x86_64.AppImage
  ```

/ Windows: Windows 11, 64-bit. Run the downloaded `setup.exe`. Windows shows #emph[“Windows protected your
  PC”] because the installer isn't signed: click #ui[More info], then #ui[Run anyway]. Accept the licence, and
  start P12 Librarian from the Start menu. It installs for your user account only and needs no administrator
  rights; uninstall it in #ui[Settings → Apps].

The command-line tool `p12` (@cli) is only included in the Linux `.deb` package for now.

== Settings on the synth <settings-on-the-synth>

The Prophet 12 ignores SysEx until it's enabled. Press #ui[Global] and set:

#table(
  columns: (auto, auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + muted.lighten(50%),
  table.header([*Global setting*], [*Value*], [*Why*]),
  [10 MIDI SysEx Enable], [On], [Lets the synth receive and send SysEx.],
  [11 MIDI SysEx Cable], [USB], [Use #ui[MIDI Port] instead if you connect through a MIDI interface's DIN ports.],
)

== Connecting

Connect the Prophet 12 by USB and start the app. It looks for MIDI ports with “Prophet 12” in the name and
connects automatically. The header then shows #text(fill: rgb("#2e9e5b"))[● Connected].

If it doesn't connect, choose the ports in the #ui[In] and #ui[Out] lists, click #ui[Connect], and use
#ui[↻] to rescan after plugging the synth in. Connecting checks that a Prophet 12 really answers; if it
doesn't, see @troubleshooting.

#note[
  On *Windows*, only one program at a time can use a MIDI port. Close any DAW, editor or other MIDI software
  that uses the Prophet 12 before connecting.
]

== The window

#let pane(title, body) = block(
  stroke: 0.6pt + muted,
  radius: 3pt,
  inset: 6pt,
  width: 100%,
  height: 5.2cm,
)[#text(weight: "bold", size: 9pt, title) #v(-4pt) #body]
#let row(slot, name, b: "") = grid(
  columns: (2.6em, 1fr, 1fr),
  gutter: 4pt,
  align: left,
  text(8pt, font: "DejaVu Sans Mono", slot), text(8pt, name), text(8pt, fill: muted, b),
)

#figure(
  block(stroke: 0.8pt + muted, radius: 4pt, inset: 8pt, width: 100%)[
    #set text(8.5pt)
    #grid(
      columns: (1fr, auto),
      [*P12 Librarian*], [In [Prophet 12 ▾] Out [Prophet 12 ▾] #text(fill: rgb("#2e9e5b"))[● Connected]],
    )
    #v(2pt)
    #grid(
      columns: (1fr, 1fr),
      gutter: 8pt,
      pane[Prophet 12 #h(1fr) #box(stroke: 0.5pt, inset: 2pt)[Read U1] #box(stroke: 0.5pt, inset: 2pt)[Back up all…]][
        #text(8pt, font: "DejaVu Sans Mono")[*U1* U2 U3 U4 _F1 F2 F3 F4_] #h(1fr) #box(stroke: 0.5pt, inset: 2pt, width: 2.4cm)[Search…]
        #v(2pt)
        #row("U1:01", "BOF Cascade 5th", b: "Basic Program B")
        #row("U1:02", "MAJOR 6 9", b: "Basic Program")
        #row("U1:03", "LFO MADNESS", b: "Basic Program B")
        #row("U1:04", "BOF Kepler", b: "KeplerRH")
        #v(1fr)
        #text(8pt, fill: muted)[1 selected #h(1fr) Audition · Rename… · Save selected…]
      ],
      pane[File — my-backup.syx #h(1fr) #box(stroke: 0.5pt, inset: 2pt)[Open .syx…]][
        #h(1fr) #box(stroke: 0.5pt, inset: 2pt, width: 2.4cm)[Search…]
        #v(2pt)
        #row("U2:10", "Warm Strings", b: "Basic Program B")
        #row("U2:11", "Dist Pad", b: "Dist Pad B")
        #row("U2:12", "Glass Bells", b: "Basic Program B")
        #v(1fr)
        #text(8pt, fill: muted)[0 selected #h(1fr) Audition · Rename… · Write to synth…]
      ],
    )
    #v(2pt)
    #text(8pt, fill: muted)[Status bar: messages, progress and Cancel]
  ],
  caption: [The main window: the synth on the left, an opened file on the right.],
)

- *Header:* MIDI ports and connection status.
- *Prophet 12 pane (left):* the programs on the synth, one bank at a time. Tabs #raw("U1")–#raw("U4") are the
  User banks; #raw("F1")–#raw("F4") (in italics) are the read-only Factory banks. The small number on a tab is
  how many of its 99 slots have been read. Slots not read yet show “—”.
- *File pane (right):* the patches in an opened `.syx` file.
- *Status bar:* what just happened, or the progress of a running operation.

Every button has a tooltip. If a button is greyed out, its tooltip says why.

== Slots

A slot is written as bank and program number, as on the synth's panel: #raw("U1:05") is User bank 1,
program 5; #raw("F3:99") is Factory bank 3, program 99. The User banks #raw("U1")–#raw("U4") can be written;
the Factory banks #raw("F1")–#raw("F4") are read-only on the synth, and the app never writes to them.

Each program has two layers, A and B. The name in the #ui[Name] column is Layer A's, the one the synth shows;
the #ui[Layer B] column shows the second layer's name (@layer-b).

== Selecting

#table(
  columns: (auto, 1fr),
  stroke: none,
  column-gutter: 1.5em,
  inset: (x: 0pt, y: 3pt),
  [Click], [Select one patch.],
  [#key("Ctrl") + click], [Add or remove a patch.],
  [#key("Shift") + click], [Select a range.],
  [Double-click], [Audition the patch (@audition).],
  [Right-click], [Menu with everything you can do with the selection.],
)

= Backing up

#warning[
  Back up all User banks *before updating the synth's OS*. OS updates have been reported to change or erase
  user programs. Keep a copy of the backup somewhere other than the computer.
]

== Reading programs

- #ui[Read U1] (the name follows the open tab) reads the 99 programs of that bank, in about 3 seconds.
- #ui[Read all User banks] reads all 396 User programs, in about 10 seconds.
- To read a Factory bank, open its tab and click #ui[Read F1] (and so on).

A progress bar shows each program as it arrives. #ui[Cancel] stops reading; programs read so far are kept.

== Backing up everything

#ui[Back up all…] (#key("Ctrl", "B"), also in the #ui[File] menu) asks where to save, reads all four User banks
and saves them to one `.syx` file.

#example[a backup before an OS update][
  + Press #key("Ctrl", "B").
  + Keep the suggested name, e.g. `p12-backup-20261008-210500.syx`, or type your own, and click #ui[Save].
  + Wait about 10 seconds. The status bar says #emph[“Saved all 396 User programs to …”].
  + Copy the file to a USB stick or cloud storage.
]

== Saving a selection

Select patches in either pane, then:

- #ui[Save selected…] saves them into *one* `.syx` file;
- #ui[Save as separate files…] saves *one file per patch* into a folder you choose, named after slot and
  name, e.g. `U1-05 BOF Cascade 5th.syx`.

The files are ordinary SysEx: other librarians and SysEx tools can read them, and each one remembers which
slot it came from.

= Writing patches to the synth

== Opening a file

#ui[Open .syx…] (#key("Ctrl", "O")) loads a `.syx` file into the File pane. Files from this app, from the synth's
own #ui[Dump] functions and from other tools work. SysEx in the file that isn't a Prophet 12 program
(for example the play lists in a #ui[Dump All User Banks] file) is skipped and counted in the status bar.

== Writing

There are two ways to write patches from the File pane:

- *Drag* them onto a slot in the Prophet 12 pane. Several patches fill consecutive slots from where you drop
  them; the slots that will be filled are outlined while you drag.
- Select them and click #ui[Write to synth…], then choose either the patches' *original locations* or
  *consecutive slots starting at* a slot you pick.

Either way, a confirmation lists every slot, the patch going there, and what it replaces. Nothing is written
until you click #ui[Write].

#example[restoring one patch from a backup][
  + #ui[Open .syx…] and choose your backup.
  + Type part of the name in the File pane's search box, e.g. `strings`.
  + Select the patch and click #ui[Write to synth…].
  + Leave #ui[Original locations] selected and check that the #ui[Replaces] column shows what you expect.
  + Click #ui[Write]. The status bar confirms that the patch was written and verified.
]

== Rearranging patches on the synth <rearrange>

Drag a patch in the Prophet 12 pane onto another slot to *swap* them: #raw("U1:04") goes to #raw("U4:50") and
#raw("U4:50") goes to #raw("U1:04"). The app asks first, then reads both slots, keeps them as a restore point,
writes each patch to the other's slot and checks the result. #ui[Undo] (#key("Ctrl", "Z")) swaps them back.

Several selected patches swap with the same number of consecutive slots from where you drop them, pair by pair.
The two groups must not overlap.

To *copy* instead, hold #key("Ctrl") while dropping: the patch is written to the new slot and stays where it was.
Patches from the Factory banks are always copied, since those banks can't be changed.

#example[moving a pad to the top of a bank][
  + Read bank U1.
  + Drag #raw("U1:37") onto #raw("U1:01") and click #ui[Swap].
  + The pad is now at #raw("U1:01"), and the patch that was there is at #raw("U1:37").
]

== Audition <audition>

Double-click a patch, press #key("Space"), or select one and click #ui[Audition], to load it into the synth's
*edit buffer*. The app then plays middle C (C3) for one second so you hear the patch straight away, and you can
keep playing it from the keyboard. Nothing is stored: selecting a program on the synth loads its stored version
again.

To step through a bank by ear, turn on #ui[Audition each patch as you move with the arrow keys] in
#ui[File → Preferences…]: every #key("↑") or #key("↓") then loads and plays the next patch.

#ui[File → Preferences…] (#key("Ctrl", ",")) also sets the note, its length (0.25–4 s) and the MIDI channel, or turns
the note off. The channel is normally *automatic*: the app asks the synth which channel it receives on, and uses
channel 1 when it is set to All. The note stops early when you audition another patch, or when the app reads
from or writes to the synth.

#warning[
  Auditioning replaces the sound currently being edited on the synth. Save any unsaved edits on the synth
  first. To keep an auditioned sound, press #ui[Write] on the synth and choose a slot.
]

= Renaming

== Rename

Select one or more patches and click #ui[Rename…], press #key("F2"), or right-click and choose #ui[Rename…].
Each patch gets an editable name field. Names can be up to 20 characters: letters (upper and lower case),
digits, spaces and most punctuation. The synth can't show #raw("* / : < > ? \\ | ~"), so those are refused and the
field turns red.

- In the *Prophet 12* pane, renaming stores the new name on the synth. Each slot is *read again first* and only
  its 20 name characters change, so edits you've made on the synth since reading it are kept. Factory patches
  can't be renamed; copy them to a User slot first.
- In the *File* pane, renaming changes the list in the app only. Save the patches or write them to the synth to
  keep the new names.

== Cleanify names

Names typed on the synth are often in capitals. Select patches, right-click and choose
#ui[Cleanify names…]. The app suggests tidier names and shows them in the same dialog, where you can change
any of them before confirming. Names that wouldn't change aren't listed.

#table(
  columns: (1fr, 1fr, 2fr),
  inset: 6pt,
  stroke: 0.5pt + muted.lighten(50%),
  table.header([*Before*], [*Suggested*], [*Why*]),
  [`BOF COOL PAD`], [`Bof Cool Pad`], [Each word in capitals gets one capital letter.],
  [`DISTPAD`], [`DistPad`], [Run-together sound words are split.],
  [`WARMSTRINGS`], [`WarmStrings`], [Plurals are recognised.],
  [`LFO MADNESS`], [`LFO Madness`], [Known abbreviations (FM, LFO, PWM, EP…) stay in capitals.],
  [`SUBBASS-DIST`], [`SubBass-Dist`], [Punctuation and digits are kept.],
  [`Kepler`], [`Kepler`], [Words with lowercase letters are left alone.],
  [`BOFJUPITER`], [`Bofjupiter`], [Unknown words can't be split; edit the suggestion.],
)

Splitting uses a built-in list of about 100 sound words (pad, lead, bass, string, dist, warm…). Anything else
is just given a capital letter, so check the suggestions before confirming.

= Layer B <layer-b>

Each Prophet 12 program has two layers, A and B. Layer B is heard when #ui[Split A | B] (B plays above the split
point) or #ui[Stack A + B] (both play together) is on; otherwise #ui[Edit Layer B] on the synth plays it.

*To put any patch into Layer B of a program*, drag it onto the #ui[Layer B] column of that program's row in the
Prophet 12 pane. Only the Layer B cell is outlined while you hover over it. The dragged patch's Layer A sound and
name become the program's Layer B; Layer A stays exactly as it is on the synth.

This is the synth's #ui[Layers → Copy A -> B], but with the sound taken from any patch. Dropping a patch onto
its *own* Layer B does exactly what Copy A -> B does.

#example[a layered pad][
  + Read bank #raw("U1") and open a file with a string patch in the File pane.
  + Drag the string patch onto the #ui[Layer B] cell of your pad, e.g. #raw("U1:20").
  + Confirm. The pad keeps its sound in Layer A and gets the strings in Layer B.
  + On the synth, select #raw("U1:20") and press #ui[Stack A + B] to hear both at once.
]

#note[
  Dropping a patch on the *rest* of a row writes the whole patch to that slot instead. Watch which part of the
  row is outlined before you let go.
]

= Initializing slots

Right-click one or more User slots and choose #ui[Initialize…] to replace them with the synth's Basic Patch,
named *Init Patch*: a clean starting point for a new sound.

The first time, the app needs a copy of the Basic Patch, and the dialog explains how:

+ On the synth, press #ui[Global] and go to *23 Basic Patch*.
+ Press #ui[Write Now] (soft key 1). This loads the Basic Patch into the edit buffer, replacing any unsaved
  edits; nothing is stored.
+ Click #ui[Capture template] in the app.

After that the template is remembered, and #ui[Initialize…] goes straight to the confirmation.
#ui[Recapture template…] in the same dialog reads it again.

= Searching

Type in the search box above either list, or press #key("Ctrl", "F") to jump to the box of the pane you used last.
#key("Esc") clears it.

- Upper and lower case don't matter. Several words must *all* match, in any order: `bof pad` finds
  “BOF COOL PAD”.
- The slot counts too: `u2 pad` finds pads in bank U2.
- In the Prophet 12 pane the search covers *every bank that has been read*, not just the open tab. Each tab
  shows its number of matches; click a tab to clear the search and open that bank. Use
  #ui[Read all User banks] first to search everything.
- `#pad` finds patches tagged Pad (see the next section). Tags and words can be combined: `#pad warm`.
- The #ui[★] button next to the search box shows only favourites; it works together with the search.
- Matching letters are highlighted.

#note[
  Patches hidden by a search are never acted on, even if they were selected before. While searching, patches
  can't be dropped onto the Prophet 12 list, because the slots they would fill might be hidden. Dropping onto a
  Layer B cell still works.
]

= Tags and favourites <tags>

Tags and favourites help you find patches again. They are kept by the app on your computer; nothing is stored
on the synth or changed in your `.syx` files.

- *Favourites:* click the #ui[☆] next to a patch, or select patches and press #key("F"). Press it again to
  remove the mark. The #ui[★] button next to the search box shows only favourites.
- *Tags:* select one or more patches and press #key("T"), or right-click → #ui[Tags…]. Type a tag and press
  #key("Enter"), or click a suggestion; #key("Enter") in the empty field saves. Spaces in a tag become
  hyphens, so `warm pad` is stored as `warm-pad`. With several patches selected, tags that only some of them
  have are shown dashed with a count; #ui[+] gives all of them the tag.
- Tags appear after the name. Click one to show all patches with that tag, or search for it with `#tag`.

Tags and favourites belong to the *sound*, not to the slot or the name. They stay with a patch when it's
swapped, written to another slot, renamed, saved to a file or restored from a backup, and they show up on the
same patch in any `.syx` file. Patches that sound exactly the same share them, so tagging one unedited
“Basic Program” tags them all. Changing the sound on the synth makes it a new patch without tags.

= Safety and backups <safety>

Every operation that changes the synth (writing, renaming, initializing and Layer B copies) works the same way:

+ The slots about to change are *read from the synth* and saved to a backup file.
+ The new programs are written, one at a time, with a short pause after each so the synth can store it.
+ Every slot is *read back* and compared with what was sent. A mismatch is reported, with the backup's location.

Backups go to `~/Documents/Prophet 12 Backups/` (on Windows: `Documents\Prophet 12 Backups`) and are named after what was done and when, e.g.
`before-rename-20261008-210232.syx`. The app never deletes them unless you ask it to.

The Factory banks are never written: the app refuses before anything is sent.

== Undo and restore points <restore>

Right after a write, rename, initialize or Layer B copy, the status bar shows an #ui[Undo] button
(#key("Ctrl", "Z")). It writes the previous contents back from the backup just made, with the same backup and
verification, so the undo can itself be undone.

#ui[File → Restore points…] lists every backup, newest first, e.g. #emph[“10 Oct 2026, 20:32 — Before rename ·
U1:03–U1:07”]. Selecting one shows the slots it holds, the names they had then, and what is in those slots on
the synth now. From there you can:

- #ui[Restore] all its slots to the synth;
- #ui[Open in File pane], to restore only some of the patches by writing them as usual;
- #ui[Show in folder] or #ui[Delete] it (click #ui[Delete] twice).

#example[undoing a rename from yesterday][
  + #ui[File → Restore points…] and select the one marked #emph[Before rename] at the right time.
  + Check the #ui[Saved] and #ui[On the synth now] columns.
  + Click #ui[Restore].
]

= Settings

== Zoom, font and theme

#table(
  columns: (auto, 1fr),
  stroke: none,
  column-gutter: 1.5em,
  inset: (x: 0pt, y: 3pt),
  [#key("Ctrl", "+") or #key("Ctrl") + wheel up], [Zoom in],
  [#key("Ctrl", "−") or #key("Ctrl") + wheel down], [Zoom out],
  [#key("Ctrl", "0")], [Actual size],
)

The #ui[View] menu has the same commands.

#ui[View → Appearance…] (#key("Ctrl", "Shift", "A")) shows every theme as a small preview of the app and every
font with a sample line. Click one to use it straight away. The same choices are in #ui[View → Theme] and
#ui[View → Font].

- *Themes:* System (light or dark, following the desktop), Dracula, Catppuccin Mocha, Catppuccin Latte (light),
  Tokyo Night, Gruvbox Dark, One Dark Pro, Rosé Pine, Kanagawa, Everforest Dark and Solarized Dark.
- *Fonts:* Fira Code, JetBrains Mono and Hack (Nerd Font versions, included with the app; an installed copy is
  used when present), or the system font.

Zoom (80–250%), font and theme are remembered.

== Where the app keeps things

#table(
  columns: (auto, 1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + muted.lighten(50%),
  table.header([], [*Linux*], [*Windows*]),
  [Backups], [`~/Documents/Prophet 12 Backups/`], [`Documents\Prophet 12 Backups\`],
  [Settings], [`~/.config/app.p12librarian/settings.json`], [`%APPDATA%\app.p12librarian\settings.json`],
  [Tags and favourites], [`~/.config/app.p12librarian/tags.json`], [`%APPDATA%\app.p12librarian\tags.json`],
  [Init template], [`~/.local/share/app.p12librarian/init-template.syx`], [`%APPDATA%\app.p12librarian\init-template.syx`],
)

= Keyboard shortcuts

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + muted.lighten(50%),
  table.header([*Keys*], [*Action*]),
  [#key("Ctrl", "O")], [Open a `.syx` file],
  [#key("Ctrl", "B")], [Back up all User banks],
  [#key("Ctrl", "F")], [Search in the pane used last],
  [#key("F2")], [Rename the selected patches],
  [#key("F")], [Mark or unmark the selected patches as favourites],
  [#key("T")], [Tags for the selected patches],
  [#key("Space")], [Audition the selected patch],
  [#key("↑") #key("↓")], [Move through the list; with #key("Shift"), extend the selection],
  [#key("Home") #key("End")], [First or last patch],
  [#key("Page Up") #key("Page Down")], [Move a screenful],
  [#key("←") #key("→")], [Switch between the Prophet 12 pane and the File pane],
  [#key("Ctrl", "Page Up") #key("Ctrl", "Page Down")], [Previous or next bank],
  [#key("Ctrl", "A")], [Select all patches in the pane],
  [#key("Enter")], [File pane: write the selected patches to the synth (asks first)],
  [#key("Delete")], [Prophet 12 pane: initialize the selected slots (asks first)],
  [#key("↓") in a search box], [Jump to the first result],
  [#key("Ctrl", "Z")], [Undo the last change on the synth],
  [#key("Ctrl", ",")], [Preferences (audition note)],
  [#key("F1")], [Open this guide],
  [#key("Ctrl", "+") #key("Ctrl", "−") #key("Ctrl", "0")], [Zoom in, out, actual size],
  [#key("Esc")], [Close a dialog or menu; clear a search; clear the selection],
  [#key("Ctrl", "Q")], [Quit],
)

= Troubleshooting <troubleshooting>

/ “The ports opened, but no Prophet 12 answered”: Check *MIDI SysEx Enable* and *MIDI SysEx Cable* in the
  synth's Global menu (@settings-on-the-synth), and that the selected ports are the synth's.

/ Timeouts or failed verification: Some MIDI interfaces corrupt long SysEx messages. Owners have reported
  problems with Native Instruments Maschine MIDI ports, Digidesign Mbox and CME U2MIDI interfaces. Connect the
  synth directly by USB.

/ The app doesn't find the synth after plugging it in: Click #ui[↻] next to the port lists.

/ Connecting fails on Windows although the synth is listed: Another program is using the Prophet 12's MIDI
  port; Windows allows only one at a time. Close DAWs, editors and other MIDI software, then click #ui[Connect].

/ A patch sounds different after auditioning: Auditioning replaced the edit buffer. Select the program on the
  synth again to reload it.

/ Layer B can't be heard: Turn on #ui[Split A | B] or #ui[Stack A + B] on the synth, or press #ui[Edit Layer B]
  to play Layer B alone.

= Command-line tool <cli>

The Linux version also includes `p12`, a command-line version of the main functions, which is useful for
scripts and scheduled backups. Run `p12 --help` for every option. It isn't part of the Windows version yet.

```sh
p12 ports                                    # list MIDI ports
p12 identify                                 # check the synth answers
p12 backup                                   # all User banks to one file
p12 backup -s U1,U2:10-20 -o mine.syx        # a selection
p12 backup -s user --split -o patches/       # one file per program
p12 edit-buffer -o current.syx               # the sound being edited
p12 list mine.syx                            # list a file (no synth needed)
p12 send mine.syx --select U1:05 --audition  # play without storing
p12 send mine.syx --select U1:05 --to U4:99 --verify
```

`send` backs up the slots it will overwrite, shows what will be replaced and asks before writing.

= Technical notes

For the curious, and for anyone building on this work.

- *Messages* (manual, Appendix D): request a program with `F0 01 2A 05 <bank> <program> F7`; the reply and
  writes use `F0 01 2A 02 <bank> <program> <1171 bytes> F7`. The edit buffer uses types `06` (request) and `03`
  (dump). Banks 0–3 are U1–U4 and 4–7 are F1–F4; programs are 0–98.
- *Packing:* each program is 1024 bytes, sent as 1171 MIDI bytes: every 7 bytes travel as 8, with the top bits
  collected in a leading byte.
- *Layout* (found on the tested synth; not in the manual): Layer A is bytes 0–511 and Layer B bytes 512–1023. The
  name is at bytes 402–421 (Layer A) and 914–933 (Layer B), 20 characters padded with spaces. Byte positions are
  *not* the NRPN parameter numbers.
- *Copy A -> B* on the synth copies all 512 bytes of the layer. Bytes 23 and 293 of each layer hold the same value
  in both layers of every program examined, so they are kept when a layer is copied from another program.
- *Global Parameters dump*: the manual describes 50 nibbles, but the synth (OS 1.4.2.3) sends one byte per
  parameter, 26 bytes in all. The MIDI channel is the third byte (0 = All, 1–16).
- The *Device Inquiry* reply only reports the main OS version (1 for OS 1.4.2.3); the Global screen shows the
  full version.

= About

#image("../app/src/assets/brand/lockup-on-light.svg", width: 5cm)

P12 Librarian is made by *Qutrit* and *Claude* (Opus 5.5), an AI model made by Anthropic.
Contact: #link("mailto:qutritlogic@proton.me")[qutritlogic\@proton.me]. Website, with new versions:
#link("https://qutrit.se/p12-librarian/")[qutrit.se/p12-librarian].

The *Prophet 12* was designed by *Dave Smith Instruments (DSI)*, founded by Dave Smith, and released in 2013. The
company is now called Sequential:
#link("https://sequential.com/product/prophet-12-keyboard/").

This project is not affiliated with Sequential or Dave Smith Instruments. Prophet 12 is a trademark of its owner.

P12 Librarian is *freeware*: free to use, including for professional work, but not to redistribute or modify.
Get it from the website. The full licence, and the licences of the libraries, fonts and themes it includes, are
under #ui[Help → Licences].

// The text comes from DISCLAIMER.md, the same file the app shows.
= Disclaimer <disclaimer>

#{
  set text(size: 9pt)
  for block in read("../DISCLAIMER.md").split("\n\n").map(b => b.trim()) {
    if block == "" or block.starts-with("# ") {
      continue
    } else if block.starts-with("## ") {
      heading(level: 2, outlined: false, numbering: none, block.slice(3))
    } else {
      par(block.replace("\n", " "))
    }
  }
}
