//! `.syx` files: raw SysEx messages concatenated back to back.
//!
//! Messages that aren't recognised Prophet 12 messages (for example the Play
//! List data included in "Dump All User Banks") are kept as raw bytes so
//! nothing is silently lost.

use crate::sysex::{SYSEX_END, SYSEX_START};
use crate::{Error, Location, Message, Program};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Message(Message),
    Unknown { raw: Vec<u8>, reason: Error },
}

/// Splits a byte stream into complete `F0 … F7` messages. Bytes outside a
/// message are skipped; a message cut short by a new `F0` is dropped.
pub fn split(bytes: &[u8]) -> Vec<&[u8]> {
    let mut messages = Vec::new();
    let mut start = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            SYSEX_START => start = Some(i),
            SYSEX_END => {
                if let Some(s) = start.take() {
                    messages.push(&bytes[s..=i]);
                }
            }
            _ => {}
        }
    }
    messages
}

pub fn parse(bytes: &[u8]) -> Vec<Entry> {
    split(bytes)
        .into_iter()
        .map(|raw| match Message::parse(raw) {
            Ok(message) => Entry::Message(message),
            Err(reason) => Entry::Unknown {
                raw: raw.to_vec(),
                reason,
            },
        })
        .collect()
}

pub fn encode<'a>(messages: impl IntoIterator<Item = &'a Message>) -> Vec<u8> {
    messages.into_iter().flat_map(Message::to_bytes).collect()
}

/// A file name for a single program, e.g. `U1-05 BOF Cascade 5th.syx`.
pub fn file_name(location: Option<Location>, program: &Program) -> String {
    let slot = location.map_or("edit".to_string(), |l| l.to_string().replace(':', "-"));
    let name: String = program
        .name()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " -_".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    match name.trim() {
        "" => format!("{slot}.syx"),
        name => format!("{slot} {name}.syx"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::named;

    #[test]
    fn round_trips_and_keeps_unknown() {
        let a = Message::ProgramDump {
            location: "U1:01".parse().unwrap(),
            program: named("A"),
        };
        let b = Message::EditBufferDump(named("B"));
        let foreign = [0xF0, 0x43, 0x00, 0xF7];

        let mut bytes = vec![0x00, 0x12]; // junk before the first message
        bytes.extend(encode([&a]));
        bytes.extend(foreign);
        bytes.extend(encode([&b]));

        let entries = parse(&bytes);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], Entry::Message(a));
        assert!(matches!(&entries[1], Entry::Unknown { raw, .. } if raw == &foreign));
        assert_eq!(entries[2], Entry::Message(b));
    }

    #[test]
    fn file_names() {
        let loc = Some("U1:05".parse().unwrap());
        assert_eq!(
            file_name(loc, &named("BOF/Cascade 5th")),
            "U1-05 BOF_Cascade 5th.syx"
        );
        assert_eq!(file_name(None, &named("")), "edit.syx");
    }

    #[test]
    fn drops_message_interrupted_by_new_start() {
        let bytes = [0xF0, 0x01, 0x2A, 0xF0, 0x01, 0x2A, 0x06, 0xF7];
        assert_eq!(split(&bytes), vec![&bytes[3..]]);
    }
}
