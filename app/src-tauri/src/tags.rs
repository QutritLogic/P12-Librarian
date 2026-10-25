//! Tags and favourites, kept by sound (`Program::fingerprint`) in tags.json
//! next to settings.json. Keyed by sound rather than by slot or name, they
//! follow a patch through swaps, writes, renames, files and backups.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Longest tag kept; longer ones are cut.
const MAX_TAG_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub favourite: bool,
    /// The name it had when last tagged, so the file can be read by hand.
    #[serde(default)]
    pub name: String,
}

impl Entry {
    fn is_empty(&self) -> bool {
        self.tags.is_empty() && !self.favourite
    }
}

type Tags = BTreeMap<String, Entry>;

#[derive(Serialize, Deserialize)]
struct TagFile {
    version: u32,
    #[serde(default)]
    patches: Tags,
}

/// The tags, loaded from tags.json on first use.
#[derive(Default)]
pub struct TagState(Mutex<Option<Tags>>);

/// One patch's change: tags to add and remove, and optionally its favourite mark.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    fingerprint: String,
    name: String,
    #[serde(default)]
    add: Vec<String>,
    #[serde(default)]
    remove: Vec<String>,
    favourite: Option<bool>,
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("tags.json"))
}

fn load(app: &AppHandle) -> Tags {
    path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|json| serde_json::from_str::<TagFile>(&json).ok())
        .map(|file| file.patches)
        .unwrap_or_default()
}

/// Writes tags.json through a temporary file, so a crash can't leave half a file.
fn save(app: &AppHandle, tags: &Tags) -> Result<(), String> {
    let path = path(app).ok_or("No folder for the tags file")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Saving tags: {e}"))?;
    }
    let file = TagFile {
        version: 1,
        patches: tags.clone(),
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, json)
        .and_then(|()| fs::rename(&temp, &path))
        .map_err(|e| format!("Saving tags to {}: {e}", path.display()))
}

/// A tag as stored: trimmed, without a leading `#`, with spaces turned into
/// hyphens (so `#tag` search can find it) and at most 32 characters. None if
/// nothing is left.
pub fn clean_tag(tag: &str) -> Option<String> {
    let words: Vec<&str> = tag
        .trim()
        .trim_start_matches('#')
        .split_whitespace()
        .collect();
    let tag: String = words.join("-").chars().take(MAX_TAG_LENGTH).collect();
    (!tag.is_empty()).then_some(tag)
}

fn same_tag(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

fn apply(tags: &mut Tags, change: Change) {
    let entry = tags.entry(change.fingerprint.clone()).or_default();
    entry.name = change.name;
    for tag in change.remove.iter().filter_map(|t| clean_tag(t)) {
        entry.tags.retain(|t| !same_tag(t, &tag));
    }
    for tag in change.add.iter().filter_map(|t| clean_tag(t)) {
        if !entry.tags.iter().any(|t| same_tag(t, &tag)) {
            entry.tags.push(tag);
        }
    }
    entry.tags.sort_by_key(|t| t.to_lowercase());
    if let Some(favourite) = change.favourite {
        entry.favourite = favourite;
    }
    if entry.is_empty() {
        tags.remove(&change.fingerprint);
    }
}

/// All tags and favourites, by fingerprint.
#[tauri::command]
pub fn get_tags(app: AppHandle, state: tauri::State<'_, TagState>) -> Tags {
    let mut tags = state.0.lock().unwrap_or_else(|p| p.into_inner());
    tags.get_or_insert_with(|| load(&app)).clone()
}

/// Applies changes, saves them and returns all tags and favourites.
#[tauri::command]
pub fn update_tags(
    app: AppHandle,
    state: tauri::State<'_, TagState>,
    changes: Vec<Change>,
) -> Result<Tags, String> {
    let mut guard = state.0.lock().unwrap_or_else(|p| p.into_inner());
    let mut tags = guard.get_or_insert_with(|| load(&app)).clone();
    for change in changes {
        apply(&mut tags, change);
    }
    save(&app, &tags)?;
    *guard = Some(tags.clone());
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(add: &[&str], remove: &[&str], favourite: Option<bool>) -> Change {
        Change {
            fingerprint: "f".into(),
            name: "Pad".into(),
            add: add.iter().map(|s| s.to_string()).collect(),
            remove: remove.iter().map(|s| s.to_string()).collect(),
            favourite,
        }
    }

    #[test]
    fn cleans_tags() {
        assert_eq!(clean_tag("  #Warm pad "), Some("Warm-pad".into()));
        assert_eq!(clean_tag("#"), None);
        assert_eq!(clean_tag("   "), None);
        assert_eq!(clean_tag(&"x".repeat(40)).unwrap().len(), 32);
    }

    #[test]
    fn adds_sorted_without_duplicates_and_removes_ignoring_case() {
        let mut tags = Tags::new();
        apply(&mut tags, change(&["pad", "Bass", "PAD"], &[], None));
        assert_eq!(tags["f"].tags, ["Bass", "pad"]);
        apply(&mut tags, change(&[], &["BASS"], Some(true)));
        assert_eq!(tags["f"].tags, ["pad"]);
        assert!(tags["f"].favourite);
    }

    #[test]
    fn drops_entries_left_without_tags_or_favourite() {
        let mut tags = Tags::new();
        apply(&mut tags, change(&["pad"], &[], Some(true)));
        apply(&mut tags, change(&[], &["pad"], Some(false)));
        assert!(tags.is_empty());
    }

    #[test]
    fn file_round_trip_leaves_out_empty_fields() {
        let mut tags = Tags::new();
        apply(&mut tags, change(&[], &[], Some(true)));
        let json = serde_json::to_string(&TagFile {
            version: 1,
            patches: tags.clone(),
        })
        .unwrap();
        assert_eq!(
            json,
            r#"{"version":1,"patches":{"f":{"favourite":true,"name":"Pad"}}}"#
        );
        let back: TagFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.patches, tags);
    }
}
