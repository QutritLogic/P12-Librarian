//! A single unpacked Prophet 12 program (1024 bytes).

use std::fmt;
use std::ops::Range;

use crate::pack::{self, PROGRAM_SIZE};
use crate::{Error, Result, name};

/// Name byte ranges, found by inspecting real dumps (OS 1.x). Each layer
/// occupies 512 bytes (Layer B = Layer A + 512, like the NRPN numbering),
/// but byte offsets do NOT equal NRPN numbers: the name NRPNs are 480-499.
const LAYER_A_NAME: Range<usize> = 402..422;
const LAYER_B_NAME: Range<usize> = 914..934;

/// Each layer is one half of the program. The synth's own "Copy A -> B"
/// makes the B half an exact copy of the A half, name included (verified by
/// diffing edit buffer dumps before and after it).
const LAYER_SIZE: usize = PROGRAM_SIZE / 2;

/// Offsets (within a layer) that hold the same value in both layers of every
/// real program examined, even when the layers differ: probably program-wide
/// settings stored twice. Copying a layer from another program keeps the
/// target's own values here so the two copies stay in step.
const MIRRORED_OFFSETS: [usize; 2] = [23, 293];

#[derive(Clone, PartialEq, Eq)]
pub struct Program {
    data: Box<[u8; PROGRAM_SIZE]>,
}

impl Program {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let data: [u8; PROGRAM_SIZE] = bytes
            .try_into()
            .map_err(|_| Error::WrongProgramSize(bytes.len()))?;
        Ok(Self {
            data: Box::new(data),
        })
    }

    pub fn from_packed(packed: &[u8]) -> Result<Self> {
        Self::from_bytes(&pack::unpack(packed)?)
    }

    pub fn to_packed(&self) -> Vec<u8> {
        pack::pack(&self.data[..])
    }

    pub fn as_bytes(&self) -> &[u8; PROGRAM_SIZE] {
        &self.data
    }

    /// The program name as shown on the panel (Layer A).
    pub fn name(&self) -> String {
        decode_name(&self.data[LAYER_A_NAME])
    }

    pub fn layer_b_name(&self) -> String {
        decode_name(&self.data[LAYER_B_NAME])
    }

    /// This program with its Layer B replaced by `source`'s Layer A (sound and
    /// name), like the synth's "Copy A -> B" across two programs. Layer A is
    /// unchanged.
    pub fn with_layer_b_from(&self, source: &Program) -> Program {
        let mut data = self.data.clone();
        data[LAYER_SIZE..].copy_from_slice(&source.data[..LAYER_SIZE]);
        for offset in MIRRORED_OFFSETS {
            data[LAYER_SIZE + offset] = self.data[offset];
        }
        Program { data }
    }

    /// Sets the Layer A (panel) name, padded with spaces as the synth does.
    pub fn set_name(&mut self, name: &str) -> Result<()> {
        name::validate(name)?;
        let field = &mut self.data[LAYER_A_NAME];
        field.fill(b' ');
        field[..name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }
}

fn decode_name(bytes: &[u8]) -> String {
    let name: String = bytes
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| {
            if (0x20..0x7F).contains(&b) {
                b as char
            } else {
                '?'
            }
        })
        .collect();
    name.trim_end().to_string()
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

/// Test helper: a blank program with the given Layer A name.
#[cfg(test)]
pub(crate) fn named(name: &str) -> Program {
    let mut data = [0u8; PROGRAM_SIZE];
    data[LAYER_A_NAME].fill(b' ');
    data[LAYER_A_NAME.start..][..name.len()].copy_from_slice(name.as_bytes());
    Program::from_bytes(&data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_padded_name() {
        assert_eq!(named("Brass Pad").name(), "Brass Pad");
    }

    #[test]
    fn names_at_offsets_seen_in_real_dumps() {
        let mut data = [0u8; PROGRAM_SIZE];
        data[402..422].copy_from_slice(b"BOF Cascade 5th     ");
        data[914..934].copy_from_slice(b"Basic Program B     ");
        let program = Program::from_bytes(&data).unwrap();
        assert_eq!(program.name(), "BOF Cascade 5th");
        assert_eq!(program.layer_b_name(), "Basic Program B");
    }

    #[test]
    fn set_name_pads_and_validates() {
        let mut program = named("A Much Longer Name");
        program.set_name("Short").unwrap();
        assert_eq!(&program.as_bytes()[LAYER_A_NAME], b"Short               ");
        assert_eq!(program.name(), "Short");
        assert!(program.set_name("No/Slash").is_err());
        assert_eq!(program.name(), "Short");
    }

    #[test]
    fn copies_source_layer_a_into_layer_b() {
        let mut target = [1u8; PROGRAM_SIZE];
        target[..LAYER_SIZE].fill(2);
        target[23] = 10;
        target[293] = 3;
        target[LAYER_A_NAME].copy_from_slice(b"Target              ");
        let target = Program::from_bytes(&target).unwrap();
        let mut source = [7u8; PROGRAM_SIZE];
        source[23] = 9;
        source[LAYER_A_NAME].copy_from_slice(b"Source Lead         ");
        let source = Program::from_bytes(&source).unwrap();

        let result = target.with_layer_b_from(&source);
        assert_eq!(
            &result.as_bytes()[..LAYER_SIZE],
            &target.as_bytes()[..LAYER_SIZE]
        );
        assert_eq!(result.name(), "Target");
        assert_eq!(result.layer_b_name(), "Source Lead");
        for i in 0..LAYER_SIZE {
            let expected = if MIRRORED_OFFSETS.contains(&i) {
                target.as_bytes()[i]
            } else {
                source.as_bytes()[i]
            };
            assert_eq!(result.as_bytes()[LAYER_SIZE + i], expected, "offset {i}");
        }
    }

    #[test]
    fn copying_own_layer_matches_the_synths_copy_a_to_b() {
        // Within one program the mirrored bytes are already equal, so the
        // result is exactly what the synth produces: B half = A half.
        let mut data: Vec<u8> = (0..PROGRAM_SIZE).map(|i| (i % 251) as u8).collect();
        for offset in MIRRORED_OFFSETS {
            data[LAYER_SIZE + offset] = data[offset];
        }
        let program = Program::from_bytes(&data).unwrap();
        let copied = program.with_layer_b_from(&program);
        assert_eq!(
            &copied.as_bytes()[LAYER_SIZE..],
            &program.as_bytes()[..LAYER_SIZE]
        );
    }

    #[test]
    fn rejects_wrong_size() {
        assert_eq!(
            Program::from_bytes(&[0; 10]),
            Err(Error::WrongProgramSize(10))
        );
    }

    #[test]
    fn packed_round_trip() {
        let program = named("Round Trip");
        assert_eq!(Program::from_packed(&program.to_packed()).unwrap(), program);
    }
}
