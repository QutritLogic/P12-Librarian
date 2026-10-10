//! MIDI transport for a Prophet 12, built on `midir`.
//!
//! Requests are strictly one at a time: send a request, wait for the matching
//! reply (with a timeout), then move on. Flooding the synth with requests or
//! writes is the most common way bulk SysEx transfers go wrong.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use p12_core::sysex::{SYSEX_END, SYSEX_START};
use p12_core::{Location, Message, Program};

const CLIENT_NAME: &str = "p12-librarian";
/// Port name pattern used when the caller doesn't specify one.
pub const DEFAULT_PORT_PATTERN: &str = "prophet 12";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not initialise MIDI: {0}")]
    Init(#[from] midir::InitError),
    #[error("no MIDI {kind} port matches {pattern:?}; available: {available:?}")]
    NoPort {
        kind: &'static str,
        pattern: String,
        available: Vec<String>,
    },
    #[error("MIDI {kind} pattern {pattern:?} matches several ports: {matches:?}")]
    AmbiguousPort {
        kind: &'static str,
        pattern: String,
        matches: Vec<String>,
    },
    #[error("could not connect to {0}")]
    Connect(String),
    #[error("could not send MIDI: {0}")]
    Send(#[from] midir::SendError),
    #[error(
        "timed out waiting for {0} (is MIDI SysEx Enable on and SysEx Cable set to this port?)"
    )]
    Timeout(String),
    #[error("{0} is in a Factory bank, which is read-only")]
    FactoryBank(Location),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Ports {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub fn list_ports() -> Result<Ports> {
    let input = MidiInput::new(CLIENT_NAME)?;
    let output = MidiOutput::new(CLIENT_NAME)?;
    Ok(Ports {
        inputs: input
            .ports()
            .iter()
            .filter_map(|p| input.port_name(p).ok())
            .collect(),
        outputs: output
            .ports()
            .iter()
            .filter_map(|p| output.port_name(p).ok())
            .collect(),
    })
}

/// Picks the port whose name matches `pattern`: a port index, or a
/// case-insensitive substring that must match exactly one port.
fn select_port(kind: &'static str, names: &[String], pattern: &str) -> Result<usize> {
    if let Ok(index) = pattern.parse::<usize>()
        && index < names.len()
    {
        return Ok(index);
    }
    let needle = pattern.to_lowercase();
    let matches: Vec<usize> = (0..names.len())
        .filter(|&i| names[i].to_lowercase().contains(&needle))
        .collect();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(Error::NoPort {
            kind,
            pattern: pattern.into(),
            available: names.to_vec(),
        }),
        _ => Err(Error::AmbiguousPort {
            kind,
            pattern: pattern.into(),
            matches: matches.iter().map(|&i| names[i].clone()).collect(),
        }),
    }
}

/// Reassembles complete SysEx messages from the input stream. Some backends
/// deliver long SysEx in chunks, and realtime bytes may be interleaved.
struct SysexAssembler {
    buffer: Vec<u8>,
    active: bool,
}

impl SysexAssembler {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            active: false,
        }
    }

    fn feed(&mut self, bytes: &[u8], mut emit: impl FnMut(Vec<u8>)) {
        for &b in bytes {
            match b {
                SYSEX_START => {
                    self.buffer.clear();
                    self.buffer.push(b);
                    self.active = true;
                }
                SYSEX_END if self.active => {
                    self.buffer.push(b);
                    self.active = false;
                    emit(std::mem::take(&mut self.buffer));
                }
                0xF8..=0xFF => {} // realtime bytes may appear anywhere
                0x80..=0xF7 => self.active = false, // any other status aborts SysEx
                _ if self.active => self.buffer.push(b),
                _ => {}
            }
        }
    }
}

pub struct Synth {
    _input: MidiInputConnection<()>,
    output: MidiOutputConnection,
    incoming: Receiver<Vec<u8>>,
    /// How long to wait for each reply.
    pub timeout: Duration,
}

impl Synth {
    pub fn connect(input_pattern: &str, output_pattern: &str) -> Result<Self> {
        let mut input = MidiInput::new(CLIENT_NAME)?;
        input.ignore(Ignore::None); // midir drops SysEx by default
        let in_ports = input.ports();
        let in_names: Vec<String> = in_ports
            .iter()
            .filter_map(|p| input.port_name(p).ok())
            .collect();
        let in_port = &in_ports[select_port("input", &in_names, input_pattern)?];

        let output = MidiOutput::new(CLIENT_NAME)?;
        let out_ports = output.ports();
        let out_names: Vec<String> = out_ports
            .iter()
            .filter_map(|p| output.port_name(p).ok())
            .collect();
        let out_port = &out_ports[select_port("output", &out_names, output_pattern)?];

        let (tx, incoming) = mpsc::channel();
        let mut assembler = SysexAssembler::new();
        let input = input
            .connect(
                in_port,
                "p12-in",
                move |_timestamp, bytes, _| {
                    assembler.feed(bytes, |msg| {
                        let _ = tx.send(msg);
                    })
                },
                (),
            )
            .map_err(|e| Error::Connect(e.to_string()))?;
        let output = output
            .connect(out_port, "p12-out")
            .map_err(|e| Error::Connect(e.to_string()))?;

        Ok(Self {
            _input: input,
            output,
            incoming,
            timeout: Duration::from_secs(2),
        })
    }

