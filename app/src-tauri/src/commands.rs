//! Tauri commands. Programs live here in the backend; the frontend only sees
//! `PatchSummary` values and refers to programs by id.
//!
//! MIDI calls block, so every command that talks to the synth runs on a
//! blocking thread and reports progress through `progress` events.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use p12_core::sysex::{GLOBAL_MIDI_CHANNEL, global_parameter};
use p12_core::syx::{self, Entry};
use p12_core::{Location, Message, Program, name, parse_selection};
use p12_midi::Synth;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::SettingsState;

/// Pause after each write so the synth can store the program (tested with OS 1.x).
const WRITE_SETTLE: Duration = Duration::from_millis(150);

/// Pause between loading a patch into the edit buffer and playing the
/// audition note, so the note sounds with the new patch.
const AUDITION_LOAD_SETTLE: Duration = Duration::from_millis(50);
const AUDITION_VELOCITY: u8 = 100;

type CmdResult<T> = Result<T, String>;

trait StrErr<T> {
    fn str_err(self) -> CmdResult<T>;
}

impl<T, E: Display> StrErr<T> for Result<T, E> {
    fn str_err(self) -> CmdResult<T> {
        self.map_err(|e| e.to_string())
    }
}

#[derive(Default)]
pub struct AppState {
    synth: Mutex<Option<Synth>>,
    library: Mutex<Library>,
    cancel: AtomicBool,
    /// The audition note that is still sounding, if any.
    sounding: Mutex<Option<SoundingNote>>,
    /// Counts audition notes, so a timer only stops the note it started.
    note_generation: AtomicU64,
}

#[derive(Clone, Copy)]
struct SoundingNote {
    generation: u64,
    channel: u8,
    note: u8,
}

/// Stops the audition note, if one is sounding. Called before every use of
/// the synth so a note never hangs on through a read or write. Lock order:
/// `synth`, then `sounding`.
fn stop_note(state: &AppState, synth: &mut Synth) {
    if let Some(note) = lock(&state.sounding).take() {
        let _ = synth.note_off(note.channel, note.note);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Synth,
    File,
}

struct Stored {
    origin: Origin,
    location: Option<Location>,
    program: Program,
}

#[derive(Default)]
struct Library {
    next_id: u32,
    patches: HashMap<u32, Stored>,
}

impl Library {
    /// Stores a program. A synth program replaces any earlier copy of the same slot.
    fn insert(
        &mut self,
        origin: Origin,
        location: Option<Location>,
        program: Program,
    ) -> PatchSummary {
        if origin == Origin::Synth {
            self.patches
                .retain(|_, p| !(p.origin == Origin::Synth && p.location == location));
        }
        let id = self.next_id;
        self.next_id += 1;
        let summary = PatchSummary::new(id, location, &program);
        self.patches.insert(
            id,
            Stored {
                origin,
                location,
                program,
            },
        );
        summary
    }

    fn get(&self, id: u32) -> CmdResult<&Stored> {
        self.patches
            .get(&id)
            .ok_or_else(|| format!("Unknown patch id {id}"))
    }

    fn messages(&self, ids: &[u32]) -> CmdResult<Vec<Message>> {
        ids.iter()
            .map(|&id| {
                let stored = self.get(id)?;
                let program = stored.program.clone();
                Ok(match stored.location {
                    Some(location) => Message::ProgramDump { location, program },
                    None => Message::EditBufferDump(program),
                })
            })
            .collect()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSummary {
    id: u32,
    /// `U1:05` style, or null for an edit buffer dump.
    location: Option<String>,
    name: String,
    layer_b: String,
}

impl PatchSummary {
    fn new(id: u32, location: Option<Location>, program: &Program) -> Self {
        Self {
            id,
            location: location.map(|l| l.to_string()),
            name: program.name(),
            layer_b: program.layer_b_name(),
        }
    }
}

#[derive(Serialize, Clone)]
struct Progress {
    label: String,
    done: usize,
    total: usize,
}

fn emit_progress(app: &AppHandle, label: String, done: usize, total: usize) {
    let _ = app.emit("progress", Progress { label, done, total });
}

/// Runs blocking MIDI/file work off the async runtime.
async fn blocking<T: Send + 'static>(
    app: AppHandle,
    job: impl FnOnce(&AppHandle, &AppState) -> CmdResult<T> + Send + 'static,
) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(move || job(&app, &app.state::<AppState>()))
        .await
        .str_err()?
}

fn with_synth<T>(state: &AppState, job: impl FnOnce(&mut Synth) -> CmdResult<T>) -> CmdResult<T> {
    let mut guard = lock(&state.synth);
    let synth = guard.as_mut().ok_or("Not connected to a Prophet 12")?;
    stop_note(state, synth);
    job(synth)
}

/// Closes the connection, stopping any audition note first.
fn close_synth(state: &AppState) {
    let mut guard = lock(&state.synth);
    if let Some(synth) = guard.as_mut() {
        stop_note(state, synth);
    }
    *guard = None;
}

/// Reads programs one by one, calling `each` as they arrive. Stops early if cancelled.
fn fetch(
    app: &AppHandle,
    state: &AppState,
    synth: &mut Synth,
    locations: &BTreeSet<Location>,
    verb: &str,
    mut each: impl FnMut(Location, Program),
) -> CmdResult<bool> {
    state.cancel.store(false, Ordering::Relaxed);
    for (i, &location) in locations.iter().enumerate() {
        if state.cancel.load(Ordering::Relaxed) {
            return Ok(false);
        }
        let program = synth.fetch_program(location).str_err()?;
        emit_progress(
            app,
            format!("{verb} {location} {}", program.name()),
            i + 1,
            locations.len(),
        );
        each(location, program);
    }
    Ok(true)
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn write_syx(path: &Path, messages: &[Message]) -> CmdResult<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).map_err(|e| format!("Creating {}: {e}", dir.display()))?;
    }
    fs::write(path, syx::encode(messages)).map_err(|e| format!("Writing {}: {e}", path.display()))
}

