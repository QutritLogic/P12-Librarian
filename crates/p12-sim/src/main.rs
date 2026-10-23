//! A simulated Prophet 12 for testing P12 Librarian without the synth. It
//! opens virtual MIDI ports named "Prophet 12 Simulator" and answers the
//! app's SysEx like the real synth: Device Inquiry, program and edit buffer
//! requests, the Global parameters, and program dumps written to the User
//! banks. Virtual ports exist on macOS (CoreMIDI) and Linux (ALSA), not on
//! Windows.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use p12_core::location::{BANKS, bank_name};
use p12_core::syx::{self, Entry};
use p12_core::{Location, Message, Program};

/// A simulated Prophet 12 on virtual MIDI ports, for testing P12 Librarian
/// without the synth. Start it, then start the app: it connects by itself.
/// Quit with Ctrl+C.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Name of the virtual MIDI ports. The app looks for "Prophet 12" in port names.
    #[arg(long, default_value = "Prophet 12 Simulator")]
    name: String,
    /// .syx files whose programs replace the generated ones in their slots,
    /// e.g. a backup made from a real synth.
    #[arg(long = "load", value_name = "FILE")]
    load: Vec<PathBuf>,
    /// Keep the simulator's programs in this .syx file: read at start if it
    /// exists, saved again after every write, so writes survive a restart.
    #[arg(long, value_name = "FILE")]
    state: Option<PathBuf>,
    /// Milliseconds before each reply, roughly like the real synth.
    #[arg(long, default_value_t = 15)]
    delay_ms: u64,
    /// OS version reported to the app.
    #[arg(long, default_value = "1.4.2")]
    os: String,
    /// Fail every Nth write: the slot keeps its old program, so the app's
    /// verification reports it. 0 = never.
    #[arg(long, default_value_t = 0, value_name = "N")]
    fail_every: usize,
    /// Also log every program read, not only writes and other requests.
    #[arg(short, long)]
    verbose: bool,
}

/// What the simulated synth holds.
struct Synth {
    programs: BTreeMap<Location, Program>,
    edit_buffer: Program,
    writes: usize,
}

// ───────────────────────────── Generated programs ─────────────────────────────

/// A small, fixed pseudo-random sequence, so every run starts the same.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
}

const ADJECTIVES: &[&str] = &[
    "Warm", "Glass", "Dark", "Soft", "Wide", "Bright", "Dusty", "Hollow", "Velvet", "Cold",
    "Analog", "Rising", "Broken", "Slow", "Deep", "Tiny", "Wobbly", "Shimmer", "Metal", "Airy",
];
const NOUNS: &[&str] = &[
    "Pad", "Bass", "Lead", "Bell", "Strings", "Brass", "Pluck", "Keys", "Choir", "Arp", "Drone",
    "Sweep", "Organ", "Sync", "Split", "Swell", "Noise", "Piano", "Flute", "Texture",
];

/// A program with made-up parameter values and a name, and a Layer B that's
/// either a copy of Layer A or another made-up layer.
fn generated_program(rng: &mut Rng) -> Program {
    let a = generated_layer(rng);
    if rng.next().is_multiple_of(3) {
        a.with_layer_b_from(&generated_layer(rng))
    } else {
        a.with_layer_b_from(&a)
    }
}

/// Made-up parameter values (0-127) with a made-up name.
fn generated_layer(rng: &mut Rng) -> Program {
    let bytes: Vec<u8> = (0..p12_core::pack::PROGRAM_SIZE)
        .map(|_| (rng.next() % 128) as u8)
        .collect();
    let mut program = Program::from_bytes(&bytes).expect("right size");
    let name = format!("{} {}", rng.pick(ADJECTIVES), rng.pick(NOUNS));
    program.set_name(&name).expect("valid name");
    program
}

/// All eight banks of made-up programs. The last ten slots of U4 repeat
/// programs from U1, some renamed, so the app finds duplicates.
fn generated_banks() -> BTreeMap<Location, Program> {
    let mut rng = Rng(0x5eed_1234_abcd_0012);
    let mut programs: BTreeMap<Location, Program> = (0..BANKS)
        .flat_map(Location::bank_locations)
        .map(|location| (location, generated_program(&mut rng)))
        .collect();
    for i in 0..10 {
        let from = Location::new(0, i * 3).expect("valid");
        let mut copy = programs[&from].clone();
        if i % 2 == 1 {
            copy.set_name(&format!("Copy {}", i + 1))
                .expect("valid name");
        }
        programs.insert(Location::new(3, 89 + i).expect("valid"), copy);
    }
    programs
}

