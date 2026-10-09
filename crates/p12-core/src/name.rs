//! Program names: the allowed character set and "cleanify", which turns
//! lazily typed ALL-CAPS names into readable ones ("DISTPAD" → "DistPad").

use crate::{Error, Result};

pub const MAX_NAME_LEN: usize = 20;

/// Characters the Prophet 12 accepts in a name (manual p.82): printable
/// ASCII except `* / : < > ? \ | ~`.
pub fn is_allowed_char(c: char) -> bool {
    matches!(c, ' '..='}') && !"*/:<>?\\|".contains(c)
}

pub fn validate(name: &str) -> Result<()> {
    if name.chars().count() > MAX_NAME_LEN {
        return Err(Error::InvalidName(format!(
            "{name:?} is longer than {MAX_NAME_LEN} characters"
        )));
    }
    if let Some(c) = name.chars().find(|&c| !is_allowed_char(c)) {
        return Err(Error::InvalidName(format!(
            "{c:?} can't be used in a Prophet 12 name"
        )));
    }
    Ok(())
}

/// Kept in capitals when they appear as a whole word or a compound part.
const ACRONYMS: &[&str] = &[
    "adsr", "am", "bpm", "cs", "dsi", "dx", "ep", "fm", "fx", "ii", "iii", "iv", "jp", "lfo", "lh",
    "ms", "ob", "pm", "ppg", "pwm", "rh", "sfx", "sh", "tb", "vca", "vcf", "vco", "vi", "vii",
    "viii",
];

/// Words used to split run-together capitals. Plurals ending in "s" match
/// automatically. Kept to sound words so that ordinary English words are
/// rarely split by accident.
const WORDS: &[&str] = &[
    "acid", "air", "airy", "ambient", "analog", "arp", "atmo", "bass", "bell", "big", "bit",
    "blip", "bow", "bowed", "brass", "bright", "buzz", "choir", "chord", "chorus", "clav", "cold",
    "cosmic", "crush", "dark", "deep", "delay", "detune", "digital", "dist", "drive", "drone",
    "drum", "dream", "dub", "echo", "electric", "evolving", "fat", "filter", "flute", "funk",
    "fuzz", "glass", "glide", "glitch", "growl", "guitar", "harp", "hit", "hollow", "horn", "keys",
    "kick", "lead", "low", "lush", "metal", "mod", "mono", "noise", "organ", "pad", "perc",
    "phase", "piano", "pipe", "pluck", "poly", "pulse", "reed", "res", "reso", "saw", "seq",
    "shimmer", "sine", "soft", "solo", "space", "square", "stab", "step", "string", "sub", "sweep",
    "sync", "synth", "texture", "tine", "tri", "uni", "vox", "warm", "wave", "wide", "wind",
    "wobble",
];

fn is_acronym(word: &str) -> bool {
    ACRONYMS.contains(&word)
}

fn is_word(word: &str) -> bool {
    let known = |w: &str| WORDS.contains(&w) || is_acronym(w);
    known(word)
        || word
            .strip_suffix('s')
            .is_some_and(|w| w.len() >= 2 && known(w))
}

/// Splits a lowercase run into known words, using as few parts as possible.
/// Returns `None` if it can't be split completely.
fn split_words(run: &str) -> Option<Vec<&str>> {
    // best[i] = fewest parts covering run[..i], and where the last part starts.
    let mut best: Vec<Option<(usize, usize)>> = vec![None; run.len() + 1];
    best[0] = Some((0, 0));
    for end in 1..=run.len() {
        for start in 0..end {
            let Some((parts, _)) = best[start] else {
                continue;
            };
            if is_word(&run[start..end]) && best[end].is_none_or(|(p, _)| parts + 1 < p) {
                best[end] = Some((parts + 1, start));
            }
        }
    }
    best[run.len()]?;
    let mut parts = Vec::new();
    let mut end = run.len();
    while end > 0 {
        let (_, start) = best[end].expect("reachable positions have a predecessor");
        parts.push(&run[start..end]);
        end = start;
    }
    parts.reverse();
    Some(parts)
}

fn capitalise(word: &str) -> String {
    if is_acronym(word) {
        return word.to_ascii_uppercase();
    }
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_uppercase().to_string() + chars.as_str()
    })
}

/// Rewrites one all-capitals run of letters.
fn clean_run(run: &str) -> String {
    let lower = run.to_ascii_lowercase();
    if run.len() < 2 || run.bytes().any(|b| b.is_ascii_lowercase()) || is_acronym(&lower) {
        return run.to_string();
    }
    match split_words(&lower) {
        Some(parts) if parts.len() > 1 => parts.into_iter().map(capitalise).collect(),
        _ => capitalise(&lower),
    }
}

/// Turns ALL-CAPS words into capitalised ones and splits run-together sound
/// words: `"BOF COOL PAD"` → `"Bof Cool Pad"`, `"DISTPAD"` → `"DistPad"`.
/// Words that already contain lowercase letters are left alone, and the
/// length never changes, so a valid name stays valid.
pub fn cleanify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut run = String::new();
    for c in name.chars() {
        if c.is_ascii_alphabetic() {
            run.push(c);
        } else {
            out += &clean_run(&run);
            run.clear();
            out.push(c);
        }
    }
    out += &clean_run(&run);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_from_the_request() {
        assert_eq!(cleanify("BOF COOL PAD"), "Bof Cool Pad");
        assert_eq!(cleanify("DISTPAD"), "DistPad");
    }

    #[test]
    fn real_patch_names() {
        assert_eq!(cleanify("BOF Cascade 5th"), "Bof Cascade 5th");
        assert_eq!(cleanify("MAJOR 6 9"), "Major 6 9");
        assert_eq!(cleanify("LFO MADNESS"), "LFO Madness");
        assert_eq!(cleanify("BOF JUPITER"), "Bof Jupiter");
        assert_eq!(cleanify("Kepler"), "Kepler");
        assert_eq!(cleanify("KeplerRH"), "KeplerRH");
    }

    #[test]
    fn splits_compounds_with_plurals_and_acronyms() {
        assert_eq!(cleanify("WARMSTRINGS"), "WarmStrings");
        assert_eq!(cleanify("FMBELLS 2"), "FMBells 2");
        assert_eq!(cleanify("SUBBASS-DIST"), "SubBass-Dist");
        assert_eq!(cleanify("PAD2"), "Pad2");
    }

    #[test]
    fn leaves_what_it_cannot_improve() {
        assert_eq!(cleanify("CASCADE"), "Cascade"); // no full split: just capitalised
        assert_eq!(cleanify("PAD A"), "Pad A"); // single letters stay
        assert_eq!(cleanify("Bright PAD"), "Bright Pad");
        assert_eq!(cleanify(""), "");
        for name in ["BOF COOL PAD", "x", "SUBBASS-DIST  "] {
            assert_eq!(cleanify(name).len(), name.len());
        }
    }

    #[test]
    fn validates_names() {
        assert!(validate("Bof Cool Pad").is_ok());
        assert!(validate("{Glass} #2 (soft)").is_ok());
        assert!(validate("12345678901234567890").is_ok());
        assert!(validate("123456789012345678901").is_err());
        for bad in ["a/b", "a*b", "a:b", "a?b", "a~b", "a|b", "å"] {
            assert!(validate(bad).is_err(), "{bad}");
        }
    }
}