fn backup_path(app: &AppHandle) -> CmdResult<PathBuf> {
    let base = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .str_err()?;
    Ok(base.join("Prophet 12 Backups"))
}

#[derive(Serialize)]
pub struct Ports {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

#[tauri::command]
pub fn list_ports() -> CmdResult<Ports> {
    let ports = p12_midi::list_ports().str_err()?;
    Ok(Ports {
        inputs: ports.inputs,
        outputs: ports.outputs,
    })
}

/// Connects to the given port indices and checks that a Prophet 12 answers.
#[tauri::command]
pub async fn connect(app: AppHandle, input: usize, output: usize) -> CmdResult<u8> {
    blocking(app, move |_, state| {
        close_synth(state); // close any previous connection first
        let mut synth = Synth::connect(&input.to_string(), &output.to_string()).str_err()?;
        let version = synth
            .identify()
            .map_err(|e| format!("The ports opened, but no Prophet 12 answered. {e}"))?;
        *lock(&state.synth) = Some(synth);
        Ok(version)
    })
    .await
}

#[tauri::command]
pub fn disconnect(state: tauri::State<'_, AppState>) {
    close_synth(&state);
}

#[tauri::command]
pub fn cancel(state: tauri::State<'_, AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

#[derive(Serialize)]
pub struct ReadResult {
    patches: Vec<PatchSummary>,
    complete: bool,
}

/// Reads a selection such as `U1` or `user`. If cancelled, returns what was read so far.
#[tauri::command]
pub async fn read_programs(app: AppHandle, selection: String) -> CmdResult<ReadResult> {
    blocking(app, move |app, state| {
        let locations = parse_selection(&selection).str_err()?;
        with_synth(state, |synth| {
            let mut patches = Vec::new();
            let complete = fetch(
                app,
                state,
                synth,
                &locations,
                "Reading",
                |location, program| {
                    patches.push(lock(&state.library).insert(
                        Origin::Synth,
                        Some(location),
                        program,
                    ));
                },
            )?;
            Ok(ReadResult { patches, complete })
        })
    })
    .await
}

#[derive(Serialize)]
pub struct OpenedFile {
    patches: Vec<PatchSummary>,
    /// SysEx messages in the file that aren't Prophet 12 programs.
    skipped: usize,
}

/// Loads a .syx file, replacing the previously opened one.
#[tauri::command]
pub fn open_syx(state: tauri::State<'_, AppState>, path: PathBuf) -> CmdResult<OpenedFile> {
    let bytes = fs::read(&path).map_err(|e| format!("Reading {}: {e}", path.display()))?;
    let mut library = lock(&state.library);
    library.patches.retain(|_, p| p.origin != Origin::File);
    let mut patches = Vec::new();
    let mut skipped = 0;
    for entry in syx::parse(&bytes) {
        match entry {
            Entry::Message(Message::ProgramDump { location, program }) => {
                patches.push(library.insert(Origin::File, Some(location), program))
            }
            Entry::Message(Message::EditBufferDump(program)) => {
                patches.push(library.insert(Origin::File, None, program))
            }
            _ => skipped += 1,
        }
    }
    if patches.is_empty() {
        return Err(format!(
            "No Prophet 12 programs found in {}",
            path.display()
        ));
    }
    Ok(OpenedFile { patches, skipped })
}

/// Saves patches into one .syx file.
#[tauri::command]
pub fn save_syx(state: tauri::State<'_, AppState>, ids: Vec<u32>, path: PathBuf) -> CmdResult<()> {
    let messages = lock(&state.library).messages(&ids)?;
    write_syx(&path, &messages)
}

/// Saves each patch as its own .syx file in `dir`.
#[tauri::command]
pub fn save_syx_split(
    state: tauri::State<'_, AppState>,
    ids: Vec<u32>,
    dir: PathBuf,
) -> CmdResult<()> {
    for message in lock(&state.library).messages(&ids)? {
        let location = match &message {
            Message::ProgramDump { location, .. } => Some(*location),
            _ => None,
        };
        let program = message.program().expect("library messages are dumps");
        write_syx(
            &dir.join(syx::file_name(location, program)),
            std::slice::from_ref(&message),
        )?;
    }
    Ok(())
}

/// Loads a patch into the edit buffer, nothing is stored on the synth, and
/// plays a note with it if the audition settings ask for one.
#[tauri::command]
pub async fn audition(app: AppHandle, id: u32) -> CmdResult<()> {
    let settings = app
        .state::<SettingsState>()
        .0
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .audition;
    blocking(app, move |app, state| {
        let program = lock(&state.library).get(id)?.program.clone();
        let started = with_synth(state, |synth| {
            synth.send_edit_buffer(&program).str_err()?;
            if !settings.play_note {
                return Ok(None);
            }
            let channel = match settings.channel {
                0 => synth_channel(synth),
                channel => channel,
            };
            thread::sleep(AUDITION_LOAD_SETTLE);
            synth
                .note_on(channel, settings.note, AUDITION_VELOCITY)
                .str_err()?;
            let note = SoundingNote {
                generation: state.note_generation.fetch_add(1, Ordering::SeqCst) + 1,
                channel,
                note: settings.note,
            };
            *lock(&state.sounding) = Some(note);
            Ok(Some(note.generation))
        })?;
        if let Some(generation) = started {
            let app = app.clone();
            let length = Duration::from_millis(settings.length_ms.into());
            thread::spawn(move || {
                thread::sleep(length);
                let state = app.state::<AppState>();
                let mut guard = lock(&state.synth);
                let mut sounding = lock(&state.sounding);
                // A newer audition, or any other use of the synth, has already stopped it.
                if sounding.is_some_and(|note| note.generation == generation)
                    && let (Some(note), Some(synth)) = (sounding.take(), guard.as_mut())
                {
                    let _ = synth.note_off(note.channel, note.note);
                }
            });
        }
        Ok(())
    })
    .await
}

/// The channel the synth receives on, from its Global parameters: channel 1
/// when it is set to All or can't be asked.
fn synth_channel(synth: &mut Synth) -> u8 {
    synth
        .fetch_globals()
        .ok()
        .and_then(|globals| global_parameter(&globals, GLOBAL_MIDI_CHANNEL))
        .filter(|channel| (1..=16).contains(channel))
        .unwrap_or(1)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteReport {
    backup_path: String,
    /// The written slots as read back from the synth.
    written: Vec<PatchSummary>,
}

/// The one way programs get stored on the synth: reads the current contents
/// of `targets` and saves them as a backup, writes `make(slot, current)` to
/// each slot, then reads every slot back and checks it matches.
fn write_and_verify(
    app: &AppHandle,
    state: &AppState,
    synth: &mut Synth,
    targets: &BTreeSet<Location>,
    (verb, backup_prefix): (&str, &str),
    make: impl Fn(Location, &Program) -> CmdResult<Program>,
) -> CmdResult<WriteReport> {
    let mut current = Vec::new();
    let complete = fetch(
        app,
        state,
        synth,
        targets,
        "Backing up",
        |location, program| current.push((location, program)),
    )?;
    if !complete {
        return Err("Cancelled before anything was written".into());
    }
    let backup = backup_path(app)?.join(format!("{backup_prefix}-{}.syx", timestamp()));
    let previous: Vec<_> = current
        .iter()
        .map(|(location, program)| Message::ProgramDump {
            location: *location,
            program: program.clone(),
        })
        .collect();
    write_syx(&backup, &previous)?;

    let plan = current
        .iter()
        .map(|(location, program)| Ok((*location, make(*location, program)?)))
        .collect::<CmdResult<Vec<_>>>()?;
    for (i, (target, program)) in plan.iter().enumerate() {
        synth
            .write_program(*target, program, WRITE_SETTLE)
            .str_err()?;
        emit_progress(
            app,
            format!("{verb} {target} {}", program.name()),
            i + 1,
            plan.len(),
        );
    }

    let mut written = Vec::new();
    let mut mismatches = Vec::new();
    fetch(
        app,
        state,
        synth,
        targets,
        "Verifying",
        |location, program| {
            if plan.iter().any(|(t, p)| *t == location && *p != program) {
                mismatches.push(location.to_string());
            }
            written.push(lock(&state.library).insert(Origin::Synth, Some(location), program));
        },
    )?;
    if !mismatches.is_empty() {
        return Err(format!(
            "Verification failed for {}. The previous contents are saved in {}",
            mismatches.join(", "),
            backup.display()
        ));
    }
    Ok(WriteReport {
        backup_path: backup.display().to_string(),
        written,
    })
}

fn read_only_error(location: Location) -> String {
    format!("{location} is in a Factory bank, which is read-only")
}

#[derive(Deserialize)]
pub struct WriteRequest {
    id: u32,
    target: String,
}

/// Writes patches to User slots, with backup and verification.
#[tauri::command]
pub async fn write_programs(app: AppHandle, writes: Vec<WriteRequest>) -> CmdResult<WriteReport> {
    blocking(app, move |app, state| {
        let mut plan = BTreeMap::new();
        {
            let library = lock(&state.library);
            for w in &writes {
                let target: Location = w.target.parse().str_err()?;
                if !target.is_user() {
                    return Err(read_only_error(target));
                }
                plan.insert(target, library.get(w.id)?.program.clone());
            }
        }
        if plan.len() != writes.len() {
            return Err("Several patches would be written to the same slot".into());
        }
        let targets: BTreeSet<Location> = plan.keys().copied().collect();
        with_synth(state, |synth| {
            write_and_verify(
                app,
                state,
                synth,
                &targets,
                ("Writing", "before-write"),
                |location, _| Ok(plan[&location].clone()),
            )
        })
    })
    .await
}

/// Suggested tidy versions of the given names (see `p12_core::name::cleanify`).
#[tauri::command]
pub fn cleanify_names(names: Vec<String>) -> Vec<String> {
    names.iter().map(|n| name::cleanify(n)).collect()
}

#[derive(Deserialize)]
pub struct RenameRequest {
    id: u32,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameReport {
    /// Set when patches on the synth were renamed.
    backup_path: Option<String>,
    patches: Vec<PatchSummary>,
}

/// Renames patches. Patches read from the synth are renamed on the synth:
/// each slot is read again and only the name in that fresh copy changes, so
/// edits made on the synth since it was read aren't lost. Patches from a
/// file are renamed in the app only.
#[tauri::command]
pub async fn rename_programs(
    app: AppHandle,
    renames: Vec<RenameRequest>,
) -> CmdResult<RenameReport> {
    blocking(app, move |app, state| {
        let mut on_synth = BTreeMap::new();
        let mut in_app = Vec::new();
        {
            let library = lock(&state.library);
            for r in &renames {
                name::validate(&r.name).str_err()?;
                let stored = library.get(r.id)?;
                match (stored.origin, stored.location) {
                    (Origin::Synth, Some(location)) if !location.is_user() => {
                        return Err(format!(
                            "{}. Copy the patch to a User slot to rename it.",
                            read_only_error(location)
                        ));
                    }
                    (Origin::Synth, Some(location)) => {
                        on_synth.insert(location, r.name.clone());
                    }
                    _ => in_app.push(r),
                }
            }
        }

        let mut patches = Vec::new();
        {
            let mut library = lock(&state.library);
            for r in in_app {
                let stored = library.patches.get_mut(&r.id).expect("checked above");
                stored.program.set_name(&r.name).str_err()?;
                patches.push(PatchSummary::new(r.id, stored.location, &stored.program));
            }
        }

        let mut backup_path = None;
        if !on_synth.is_empty() {
            let targets: BTreeSet<Location> = on_synth.keys().copied().collect();
            let report = with_synth(state, |synth| {
                write_and_verify(
                    app,
                    state,
                    synth,
                    &targets,
                    ("Renaming", "before-rename"),
                    |location, current| {
                        let mut program = current.clone();
                        program.set_name(&on_synth[&location]).str_err()?;
                        Ok(program)
                    },
                )
            })?;
            backup_path = Some(report.backup_path);
            patches.extend(report.written);
        }
        Ok(RenameReport {
            backup_path,
            patches,
        })
    })
    .await
}

/// Copies a patch's Layer A (sound and name) into Layer B of a User slot.
/// The slot is read again first, so its Layer A is kept exactly as it is on
/// the synth; then the usual backup, write and verify follow.
#[tauri::command]
pub async fn copy_to_layer_b(app: AppHandle, id: u32, target: String) -> CmdResult<WriteReport> {
    blocking(app, move |app, state| {
        let source = lock(&state.library).get(id)?.program.clone();
        let target: Location = target.parse().str_err()?;
        if !target.is_user() {
            return Err(read_only_error(target));
        }
        with_synth(state, |synth| {
            write_and_verify(
                app,
                state,
                synth,
                &BTreeSet::from([target]),
                ("Writing Layer B of", "before-layer-b"),
                |_, current| Ok(current.with_layer_b_from(&source)),
            )
        })
    })
    .await
}

/// Name given to initialized slots.
const INIT_NAME: &str = "Init Patch";

fn init_template_path(app: &AppHandle) -> CmdResult<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()
        .str_err()?
        .join("init-template.syx"))
}

/// The captured Basic Patch, if any.
fn load_init_template(app: &AppHandle) -> CmdResult<Option<Program>> {
    let path = init_template_path(app)?;
    let Ok(bytes) = fs::read(&path) else {
        return Ok(None);
    };
    let program = syx::parse(&bytes)
        .into_iter()
        .find_map(|entry| match entry {
            Entry::Message(message) => message.program().cloned(),
            Entry::Unknown { .. } => None,
        });
    program.map(Some).ok_or_else(|| {
        format!(
            "{} doesn't contain a program; capture the template again",
            path.display()
        )
    })
}

/// Whether an init template has been captured.
#[tauri::command]
pub fn has_init_template(app: AppHandle) -> CmdResult<bool> {
    Ok(load_init_template(&app)?.is_some())
}

/// Reads the edit buffer and keeps it as the init template. The user loads
/// the synth's Basic Patch into the edit buffer first (Global → Basic Patch
/// → Write Now).
#[tauri::command]
pub async fn capture_init_template(app: AppHandle) -> CmdResult<String> {
    blocking(app, move |app, state| {
        let program = with_synth(state, |synth| synth.fetch_edit_buffer().str_err())?;
        write_syx(
            &init_template_path(app)?,
            &[Message::EditBufferDump(program.clone())],
        )?;
        Ok(program.name())
    })
    .await
}

/// Writes the init template, named "Init Patch", to User slots, with backup
/// and verification.
#[tauri::command]
pub async fn init_programs(app: AppHandle, locations: Vec<String>) -> CmdResult<WriteReport> {
    blocking(app, move |app, state| {
        let mut template =
            load_init_template(app)?.ok_or("No init template has been captured yet")?;
        template.set_name(INIT_NAME).str_err()?;
        let mut targets = BTreeSet::new();
        for location in &locations {
            let target: Location = location.parse().str_err()?;
            if !target.is_user() {
                return Err(read_only_error(target));
            }
            targets.insert(target);
        }
        with_synth(state, |synth| {
            write_and_verify(
                app,
                state,
                synth,
                &targets,
                ("Initializing", "before-init"),
                |_, _| Ok(template.clone()),
            )
        })
    })
    .await
}

#[tauri::command]
pub fn backup_dir(app: AppHandle) -> CmdResult<String> {
    Ok(backup_path(&app)?.display().to_string())
}