/// Puts the programs stored in a .syx file into their slots. Returns how many.
fn load(programs: &mut BTreeMap<Location, Program>, path: &Path) -> Result<usize> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let mut count = 0;
    for entry in syx::parse(&bytes) {
        if let Entry::Message(Message::ProgramDump { location, program }) = entry {
            programs.insert(location, program);
            count += 1;
        }
    }
    Ok(count)
}

fn save(programs: &BTreeMap<Location, Program>, path: &Path) -> Result<()> {
    let messages: Vec<Message> = programs
        .iter()
        .map(|(location, program)| Message::ProgramDump {
            location: *location,
            program: program.clone(),
        })
        .collect();
    std::fs::write(path, syx::encode(&messages))
        .with_context(|| format!("writing {}", path.display()))
}

// ───────────────────────────── Answering the app ─────────────────────────────

/// One byte per Global parameter, as OS 1.4.2.3 sends them (26 bytes). Only
/// the MIDI channel (index 2) is used by the app: 1.
fn globals() -> Vec<u8> {
    let mut data = vec![0; 26];
    data[p12_core::sysex::GLOBAL_MIDI_CHANNEL] = 1;
    data
}

/// What the synth sends back for one incoming SysEx message, if anything.
fn answer(synth: &mut Synth, cli: &Cli, bytes: &[u8]) -> Option<Message> {
    let message = match Message::parse(bytes) {
        Ok(message) => message,
        Err(e) => {
            println!("  ignored a SysEx message ({e})");
            return None;
        }
    };
    match message {
        Message::DeviceInquiry => {
            println!("Device Inquiry: answering OS {}", cli.os);
            let version = cli.os.split('.').filter_map(|p| p.parse().ok()).collect();
            Some(Message::DeviceInquiryReply { version })
        }
        Message::RequestProgram(location) => {
            let program = synth.programs.get(&location)?.clone();
            if cli.verbose {
                println!("Read {location} {}", program.name());
            }
            Some(Message::ProgramDump { location, program })
        }
        Message::RequestEditBuffer => {
            println!("Edit buffer requested: {}", synth.edit_buffer.name());
            Some(Message::EditBufferDump(synth.edit_buffer.clone()))
        }
        Message::RequestGlobals => {
            println!("Global parameters requested");
            Some(Message::GlobalsDump(globals()))
        }
        Message::ProgramDump { location, program } => {
            synth.writes += 1;
            if !location.is_user() {
                println!("Write to {location} ignored: Factory banks are read-only");
            } else if cli.fail_every > 0 && synth.writes.is_multiple_of(cli.fail_every) {
                println!(
                    "Write to {location} {} DROPPED (--fail-every {})",
                    program.name(),
                    cli.fail_every
                );
            } else {
                println!("Wrote {location} {}", program.name());
                synth.programs.insert(location, program);
                if let Some(path) = &cli.state
                    && let Err(e) = save(&synth.programs, path)
                {
                    println!("  could not save the state: {e:#}");
                }
            }
            None
        }
        Message::EditBufferDump(program) => {
            println!("Loaded into the edit buffer: {}", program.name());
            synth.edit_buffer = program;
            None
        }
        Message::GlobalsDump(_) | Message::DeviceInquiryReply { .. } => None,
    }
}

