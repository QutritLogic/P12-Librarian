use std::collections::BTreeSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use p12_core::sysex::{GLOBAL_MIDI_CHANNEL, global_parameter, os_version_text};
use p12_core::syx::{self, Entry};
use p12_core::{Location, Message, Program, parse_selection};
use p12_midi::{DEFAULT_PORT_PATTERN, Synth};

const DISCLAIMER: &str = include_str!("../../../DISCLAIMER.md");

const RISK_NOTICE: &str = "Use at your own risk: writing to the synth replaces the programs stored \
there, and data can be lost. By using p12 you accept the disclaimer; run `p12 disclaimer` to read it.";

/// Librarian for the Sequential Prophet 12.
///
/// Locations are written as bank:program, e.g. U1:05 (User bank 1) or F3:99
/// (Factory bank 3). Selections are comma-separated: `user`, `factory`,
/// `all`, a bank (`U2`), a program (`U2:15`) or a range (`U2:15-30`).
#[derive(Parser)]
#[command(version, after_help = RISK_NOTICE)]
struct Cli {
    /// MIDI port for both directions: index or name substring.
    #[arg(long, global = true, default_value = DEFAULT_PORT_PATTERN)]
    port: String,
    /// Input port, if different from --port.
    #[arg(long = "in", global = true)]
    in_port: Option<String>,
    /// Output port, if different from --port.
    #[arg(long = "out", global = true)]
    out_port: Option<String>,
    /// Seconds to wait for each reply from the synth.
    #[arg(long, global = true, default_value_t = 2.0)]
    timeout: f64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List MIDI input and output ports.
    Ports,
    /// Ask the synth to identify itself and report its OS version.
    Identify,
    /// Read programs from the synth and save them as .syx.
    Backup {
        /// Programs to read.
        #[arg(short, long, default_value = "user")]
        select: String,
        /// Output file, or directory with --split. Defaults to a timestamped name.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Write one .syx file per program into the output directory.
        #[arg(long)]
        split: bool,
    },
    /// Save the synth's current edit buffer (unsaved edits) as .syx.
    EditBuffer {
        #[arg(short, long, default_value = "edit-buffer.syx")]
        output: PathBuf,
    },
    /// List the programs in .syx files (no synth needed).
    List {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Also show Layer B names.
        #[arg(long)]
        layers: bool,
    },
    /// Send programs from a .syx file to the synth.
    Send {
        file: PathBuf,
        /// Only send programs stored at these locations in the file.
        #[arg(short, long)]
        select: Option<String>,
        /// Write to consecutive slots starting here instead of the original locations.
        #[arg(long, conflicts_with = "audition")]
        to: Option<Location>,
        /// Load a single program into the edit buffer without storing it.
        #[arg(long)]
        audition: bool,
        /// Pause after each write so the synth can store it.
        #[arg(long, default_value_t = 150)]
        delay_ms: u64,
        /// Read every written program back and compare it.
        #[arg(long)]
        verify: bool,
        /// Skip the confirmation prompt.
        #[arg(short, long)]
        yes: bool,
        /// Don't back up the slots that will be overwritten (not recommended).
        #[arg(long)]
        no_backup: bool,
        /// Where automatic backups are written.
        #[arg(long, default_value = "backups")]
        backup_dir: PathBuf,
    },
    /// Show the disclaimer and limitation of liability.
    Disclaimer,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let connect = || -> Result<Synth> {
        let input = cli.in_port.as_deref().unwrap_or(&cli.port);
        let output = cli.out_port.as_deref().unwrap_or(&cli.port);
        let mut synth = Synth::connect(input, output)?;
        synth.timeout = Duration::from_secs_f64(cli.timeout);
        Ok(synth)
    };

    match &cli.command {
        Command::Ports => ports(),
        Command::Identify => identify(&mut connect()?),
        Command::Backup {
            select,
            output,
            split,
        } => backup(
            &mut connect()?,
            &parse_selection(select)?,
            output.as_deref(),
            *split,
        ),
        Command::EditBuffer { output } => edit_buffer(&mut connect()?, output),
        Command::List { files, layers } => list(files, *layers),
        Command::Disclaimer => {
            print!("{DISCLAIMER}");
            Ok(())
        }
        Command::Send {
            file,
            select,
            to,
            audition,
            delay_ms,
            verify,
            yes,
            no_backup,
            backup_dir,
        } => {
            let items = load_programs(file, select.as_deref())?;
            if *audition {
                send_audition(&mut connect()?, &items)
            } else {
                let plan = plan_writes(items, *to)?;
                let options = SendOptions {
                    delay: Duration::from_millis(*delay_ms),
                    verify: *verify,
                    confirm: !*yes,
                    backup_dir: (!*no_backup).then_some(backup_dir.as_path()),
                };
                send(&mut connect()?, &plan, &options)
            }
        }
    }
}

