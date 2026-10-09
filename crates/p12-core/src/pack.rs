//! "Packed MS bit" format (manual p.89).
//!
//! Every 7 data bytes are sent as 8 MIDI-safe bytes: a leading byte holding
//! the stripped high bits (`0 G7 F7 E7 D7 C7 B7 A7`), then the 7 bytes with
//! their high bit cleared. A final partial group of n bytes becomes n + 1.

use crate::{Error, Result};

/// Size of one unpacked program.
pub const PROGRAM_SIZE: usize = 1024;
/// Size of one program once packed for transmission.
pub const PACKED_PROGRAM_SIZE: usize = packed_len(PROGRAM_SIZE);

pub const fn packed_len(unpacked: usize) -> usize {
    let rem = unpacked % 7;
    unpacked / 7 * 8 + if rem == 0 { 0 } else { rem + 1 }
}

pub fn pack(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(packed_len(data.len()));
    for group in data.chunks(7) {
        let high_bits = group
            .iter()
            .enumerate()
            .fold(0u8, |acc, (i, b)| acc | ((b >> 7) << i));
        out.push(high_bits);
        out.extend(group.iter().map(|b| b & 0x7F));
    }
    out
}

pub fn unpack(packed: &[u8]) -> Result<Vec<u8>> {
    if let Some(pos) = packed.iter().position(|b| b & 0x80 != 0) {
        return Err(Error::NotMidiSafe(pos));
    }
    let mut out = Vec::with_capacity(packed.len() / 8 * 7 + 7);
    for group in packed.chunks(8) {
        let (&high_bits, bytes) = group.split_first().expect("chunks are never empty");
        if bytes.is_empty() {
            return Err(Error::TrailingHeader);
        }
        out.extend(
            bytes
                .iter()
                .enumerate()
                .map(|(i, b)| b | (((high_bits >> i) & 1) << 7)),
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_packs_to_1171_bytes() {
        assert_eq!(PACKED_PROGRAM_SIZE, 1171);
        assert_eq!(pack(&[0u8; PROGRAM_SIZE]).len(), 1171);
    }

    #[test]
    fn matches_manual_bit_layout() {
        // A has its high bit set, G has its high bit set, others don't.
        let input = [0x81, 0x02, 0x03, 0x04, 0x05, 0x06, 0xFF];
        let packed = pack(&input);
        assert_eq!(packed[0], 0b0100_0001); // G7 at bit 6, A7 at bit 0
        assert_eq!(&packed[1..], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x7F]);
    }

    #[test]
    fn round_trips_all_byte_values() {
        let data: Vec<u8> = (0..PROGRAM_SIZE).map(|i| (i * 37 % 256) as u8).collect();
        let packed = pack(&data);
        assert!(packed.iter().all(|b| b & 0x80 == 0));
        assert_eq!(unpack(&packed).unwrap(), data);
    }

    #[test]
    fn round_trips_partial_groups() {
        for len in 0..=15 {
            let data: Vec<u8> = (0..len).map(|i| 0x80 | i as u8).collect();
            let packed = pack(&data);
            assert_eq!(packed.len(), packed_len(len));
            assert_eq!(unpack(&packed).unwrap(), data);
        }
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(unpack(&[0x00, 0x80]), Err(Error::NotMidiSafe(1)));
        assert_eq!(unpack(&[0; 9]), Err(Error::TrailingHeader));
    }
}
