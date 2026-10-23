# Building P12 Librarian on macOS

P12 Librarian isn't released for macOS, but it builds there from source. These steps are written
for macOS 11 Big Sur (tested target: 11.7) and also work on newer versions. No Prophet 12 is
needed: the simulator in `crates/p12-sim` stands in for the synth.

## 1. Prerequisites

Open Terminal and install:

1. **Xcode Command Line Tools** (compiler and linker; the full Xcode isn't needed):

   ```bash
   xcode-select --install
   ```

2. **Rust**, with rustup:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

3. **Node.js 22 LTS**, 22.12 or newer. *Not Node 24*: its macOS builds need macOS 13.5 or
   newer. With [nvm](https://github.com/nvm-sh/nvm):

   ```bash
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
   # open a new Terminal window, then:
   nvm install 22
   ```

   or download the macOS installer for v22 from <https://nodejs.org/en/download>.

4. **Safari 16** (System Preferences → Software Update). The app's window uses the system's
   WebKit, which is updated together with Safari. On Big Sur the last version is Safari 16.6;
   with Safari 14 or 15 the window may stay blank.

Check:

```bash
rustc --version     # 1.88 or newer
node --version      # v22.12 or newer
```

## 2. Get the code and build

```bash
git clone https://github.com/QutritLogic/P12-Librarian.git
cd P12-Librarian/app
npm install
```

## 3. Run with the simulator

In one Terminal window, from the `P12-Librarian` folder, start the simulated Prophet 12:

```bash
cargo run --release -p p12-sim -- --state ~/p12-sim.syx
```

It prints what the app asks for and what gets written. `--state` keeps the simulated synth's
programs in `~/p12-sim.syx`, so writes are still there next time; leave it out to start fresh
each time. Ctrl+C stops it (the app then shows the synth as disconnected, like unplugging it).

In a second Terminal window, start the app in development mode (the first build takes several
minutes):

```bash
cd P12-Librarian/app
npm run tauri dev
```

Accept the disclaimer. The header should show **"Prophet 12 · Connected · OS 1.4.2"**: the app
found the ports named "Prophet 12 Simulator". Things to try (the user guide, F1 or
`app/src-tauri/resources/user-guide.pdf`, describes each):

- Read U1, Read all User banks, back up all User banks (Cmd+B in the menu).
- Double-click a patch to audition it: the simulator prints "Loaded into the edit buffer" and the
  note played.
- Write patches from a `.syx` file (e.g. the backup just made) by dragging them onto slots.
- Drag patches within the Prophet 12 pane: Swap, and Move and shift.
- Rename, Initialize, Layer B copy, Undo (the status bar button), File → Restore points.
- Compare with the backup, `is:dup` in the search box (U4:90–U4:99 repeat patches from U1).
- File → Export bank sheet… as PDF and CSV.
- Stop the simulator and start it again: the app should notice and reconnect by itself.

To test what happens when a write doesn't arrive, start the simulator with `--fail-every 5`:
every fifth write is dropped, and the app should report the verification failure and name the
restore point.

## 4. Build a .app (optional)

`npm run tauri build` first regenerates the licence list, which needs cargo-about:

```bash
cargo install --locked cargo-about --features cli
cd P12-Librarian/app
npm run tauri build -- --bundles app
# -> ../target/release/bundle/macos/P12 Librarian.app
```

The app isn't signed, so the first time, right-click it in Finder and choose Open.

## Known differences on macOS

The app was made for Linux and Windows and hasn't been adapted to macOS yet. Please report what
you notice; these are already expected:

- Keyboard shortcuts in the window use the **Control** key, not Command: Control+F to search,
  Control+Z to undo, Control+A to select all. Menu shortcuts (Cmd+O, Cmd+S, Cmd+B, Cmd+Q) use
  Command.
- Copying instead of moving when dragging uses Control, which macOS also uses for right-click,
  so Control+drag may open the context menu instead.
- The first menu (File) appears under the app's name in the menu bar.
- Backups go to `~/Documents/Prophet 12 Backups/`; settings and tags to
  `~/Library/Application Support/`.

When reporting a problem, include the macOS version, whether the Mac has an Intel or Apple
chip, and what the simulator's window printed.