#[cfg(unix)]
fn run(cli: Cli) -> Result<()> {
    use midir::os::unix::{VirtualInput, VirtualOutput};
    use midir::{Ignore, MidiInput, MidiOutput};
    use std::sync::mpsc;

    let mut programs = generated_banks();
    if let Some(path) = &cli.state
        && path.exists()
    {
        println!(
            "State: {} programs from {}",
            load(&mut programs, path)?,
            path.display()
        );
    }
    for path in &cli.load {
        println!(
            "Loaded {} programs from {}",
            load(&mut programs, path)?,
            path.display()
        );
    }
    let edit_buffer = programs[&Location::new(0, 0).expect("valid")].clone();
    let mut synth = Synth {
        programs,
        edit_buffer,
        writes: 0,
    };

    // The simulator's output is the app's input, and the other way round.
    let mut output = MidiOutput::new(&cli.name)?
        .create_virtual(&cli.name)
        .map_err(|e| anyhow::anyhow!("creating the virtual output port: {e}"))?;
    let (tx, incoming) = mpsc::channel();
    let mut assembler = p12_midi::SysexAssembler::new();
    let mut input = MidiInput::new(&cli.name)?;
    input.ignore(Ignore::None);
    let _input = input
        .create_virtual(
            &cli.name,
            move |_, bytes, _| {
                // Notes from auditioning come as plain MIDI, not SysEx.
                match bytes.first().map(|b| b & 0xF0) {
                    Some(0x90) if bytes.get(2).is_some_and(|&v| v > 0) => {
                        println!("Note on {} (channel {})", bytes[1], (bytes[0] & 0x0F) + 1)
                    }
                    _ => {}
                }
                assembler.feed(bytes, |message| {
                    let _ = tx.send(message);
                })
            },
            (),
        )
        .map_err(|e| anyhow::anyhow!("creating the virtual input port: {e}"))?;

    let banks: Vec<String> = (0..BANKS).map(bank_name).collect();
    println!(
        "Simulating a Prophet 12 on the MIDI ports \"{}\" ({}). Start P12 Librarian; it connects by itself. Ctrl+C quits.",
        cli.name,
        banks.join(" ")
    );
    for bytes in incoming {
        if let Some(reply) = answer(&mut synth, &cli, &bytes) {
            std::thread::sleep(Duration::from_millis(cli.delay_ms));
            output.send(&reply.to_bytes())?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn run(_cli: Cli) -> Result<()> {
    anyhow::bail!(
        "The simulator needs virtual MIDI ports, which Windows doesn't have. Use macOS or Linux."
    )
}

fn main() -> Result<()> {
    run(Cli::parse())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli() -> Cli {
        Cli::parse_from(["p12-sim", "--fail-every", "2"])
    }

    #[test]
    fn generated_banks_fill_every_slot_with_duplicates_in_u4() {
        let programs = generated_banks();
        assert_eq!(programs.len(), 8 * 99);
        let u1 = &programs[&Location::new(0, 0).unwrap()];
        let u4 = &programs[&Location::new(3, 89).unwrap()];
        assert_eq!(u1, u4);
        // Renamed copies are still the same sound.
        let renamed = &programs[&Location::new(3, 90).unwrap()];
        assert_eq!(renamed.name(), "Copy 2");
        assert_eq!(
            renamed.fingerprint(),
            programs[&Location::new(0, 3).unwrap()].fingerprint()
        );
        // The same every run.
        assert_eq!(
            generated_banks()[&Location::new(2, 50).unwrap()],
            programs[&Location::new(2, 50).unwrap()]
        );
    }

    #[test]
    fn answers_like_the_synth() {
        let cli = cli();
        let programs = generated_banks();
        let edit_buffer = programs[&Location::new(0, 0).unwrap()].clone();
        let mut synth = Synth {
            programs,
            edit_buffer,
            writes: 0,
        };
        let ask = |synth: &mut Synth, m: Message| answer(synth, &cli, &m.to_bytes());

        assert_eq!(
            ask(&mut synth, Message::DeviceInquiry),
            Some(Message::DeviceInquiryReply {
                version: vec![1, 4, 2]
            })
        );
        let u2 = Location::new(1, 4).unwrap();
        let Some(Message::ProgramDump { location, program }) =
            ask(&mut synth, Message::RequestProgram(u2))
        else {
            panic!("no program dump")
        };
        assert_eq!(location, u2);

        // Writes: the first is stored, the second dropped (--fail-every 2),
        // and Factory banks are never written.
        let mut new = program.clone();
        new.set_name("Written").unwrap();
        let u1 = Location::new(0, 0).unwrap();
        assert_eq!(
            ask(
                &mut synth,
                Message::ProgramDump {
                    location: u1,
                    program: new.clone()
                }
            ),
            None
        );
        assert_eq!(synth.programs[&u1].name(), "Written");
        assert_eq!(
            ask(
                &mut synth,
                Message::ProgramDump {
                    location: u2,
                    program: new.clone()
                }
            ),
            None
        );
        assert_ne!(synth.programs[&u2].name(), "Written");
        let f1 = Location::new(4, 0).unwrap();
        ask(
            &mut synth,
            Message::ProgramDump {
                location: f1,
                program: new.clone(),
            },
        );
        assert_ne!(synth.programs[&f1].name(), "Written");

        assert!(
            matches!(ask(&mut synth, Message::RequestGlobals), Some(Message::GlobalsDump(d)) if d[2] == 1)
        );
        ask(&mut synth, Message::EditBufferDump(new));
        assert!(
            matches!(ask(&mut synth, Message::RequestEditBuffer), Some(Message::EditBufferDump(p)) if p.name() == "Written")
        );
    }
}
