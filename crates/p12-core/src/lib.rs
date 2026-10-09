//! Prophet 12 SysEx protocol.
//!
//! Pure data handling with no I/O: the 7-in-8 packing scheme, the 1024-byte
//! program model, SysEx message parsing/encoding and `.syx` file handling.
//! Formats follow the *Prophet 12 Operation Manual v1.1*, Appendix D (p.86–89).

pub mod location;
pub mod name;
pub mod pack;
pub mod program;
pub mod sysex;
pub mod syx;

pub use location::{Location, parse_selection};
pub use program::Program;
pub use sysex::Message;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("bank {0} out of range (0-7)")]
    BankOutOfRange(u8),
    #[error("program {0} out of range (1-99)")]
    ProgramOutOfRange(u16),
    #[error("invalid location {0:?} (expected e.g. U1:05 or F3:99)")]
    InvalidLocation(String),
    #[error("invalid selection {0:?}")]
    InvalidSelection(String),
    #[error("packed data ends with a lone header byte")]
    TrailingHeader,
    #[error("byte at offset {0} has its high bit set; not MIDI-safe")]
    NotMidiSafe(usize),
    #[error("program data is {0} bytes, expected {expected}", expected = pack::PROGRAM_SIZE)]
    WrongProgramSize(usize),
    #[error("not a SysEx message (must start with F0 and end with F7)")]
    NotSysex,
    #[error("SysEx message is not for a Prophet 12")]
    NotProphet12,
    #[error("unsupported Prophet 12 message type {0:#04x}")]
    UnknownMessageType(u8),
    #[error("malformed message: {0}")]
    Malformed(&'static str),
    #[error("invalid name: {0}")]
    InvalidName(String),
}

pub type Result<T> = std::result::Result<T, Error>;