fn ports() -> Result<()> {
    let ports = p12_midi::list_ports()?;
    println!("Inputs:");
    for (i, name) in ports.inputs.iter().enumerate() {
        println!("  {i}: {name}");
    }
    println!("Outputs:");
    for (i, name) in ports.outputs.iter().enumerate() {
        println!("  {i}: {name}");
    }
    Ok(())
}

fn identify(synth: &mut Synth) -> Result<()> {
    // OS 1.4.2.3 reports 1.4.2: the last part is only on the Global screen.
    let version = synth.identify()?;
    println!("Prophet 12 found, OS {}", os_version_text(&version));
    let globals = synth.fetch_globals()?;
    match global_parameter(&globals, GLOBAL_MIDI_CHANNEL) {
        Some(0) => println!("MIDI channel: All"),
        Some(channel) => println!("MIDI channel: {channel}"),
        None => println!("MIDI channel: unknown (short Global parameter dump)"),
    }
    Ok(())
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn progress(done: usize, total: usize, text: &str) {
    eprint!("\r\x1b[2K[{done}/{total}] {text}");
    let _ = io::stderr().flush();
    if done == total {
        eprintln!();
    }
}

fn fetch_all(
    synth: &mut Synth,
    locations: &BTreeSet<Location>,
) -> Result<Vec<(Location, Program)>> {
    let mut programs = Vec::with_capacity(locations.len());
    for (i, &location) in locations.iter().enumerate() {
        let program = synth.fetch_program(location)?;
        progress(
            i + 1,
            locations.len(),
            &format!("{location} {}", program.name()),
        );
        programs.push((location, program));
    }
    Ok(programs)
}

fn write_syx(path: &Path, messages: &[Message]) -> Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    fs::write(path, syx::encode(messages)).with_context(|| format!("writing {}", path.display()))
}

fn backup(
    synth: &mut Synth,
    select: &BTreeSet<Location>,
    output: Option<&Path>,
    split: bool,
) -> Result<()> {
    let programs = fetch_all(synth, select)?;
    let default_name = format!(
        "p12-backup-{}{}",
        timestamp(),
        if split { "" } else { ".syx" }
    );
    let output = output.map_or_else(|| PathBuf::from(default_name), Path::to_path_buf);

    if split {
        for (location, program) in &programs {
            let file = syx::file_name(Some(*location), program);
            let message = Message::ProgramDump {
                location: *location,
                program: program.clone(),
            };
            write_syx(&output.join(file), &[message])?;
        }
    } else {
        let messages: Vec<_> = programs
            .into_iter()
            .map(|(location, program)| Message::ProgramDump { location, program })
            .collect();
        write_syx(&output, &messages)?;
    }
    println!("Saved {} programs to {}", select.len(), output.display());
    Ok(())
}

fn edit_buffer(synth: &mut Synth, output: &Path) -> Result<()> {
    let program = synth.fetch_edit_buffer()?;
    let name = program.name();
    write_syx(output, &[Message::EditBufferDump(program)])?;
    println!("Saved edit buffer {name:?} to {}", output.display());
    Ok(())
}

fn read_entries(path: &Path) -> Result<Vec<Entry>> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(syx::parse(&bytes))
}

fn list(files: &[PathBuf], layers: bool) -> Result<()> {
    for path in files {
        println!("{}:", path.display());
        for entry in read_entries(path)? {
            match entry {
                Entry::Message(message) => {
                    let slot = match &message {
                        Message::ProgramDump { location, .. } => location.to_string(),
                        Message::EditBufferDump(_) => "edit".to_string(),
                        other => {
                            println!("  {other:?}");
                            continue;
                        }
                    };
                    let program = message.program().expect("dump messages carry a program");
                    match layers {
                        true => println!(
                            "  {slot:>5}  {:<20}  B: {}",
                            program.name(),
                            program.layer_b_name()
                        ),
                        false => println!("  {slot:>5}  {}", program.name()),
                    }
                }
                Entry::Unknown { raw, reason } => {
                    println!("  (skipped {} bytes: {reason})", raw.len())
                }
            }
        }
    }
    Ok(())
}

/// A program from a file, with its original location (None for edit buffer dumps).
struct FileProgram {
    source: Option<Location>,
    program: Program,
}