    pub fn send(&mut self, message: &Message) -> Result<()> {
        self.output.send(&message.to_bytes())?;
        Ok(())
    }

    /// Sends `request` and waits for the first reply that `accept` maps to a value.
    fn request<T>(
        &mut self,
        request: &Message,
        what: &str,
        mut accept: impl FnMut(Message) -> Option<T>,
    ) -> Result<T> {
        while self.incoming.try_recv().is_ok() {} // discard stale messages
        self.send(request)?;
        let deadline = Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.incoming.recv_timeout(remaining) {
                Ok(bytes) => {
                    if let Some(value) = Message::parse(&bytes).ok().and_then(&mut accept) {
                        return Ok(value);
                    }
                }
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                    return Err(Error::Timeout(what.to_string()));
                }
            }
        }
    }

    /// Returns the raw OS version byte from the Device Inquiry reply.
    pub fn identify(&mut self) -> Result<u8> {
        self.request(
            &Message::DeviceInquiry,
            "device inquiry reply",
            |m| match m {
                Message::DeviceInquiryReply { version } => Some(version),
                _ => None,
            },
        )
    }

    /// Reads one stored program, retrying once on timeout.
    pub fn fetch_program(&mut self, location: Location) -> Result<Program> {
        let request = Message::RequestProgram(location);
        let what = format!("program {location}");
        let accept = |m| match m {
            Message::ProgramDump {
                location: l,
                program,
            } if l == location => Some(program),
            _ => None,
        };
        match self.request(&request, &what, accept) {
            Err(Error::Timeout(_)) => self.request(&request, &what, accept),
            result => result,
        }
    }

    pub fn fetch_edit_buffer(&mut self) -> Result<Program> {
        self.request(&Message::RequestEditBuffer, "edit buffer", |m| match m {
            Message::EditBufferDump(program) => Some(program),
            _ => None,
        })
    }

    /// Writes a program into a User slot, then waits `settle` so the synth
    /// can finish storing it before the next message arrives.
    pub fn write_program(
        &mut self,
        location: Location,
        program: &Program,
        settle: Duration,
    ) -> Result<()> {
        if !location.is_user() {
            return Err(Error::FactoryBank(location));
        }
        self.send(&Message::ProgramDump {
            location,
            program: program.clone(),
        })?;
        thread::sleep(settle);
        Ok(())
    }

    /// The Global parameters as sent by the synth: one byte per parameter.
    pub fn fetch_globals(&mut self) -> Result<Vec<u8>> {
        self.request(&Message::RequestGlobals, "global parameters", |m| match m {
            Message::GlobalsDump(nibbles) => Some(nibbles),
            _ => None,
        })
    }

    /// Sends a Note On; `channel` is 1-16.
    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<()> {
        self.output.send(&[
            0x90 | (channel.clamp(1, 16) - 1),
            note & 0x7F,
            velocity & 0x7F,
        ])?;
        Ok(())
    }

    /// Sends a Note Off; `channel` is 1-16.
    pub fn note_off(&mut self, channel: u8, note: u8) -> Result<()> {
        self.output
            .send(&[0x80 | (channel.clamp(1, 16) - 1), note & 0x7F, 0])?;
        Ok(())
    }

    /// Loads a program into the edit buffer for auditioning; nothing is stored.
    pub fn send_edit_buffer(&mut self, program: &Program) -> Result<()> {
        self.send(&Message::EditBufferDump(program.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(chunks: &[&[u8]]) -> Vec<Vec<u8>> {
        let mut assembler = SysexAssembler::new();
        let mut out = Vec::new();
        for chunk in chunks {
            assembler.feed(chunk, |m| out.push(m));
        }
        out
    }

    #[test]
    fn assembles_chunked_sysex_with_realtime() {
        let out = collect(&[
            &[0xF0, 0x01],
            &[0xF8, 0x2A, 0x06],
            &[0xF7, 0x90, 0x40, 0x7F],
        ]);
        assert_eq!(out, vec![vec![0xF0, 0x01, 0x2A, 0x06, 0xF7]]);
    }

    #[test]
    fn drops_sysex_aborted_by_status_byte() {
        let out = collect(&[&[0xF0, 0x01, 0x90, 0x40, 0xF7], &[0xF0, 0x02, 0xF7]]);
        assert_eq!(out, vec![vec![0xF0, 0x02, 0xF7]]);
    }

    #[test]
    fn selects_ports() {
        let names = [
            "Midi Through:0".to_string(),
            "Prophet 12:Prophet 12 MIDI 1 24:0".to_string(),
        ];
        assert_eq!(
            select_port("input", &names, DEFAULT_PORT_PATTERN).unwrap(),
            1
        );
        assert_eq!(select_port("input", &names, "0").unwrap(), 0);
        assert!(matches!(
            select_port("input", &names, "midi"),
            Err(Error::AmbiguousPort { .. })
        ));
        assert!(matches!(
            select_port("input", &names, "korg"),
            Err(Error::NoPort { .. })
        ));
    }
}
