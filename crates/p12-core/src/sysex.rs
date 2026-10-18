//! Prophet 12 SysEx messages (manual p.86–88).

use crate::pack::PACKED_PROGRAM_SIZE;
use crate::{Error, Location, Program, Result};

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_END: u8 = 0xF7;
/// DSI / Sequential manufacturer ID (a single byte).
pub const DSI_ID: u8 = 0x01;
pub const PROPHET_12_ID: u8 = 0x2A;

const PROGRAM_DUMP: u8 = 0x02;
const EDIT_BUFFER_DUMP: u8 = 0x03;
const REQUEST_PROGRAM: u8 = 0x05;
const REQUEST_EDIT_BUFFER: u8 = 0x06;
const REQUEST_GLOBALS: u8 = 0x0E;
const GLOBALS_DUMP: u8 = 0x0F;

const UNIVERSAL_NON_REALTIME: u8 = 0x7E;
const ALL_CHANNELS: u8 = 0x7F;
const GENERAL_INFO: u8 = 0x06;
const INQUIRY_REQUEST: u8 = 0x01;
const INQUIRY_REPLY: u8 = 0x02;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// A stored program; sending one writes it to that location.
    ProgramDump {
        location: Location,
        program: Program,
    },
    /// The current (unsaved) program; sending one only loads it for playing.
    EditBufferDump(Program),
    /// One byte per Global parameter (see `global_parameter`). Transmit-only
    /// on the synth.
    GlobalsDump(Vec<u8>),
    RequestProgram(Location),
    RequestEditBuffer,
    RequestGlobals,
    /// Universal Device Inquiry, addressed to all channels.
    DeviceInquiry,
    /// Reply to a Device Inquiry. `version` holds the OS version bytes after
    /// the family and model: OS 1.4.2.3 sends `[1, 4, 2]` (the last part isn't
    /// sent).
    DeviceInquiryReply {
        version: Vec<u8>,
    },
}