fn load_programs(path: &Path, select: Option<&str>) -> Result<Vec<FileProgram>> {
    let select = select.map(parse_selection).transpose()?;
    let items: Vec<FileProgram> = read_entries(path)?
        .into_iter()
        .filter_map(|entry| match entry {
            Entry::Message(Message::ProgramDump { location, program }) => Some(FileProgram {
                source: Some(location),
                program,
            }),
            Entry::Message(Message::EditBufferDump(program)) => Some(FileProgram {
                source: None,
                program,
            }),
            _ => None,
        })
        .filter(|item| match (&select, item.source) {
            (None, _) => true,
            (Some(sel), Some(location)) => sel.contains(&location),
            (Some(_), None) => false,
        })
        .collect();
    ensure!(
        !items.is_empty(),
        "no matching programs in {}",
        path.display()
    );
    Ok(items)
}

fn send_audition(synth: &mut Synth, items: &[FileProgram]) -> Result<()> {
    let [item] = items else {
        bail!(
            "--audition needs exactly one program, but {} were selected (use --select)",
            items.len()
        );
    };
    synth.send_edit_buffer(&item.program)?;
    println!(
        "Loaded {:?} into the edit buffer (nothing was stored)",
        item.program.name()
    );
    Ok(())
}

struct PlannedWrite {
    source: Option<Location>,
    target: Location,
    program: Program,
}

fn plan_writes(items: Vec<FileProgram>, to: Option<Location>) -> Result<Vec<PlannedWrite>> {
    let plan: Vec<PlannedWrite> = items
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let target = match to {
                Some(start) => start
                    .offset(i)
                    .with_context(|| format!("not enough slots after {start} for all selected programs"))?,
                None => item.source.context(
                    "the file contains an edit buffer dump, which has no location; use --to or --audition",
                )?,
            };
            Ok(PlannedWrite { source: item.source, target, program: item.program })
        })
        .collect::<Result<_>>()?;

    if let Some(w) = plan.iter().find(|w| !w.target.is_user()) {
        bail!(
            "refusing to write to {}: Factory banks are read-only",
            w.target
        );
    }
    let targets: BTreeSet<_> = plan.iter().map(|w| w.target).collect();
    ensure!(
        targets.len() == plan.len(),
        "several programs would be written to the same slot"
    );
    Ok(plan)
}

struct SendOptions<'a> {
    delay: Duration,
    verify: bool,
    confirm: bool,
    backup_dir: Option<&'a Path>,
}

fn send(synth: &mut Synth, plan: &[PlannedWrite], options: &SendOptions) -> Result<()> {
    let targets: BTreeSet<_> = plan.iter().map(|w| w.target).collect();

    let current = match options.backup_dir {
        Some(dir) => {
            eprintln!(
                "Backing up the {} slots that will be overwritten...",
                targets.len()
            );
            let current = fetch_all(synth, &targets)?;
            let path = dir.join(format!("before-send-{}.syx", timestamp()));
            let messages: Vec<_> = current
                .iter()
                .map(|(location, program)| Message::ProgramDump {
                    location: *location,
                    program: program.clone(),
                })
                .collect();
            write_syx(&path, &messages)?;
            eprintln!("Backup saved to {}", path.display());
            current
        }
        None => Vec::new(),
    };

    println!("About to write {} program(s):", plan.len());
    for w in plan {
        let from = w.source.map_or("edit".to_string(), |l| l.to_string());
        let replacing = current
            .iter()
            .find(|(l, _)| *l == w.target)
            .map(|(_, p)| format!("  (replaces {:?})", p.name()))
            .unwrap_or_default();
        println!(
            "  {from:>5} {:<20} -> {}{replacing}",
            w.program.name(),
            w.target
        );
    }
    println!("{RISK_NOTICE}");
    if options.confirm && !ask("Continue?")? {
        println!("Cancelled; nothing was written.");
        return Ok(());
    }

    for (i, w) in plan.iter().enumerate() {
        synth.write_program(w.target, &w.program, options.delay)?;
        progress(
            i + 1,
            plan.len(),
            &format!("{} {}", w.target, w.program.name()),
        );
    }

    if options.verify {
        let read_back = fetch_all(synth, &targets)?;
        let mismatches: Vec<_> = plan
            .iter()
            .filter(|w| {
                read_back
                    .iter()
                    .any(|(l, p)| *l == w.target && *p != w.program)
            })
            .map(|w| w.target.to_string())
            .collect();
        ensure!(
            mismatches.is_empty(),
            "verification failed for: {}",
            mismatches.join(", ")
        );
        println!("Verified {} program(s)", plan.len());
    }
    println!("Done.");
    Ok(())
}

fn ask(question: &str) -> Result<bool> {
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn global_port_options_coexist_with_output() {
        let cli = Cli::try_parse_from(["p12", "--out", "2", "backup", "-o", "x.syx"]).unwrap();
        assert_eq!(cli.out_port.as_deref(), Some("2"));
        assert!(matches!(
            cli.command,
            Command::Backup {
                output: Some(_),
                ..
            }
        ));
    }
}
