# P12 Librarian: notes for Claude Code

A patch librarian for the Sequential (Dave Smith Instruments) Prophet 12, open source under the
MIT License, by **Qutrit** (qutritlogic@proton.me) at <https://qutrit.se>. Start with
`README.md` and `CONTRIBUTING.md`; on macOS also `docs/macos.md`.

## Working on this repo

- **Branches and pull requests only.** Never commit to `main` or push it. For each piece of work,
  create a branch from an up-to-date `main` (e.g. `mac/cmd-shortcuts`, `fix/…`), commit there,
  push the branch and open a pull request with `gh pr create`. The maintainer reviews and merges;
  don't merge pull requests yourself.
- In the pull request, say what changed, how it was tested (simulator, real synth, which OS) and
  what wasn't tested.
- **Releases are the maintainer's job.** Don't change version numbers, create tags or GitHub
  releases, or touch the website, the download server or anything about deployment.
- Commit messages: a short summary line, then what and why. Keep each commit to one change.

## Rules for the project

- **No names.** The project is credited to "Qutrit" only. Don't add people's names to code,
  docs, package metadata or credits.
- **MIT License.** `LICENSE` is the MIT License, copyright Qutrit. New dependencies must have a
  permissive licence listed in `licenses/about.toml` (Rust); `licenses/rust.md` is generated,
  don't edit it by hand.
- **No network connections.** The app works offline and promises not to connect anywhere: no
  update checks, telemetry, crash reports, fonts or scripts from the web.
- **Changelog.** Every change a user would notice (feature, behaviour, fix) gets a line under
  `## [Unreleased]` in `CHANGELOG.md`, in the same commit, under Added / Changed / Fixed /
  Removed. Developer-only changes (simulator, build scripts, refactoring) don't.
- **User guide.** When behaviour changes, update `docs/user-guide.typ` too. `docs/build.sh`
  rebuilds `app/src-tauri/resources/user-guide.pdf` (needs the `typst` CLI); if you can't run it,
  say so in the pull request.
- **Wording.** Plain, factual, British spelling (favourite, colour, licence), as in the existing
  UI and guide. No marketing language.

## Safety: the app writes to a real synth

- Every change on the synth goes through `write_and_verify` in `app/src-tauri/src/commands.rs`:
  it backs up the target slots first (a restore point), writes, then reads every slot back and
  compares. Don't add a write path that skips it.
- The Factory banks (F1–F4) are never written; only U1–U4.
- **Test with the simulator**, not a real synth: `cargo run --release -p p12-sim` (see
  `README.md`, "Testing without the synth"). Never send anything to a real Prophet 12 unless the
  person you're working with asks for it explicitly.

## Layout

```
crates/p12-core/   protocol only, no I/O: 7-in-8 packing, Program, Message, .syx files
crates/p12-midi/   midir transport: ports, request/reply with timeout, throttled writes
crates/p12-cli/    the `p12` command-line tool (Linux only in releases)
crates/p12-sim/    simulated Prophet 12 on virtual MIDI ports (macOS, Linux)
app/src-tauri/     Tauri 2 backend: commands.rs (synth operations), settings.rs, tags.rs,
                   menu.rs, sheet.rs (bank sheet PDF/CSV)
app/src/           Svelte 5 (runes) + TypeScript frontend: App.svelte, lib/*.svelte, lib/api.ts
docs/              user guide (Typst), macOS build notes
```

SysEx details found by testing a real synth are in comments in `crates/p12-core/src/sysex.rs` and
`program.rs`, and in `README.md` ("Program data layout"). Trust those over the manual where they
differ.

## Checking a change

```bash
cargo test --workspace          # libraries, CLI, simulator and app backend
cargo clippy --workspace --all-targets
cd app && npm run check         # Svelte / TypeScript type check
cd app && npm run tauri dev     # run the app (with p12-sim running in another terminal)
```

Rust code uses edition 2024 (let-chains), so it needs Rust 1.88 or newer. Match the style of
the surrounding code: doc comments on functions and types, few inline comments, names as in the
neighbouring code.

## macOS

The app isn't released for macOS; it's being built and tested there from source (see
`docs/macos.md`). Expected differences, listed in that file, include shortcuts using Control
instead of Command and the File menu appearing under the app's name. Fixes for these should
keep Linux and Windows working as they do now (check `navigator.platform` or Tauri's OS info
rather than changing the shortcuts everywhere).
