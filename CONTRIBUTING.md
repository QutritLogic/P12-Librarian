# Contributing to P12 Librarian

Thank you for helping. Bug reports, testing on macOS and pull requests are all welcome. The
project is maintained by Qutrit (qutritlogic@proton.me), who reviews pull requests and makes the
releases.

## Reporting a bug

Open an issue on GitHub and include:

- the P12 Librarian version (bottom right of the window) and how you got it (download or own build)
- your operating system and version; on a Mac, whether it has an Intel or Apple chip
- the synth model and OS version shown in the header, or that you used the simulator
- what you did, what you expected and what happened instead

Never post backups or `.syx` files you don't want to share publicly.

## Building and testing

- Building on Linux and Windows: [README.md](README.md).
- Building on macOS: [docs/macos.md](docs/macos.md).
- **Test with the simulator**, not your synth: `cargo run --release -p p12-sim` starts a simulated
  Prophet 12 on virtual MIDI ports (macOS and Linux), and the app connects to it by itself. See
  "Testing without the synth" in the README. Only try a change on a real Prophet 12 once it works
  with the simulator, and back up your synth first by other means.

Before opening a pull request, run the same checks that run on GitHub for every pull request:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd app && npm run check
```

## Pull requests

1. Fork the repository and create a branch from `main`, e.g. `mac/cmd-shortcuts` or `fix/…`.
2. Keep each pull request to one change. Commit messages: a short summary line, then what and
   why.
3. In the pull request, say what changed, how you tested it (simulator or real synth, which
   operating system) and what you didn't test.

The maintainer may ask for changes, and decides what is merged and when it is released.

### Rules for the code

- **The synth comes first.** Every change on the synth goes through `write_and_verify` in
  `app/src-tauri/src/commands.rs`: it backs up the target slots, writes, then reads every slot
  back and compares. Don't add a write path that skips it. The Factory banks (F1–F4) are never
  written.
- **No network connections.** The app works offline: no update checks, telemetry, crash
  reports, or fonts and scripts from the web.
- **Dependencies** need a permissive licence that is listed in `licenses/about.toml`.
- **Changelog.** Every change a user would notice gets a line under `## [Unreleased]` in
  [CHANGELOG.md](CHANGELOG.md), under Added, Changed, Fixed or Removed.
- **User guide.** When behaviour changes, update `docs/user-guide.typ` too. Rebuilding the PDF
  with `docs/build.sh` needs the `typst` CLI; if you can't run it, say so in the pull request.
- **Wording.** Plain and factual, British spelling (colour, licence, favourite), as in the
  existing interface and guide.
- **Style.** Match the surrounding code: doc comments on functions and types, few inline
  comments. Rust uses edition 2024 and needs Rust 1.88 or newer.

## Licence

P12 Librarian is under the [MIT License](LICENSE). By opening a pull request you agree that your
contribution is licensed under the same terms. The app and its documentation credit QutritLogic;
contributors are credited by the git history and GitHub's list of contributors.