/// The OS version bytes from a Device Inquiry reply as text, e.g. "1.4.2".
pub fn os_version_text(version: &[u8]) -> String {
    version
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

/// Index of the MIDI Channel parameter (NRPN 1026) in a Global Parameters
/// dump: 0 = All, 1-16 = that channel.
pub const GLOBAL_MIDI_CHANNEL: usize = 2;

/// Global parameter `index` (NRPN 1024 + `index`) from the data of a
/// Global Parameters dump. The manual describes 50 nibbles, but OS 1.4.2.3
/// sends one byte per parameter (26 bytes), verified on a real synth.
pub fn global_parameter(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}

impl Message {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let body = bytes
            .strip_prefix(&[SYSEX_START])
            .and_then(|b| b.strip_suffix(&[SYSEX_END]))
            .ok_or(Error::NotSysex)?;
        match body {
            [UNIVERSAL_NON_REALTIME, _channel, GENERAL_INFO, rest @ ..] => parse_inquiry(rest),
            [DSI_ID, PROPHET_12_ID, kind, rest @ ..] => parse_p12(*kind, rest),
            _ => Err(Error::NotProphet12),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![SYSEX_START];
        match self {
            Self::DeviceInquiry => out.extend([
                UNIVERSAL_NON_REALTIME,
                ALL_CHANNELS,
                GENERAL_INFO,
                INQUIRY_REQUEST,
            ]),
            Self::DeviceInquiryReply { version } => {
                out.extend([
                    UNIVERSAL_NON_REALTIME,
                    ALL_CHANNELS,
                    GENERAL_INFO,
                    INQUIRY_REPLY,
                    DSI_ID,
                    PROPHET_12_ID,
                    0x01,
                    0x00,
                    0x00,
                ]);
                out.extend(version);
            }
            _ => {
                out.extend([DSI_ID, PROPHET_12_ID]);
                match self {
                    Self::ProgramDump { location, program } => {
                        out.extend([PROGRAM_DUMP, location.bank(), location.program()]);
                        out.extend(program.to_packed());
                    }
                    Self::EditBufferDump(program) => {
                        out.push(EDIT_BUFFER_DUMP);
                        out.extend(program.to_packed());
                    }
                    Self::GlobalsDump(nibbles) => {
                        out.push(GLOBALS_DUMP);
                        out.extend(nibbles);
                    }
                    Self::RequestProgram(location) => {
                        out.extend([REQUEST_PROGRAM, location.bank(), location.program()])
                    }
                    Self::RequestEditBuffer => out.push(REQUEST_EDIT_BUFFER),
                    Self::RequestGlobals => out.push(REQUEST_GLOBALS),
                    Self::DeviceInquiry | Self::DeviceInquiryReply { .. } => unreachable!(),
                }
            }
        }
        out.push(SYSEX_END);
        out
    }

    /// The program carried by a dump message, if any.
    pub fn program(&self) -> Option<&Program> {
        match self {
            Self::ProgramDump { program, .. } | Self::EditBufferDump(program) => Some(program),
            _ => None,
        }
    }
}

fn parse_inquiry(rest: &[u8]) -> Result<Message> {
    match rest {
        [INQUIRY_REQUEST] => Ok(Message::DeviceInquiry),
        [INQUIRY_REPLY, DSI_ID, PROPHET_12_ID, _, _, _, version @ ..] if !version.is_empty() => {
            Ok(Message::DeviceInquiryReply {
                version: version.to_vec(),
            })
        }
        [INQUIRY_REPLY, ..] => Err(Error::NotProphet12),
        _ => Err(Error::Malformed("unrecognised device inquiry message")),
    }
}

fn parse_p12(kind: u8, rest: &[u8]) -> Result<Message> {
    match (kind, rest) {
        (PROGRAM_DUMP, [bank, program, data @ ..]) => Ok(Message::ProgramDump {
            location: Location::new(*bank, *program)?,
            program: program_data(data)?,
        }),
        (EDIT_BUFFER_DUMP, data) => Ok(Message::EditBufferDump(program_data(data)?)),
        (GLOBALS_DUMP, data) => Ok(Message::GlobalsDump(data.to_vec())),
        (REQUEST_PROGRAM, [bank, program]) => {
            Ok(Message::RequestProgram(Location::new(*bank, *program)?))
        }
        (REQUEST_EDIT_BUFFER, []) => Ok(Message::RequestEditBuffer),
        (REQUEST_GLOBALS, []) => Ok(Message::RequestGlobals),
        (PROGRAM_DUMP | REQUEST_PROGRAM | REQUEST_EDIT_BUFFER | REQUEST_GLOBALS, _) => {
            Err(Error::Malformed("wrong message length"))
        }
        _ => Err(Error::UnknownMessageType(kind)),
    }
}

fn program_data(packed: &[u8]) -> Result<Program> {
    if packed.len() != PACKED_PROGRAM_SIZE {
        return Err(Error::Malformed("program data is not 1171 bytes"));
    }
    Program::from_packed(packed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::named;

    #[test]
    fn requests_match_manual() {
        let loc: Location = "U2:05".parse().unwrap();
        assert_eq!(
            Message::RequestProgram(loc).to_bytes(),
            [0xF0, 0x01, 0x2A, 0x05, 1, 4, 0xF7]
        );
        assert_eq!(
            Message::RequestEditBuffer.to_bytes(),
            [0xF0, 0x01, 0x2A, 0x06, 0xF7]
        );
        assert_eq!(
            Message::RequestGlobals.to_bytes(),
            [0xF0, 0x01, 0x2A, 0x0E, 0xF7]
        );
        assert_eq!(
            Message::DeviceInquiry.to_bytes(),
            [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]
        );
    }

    #[test]
    fn program_dump_round_trip() {
        let msg = Message::ProgramDump {
            location: "F4:99".parse().unwrap(),
            program: named("Strings"),
        };
        let bytes = msg.to_bytes();
        assert_eq!(bytes.len(), 3 + 3 + 1171 + 1);
        assert_eq!(&bytes[..6], &[0xF0, 0x01, 0x2A, 0x02, 7, 98]);
        assert_eq!(Message::parse(&bytes).unwrap(), msg);
    }

    #[test]
    fn edit_buffer_round_trip() {
        let msg = Message::EditBufferDump(named("Lead"));
        assert_eq!(Message::parse(&msg.to_bytes()).unwrap(), msg);
    }

    #[test]
    fn parses_inquiry_reply() {
        // The reply from a Prophet 12 Keyboard running OS 1.4.2.3.
        let reply = [
            0xF0, 0x7E, 0x7F, 0x06, 0x02, 0x01, 0x2A, 0x01, 0x00, 0x00, 0x01, 0x04, 0x02, 0xF7,
        ];
        let msg = Message::parse(&reply).unwrap();
        assert_eq!(
            msg,
            Message::DeviceInquiryReply {
                version: vec![1, 4, 2]
            }
        );
        assert_eq!(os_version_text(&[1, 4, 2]), "1.4.2");
        assert_eq!(Message::parse(&msg.to_bytes()).unwrap(), msg);
    }

    #[test]
    fn reads_global_parameters() {
        // The start of a real dump: coarse tune 12, fine tune 50, MIDI channel All.
        let data = [0x0C, 0x32, 0x00];
        assert_eq!(global_parameter(&data, 0), Some(12));
        assert_eq!(global_parameter(&data, 1), Some(50));
        assert_eq!(global_parameter(&data, GLOBAL_MIDI_CHANNEL), Some(0));
        assert_eq!(global_parameter(&data, 3), None);
    }

    #[test]
    fn rejects_foreign_and_truncated() {
        assert_eq!(
            Message::parse(&[0xF0, 0x43, 0x10, 0xF7]),
            Err(Error::NotProphet12)
        );
        assert_eq!(
            Message::parse(&[0xF0, 0x01, 0x2A, 0x05]),
            Err(Error::NotSysex)
        );
        let mut truncated = Message::EditBufferDump(named("x")).to_bytes();
        truncated.remove(10);
        assert!(matches!(
            Message::parse(&truncated),
            Err(Error::Malformed(_))
        ));
    }
}
