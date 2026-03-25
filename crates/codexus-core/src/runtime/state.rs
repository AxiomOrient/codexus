use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::methods::{TURN_CANCELLED, TURN_FAILED, TURN_INTERRUPTED};
use crate::runtime::approvals::PendingServerRequest;
use crate::runtime::events::Envelope;
use crate::runtime::rpc_contract::methods as events;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Starting,
    Handshaking,
    Running { generation: u64 },
    Restarting { generation: u64 },
    ShuttingDown,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub connection: ConnectionState,
    pub threads: HashMap<String, ThreadState>,
    pub pending_server_requests: HashMap<String, PendingServerRequest>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StateProjectionLimits {
    pub max_threads: usize,
    pub max_turns_per_thread: usize,
    pub max_items_per_turn: usize,
    pub max_text_bytes_per_item: usize,
    pub max_stdout_bytes_per_item: usize,
    pub max_stderr_bytes_per_item: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ThreadState {
    pub id: String,
    pub active_turn: Option<String>,
    pub turns: HashMap<String, TurnState>,
    pub last_diff: Option<String>,
    pub plan: Option<Value>,
    pub last_seq: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TurnStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TurnState {
    pub id: String,
    pub status: TurnStatus,
    pub items: HashMap<String, ItemState>,
    pub error: Option<Value>,
    pub last_seq: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ItemState {
    pub id: String,
    pub item_type: String,
    pub started: Option<Value>,
    pub completed: Option<Value>,
    pub text_accum: String,
    pub stdout_accum: String,
    pub stderr_accum: String,
    pub text_truncated: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub last_seq: u64,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Starting,
            threads: HashMap::new(),
            pending_server_requests: HashMap::new(),
        }
    }
}

impl Default for StateProjectionLimits {
    fn default() -> Self {
        Self {
            max_threads: 256,
            max_turns_per_thread: 256,
            max_items_per_turn: 256,
            max_text_bytes_per_item: 256 * 1024,
            max_stdout_bytes_per_item: 256 * 1024,
            max_stderr_bytes_per_item: 256 * 1024,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InventorySnapshot {
    pub source_revision: String,
    pub source_hash: String,
}

impl Default for InventorySnapshot {
    fn default() -> Self {
        Self {
            source_revision: crate::protocol::generated::inventory::SOURCE_REVISION.to_owned(),
            source_hash: crate::protocol::generated::inventory::SOURCE_HASH.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStateSnapshot {
    pub inventory: InventorySnapshot,
    pub runtime: RuntimeStateCore,
    pub pending_server_requests: HashMap<String, PendingServerRequest>,
}

impl Default for RuntimeStateSnapshot {
    fn default() -> Self {
        Self::from_runtime_state(&RuntimeState::default())
    }
}

impl RuntimeStateSnapshot {
    pub fn from_runtime_state(state: &RuntimeState) -> Self {
        Self {
            inventory: InventorySnapshot::default(),
            runtime: RuntimeStateCore {
                connection: state.connection.clone(),
                threads: state.threads.clone(),
            },
            pending_server_requests: state.pending_server_requests.clone(),
        }
    }

    pub fn into_runtime_state(self) -> RuntimeState {
        RuntimeState {
            connection: self.runtime.connection,
            threads: self.runtime.threads,
            // Pending server requests from a previous process are unrecoverable:
            // their RPC IDs are gone and the timeout sweeper won't see them.
            // Clear them on restore to avoid zombie entries.
            pending_server_requests: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum StateStoreError {
    Io(String),
    Serde(String),
}

impl std::fmt::Display for StateStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "{message}"),
            Self::Serde(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for StateStoreError {}

impl From<std::io::Error> for StateStoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for StateStoreError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err.to_string())
    }
}

pub trait StateStore: Send + Sync {
    fn load_snapshot(&self) -> Result<RuntimeStateSnapshot, StateStoreError>;
    fn save_snapshot(&self, snapshot: &RuntimeStateSnapshot) -> Result<(), StateStoreError>;
}

#[derive(Default)]
pub struct MemoryStateStore {
    snapshot: std::sync::RwLock<RuntimeStateSnapshot>,
}

impl MemoryStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StateStore for MemoryStateStore {
    fn load_snapshot(&self) -> Result<RuntimeStateSnapshot, StateStoreError> {
        Ok(match self.snapshot.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        })
    }

    fn save_snapshot(&self, snapshot: &RuntimeStateSnapshot) -> Result<(), StateStoreError> {
        match self.snapshot.write() {
            Ok(mut guard) => {
                *guard = snapshot.clone();
            }
            Err(poisoned) => {
                *poisoned.into_inner() = snapshot.clone();
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonFilePaths {
    pub snapshot_path: PathBuf,
}

impl JsonFilePaths {
    pub fn from_base_dir(base_dir: &Path) -> Self {
        Self {
            snapshot_path: base_dir.join(".codexus").join("state.json"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStateCore {
    connection: ConnectionState,
    threads: HashMap<String, ThreadState>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonStoreReadOp {
    ReadFile(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonStoreWriteOp {
    EnsureDir(PathBuf),
    WriteJsonFile { path: PathBuf, value: Value },
}

#[derive(Clone, Debug, PartialEq)]
pub struct JsonStoreLoadPlan {
    pub operations: Vec<JsonStoreReadOp>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JsonStoreSavePlan {
    pub operations: Vec<JsonStoreWriteOp>,
}

pub struct JsonFileStateStore {
    paths: JsonFilePaths,
}

impl JsonFileStateStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        Self {
            paths: JsonFilePaths::from_base_dir(&base_dir),
        }
    }

    pub fn paths(&self) -> &JsonFilePaths {
        &self.paths
    }

    pub fn plan_load(paths: &JsonFilePaths) -> JsonStoreLoadPlan {
        JsonStoreLoadPlan {
            operations: vec![JsonStoreReadOp::ReadFile(paths.snapshot_path.clone())],
        }
    }

    pub fn plan_save(
        paths: &JsonFilePaths,
        snapshot: &RuntimeStateSnapshot,
    ) -> Result<JsonStoreSavePlan, StateStoreError> {
        let snapshot_json = canonicalize_json_value(serde_json::to_value(snapshot)?);
        let mut operations = Vec::with_capacity(2);
        if let Some(parent) = paths.snapshot_path.parent() {
            operations.push(JsonStoreWriteOp::EnsureDir(parent.to_path_buf()));
        }
        operations.push(JsonStoreWriteOp::WriteJsonFile {
            path: paths.snapshot_path.clone(),
            value: snapshot_json,
        });
        Ok(JsonStoreSavePlan { operations })
    }

    pub fn apply_load(plan: &JsonStoreLoadPlan) -> Result<Vec<Option<String>>, StateStoreError> {
        let mut reads = Vec::with_capacity(plan.operations.len());
        for op in &plan.operations {
            match op {
                JsonStoreReadOp::ReadFile(path) => match fs::read_to_string(path) {
                    Ok(content) => reads.push(Some(content)),
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => reads.push(None),
                    Err(err) => return Err(err.into()),
                },
            }
        }
        Ok(reads)
    }

    pub fn normalize_loaded(
        read_results: &[Option<String>],
    ) -> Result<RuntimeStateSnapshot, StateStoreError> {
        match read_results.first() {
            Some(Some(snapshot)) => Ok(serde_json::from_str(snapshot)?),
            _ => Ok(RuntimeStateSnapshot::default()),
        }
    }

    pub fn apply_save(plan: &JsonStoreSavePlan) -> Result<(), StateStoreError> {
        for op in &plan.operations {
            match op {
                JsonStoreWriteOp::EnsureDir(path) => {
                    fs::create_dir_all(path)?;
                }
                JsonStoreWriteOp::WriteJsonFile { path, value } => {
                    // Write to a sibling temp file then rename so that a crash mid-write
                    // leaves the previous snapshot intact rather than a corrupted file.
                    // rename(2) is atomic on POSIX (same filesystem guaranteed by same dir).
                    let tmp = path.with_extension("tmp");
                    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
                    fs::rename(&tmp, path)?;
                }
            }
        }
        Ok(())
    }
}

fn canonicalize_json_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            // Consume the map into a sorted vec, then recurse — no extra clones.
            let mut pairs: Vec<(String, Value)> = map.into_iter().collect();
            pairs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
            Value::Object(
                pairs
                    .into_iter()
                    .map(|(k, v)| (k, canonicalize_json_value(v)))
                    .collect(),
            )
        }
        Value::Array(items) => {
            Value::Array(items.into_iter().map(canonicalize_json_value).collect())
        }
        other => other,
    }
}

impl StateStore for JsonFileStateStore {
    fn load_snapshot(&self) -> Result<RuntimeStateSnapshot, StateStoreError> {
        let plan = Self::plan_load(&self.paths);
        let reads = Self::apply_load(&plan)?;
        Self::normalize_loaded(&reads)
    }

    fn save_snapshot(&self, snapshot: &RuntimeStateSnapshot) -> Result<(), StateStoreError> {
        let plan = Self::plan_save(&self.paths, snapshot)?;
        Self::apply_save(&plan)
    }
}

/// Pure reducer: consumes old state + envelope and returns next state.
/// Delegates to `reduce_in_place_with_limits` with default retention limits.
pub fn reduce(mut state: RuntimeState, envelope: &Envelope) -> RuntimeState {
    reduce_in_place_with_limits(&mut state, envelope, &StateProjectionLimits::default());
    state
}

/// In-place reducer used by runtime projection.
/// Delegates to `reduce_in_place_with_limits` with default retention limits.
pub fn reduce_in_place(state: &mut RuntimeState, envelope: &Envelope) {
    reduce_in_place_with_limits(state, envelope, &StateProjectionLimits::default());
}

/// In-place reducer with explicit retention bounds for long-running runtimes.
/// Allocation: new map entries + appended deltas; prune candidate vectors are allocated only
/// when a cap is exceeded.
/// Complexity: O(1) average map work per event, plus O(t) touched-thread item-cap checks
/// (t <= max_turns_per_thread after pruning), and O(n log n) only when sorting eviction
/// candidates for thread/turn/item pruning.
pub fn reduce_in_place_with_limits(
    state: &mut RuntimeState,
    envelope: &Envelope,
    limits: &StateProjectionLimits,
) {
    let Some(method) = envelope.method.as_deref() else {
        return;
    };
    let seq = envelope.seq;
    let touched_thread_id = envelope.thread_id.as_deref();
    if is_stale_thread_event(state, touched_thread_id, seq) {
        return;
    }

    match method {
        events::THREAD_STARTED => handle_thread_started(state, envelope, seq),
        events::TURN_STARTED => handle_turn_started(state, envelope, seq),
        events::TURN_COMPLETED => {
            handle_turn_terminal(state, envelope, seq, TurnStatus::Completed, false)
        }
        TURN_FAILED => handle_turn_terminal(state, envelope, seq, TurnStatus::Failed, true),
        TURN_CANCELLED => handle_turn_terminal(state, envelope, seq, TurnStatus::Cancelled, false),
        TURN_INTERRUPTED => {
            handle_turn_terminal(state, envelope, seq, TurnStatus::Interrupted, false)
        }
        events::TURN_DIFF_UPDATED => handle_turn_diff_updated(state, envelope, seq),
        events::TURN_PLAN_UPDATED => handle_turn_plan_updated(state, envelope, seq),
        events::ITEM_STARTED => handle_item_started(state, envelope, seq),
        events::ITEM_AGENT_MESSAGE_DELTA => {
            handle_item_agent_message_delta(state, envelope, seq, limits)
        }
        events::ITEM_COMMAND_EXECUTION_OUTPUT_DELTA => {
            handle_item_command_output_delta(state, envelope, seq, limits)
        }
        events::ITEM_COMPLETED => handle_item_completed(state, envelope, seq),
        _ => {}
    }

    prune_state(state, limits, touched_thread_id);
}

fn is_stale_thread_event(state: &RuntimeState, thread_id: Option<&str>, seq: u64) -> bool {
    let Some(thread_id) = thread_id else {
        return false;
    };
    state
        .threads
        .get(thread_id)
        .is_some_and(|thread| seq < thread.last_seq)
}

fn handle_thread_started(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some(thread_id) = envelope.thread_id.as_deref() else {
        return;
    };
    thread_mut(state, thread_id, seq);
}

fn handle_turn_started(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some((thread_id, turn_id)) = thread_and_turn_ids(envelope) else {
        return;
    };
    let thread = thread_mut(state, thread_id, seq);
    thread.active_turn = Some(turn_id.to_owned());
    let turn = turn_mut(thread, turn_id, seq);
    turn.status = TurnStatus::InProgress;
}

fn handle_turn_terminal(
    state: &mut RuntimeState,
    envelope: &Envelope,
    seq: u64,
    status: TurnStatus,
    with_error: bool,
) {
    let Some((thread_id, turn_id)) = thread_and_turn_ids(envelope) else {
        return;
    };
    let thread = thread_mut(state, thread_id, seq);
    clear_active_turn_if_matching(thread, turn_id);
    let turn = turn_mut(thread, turn_id, seq);
    turn.status = status;
    if with_error {
        turn.error = envelope
            .json
            .get("params")
            .and_then(|p| p.get("error"))
            .cloned();
    }
}

fn clear_active_turn_if_matching(thread: &mut ThreadState, turn_id: &str) {
    if thread.active_turn.as_deref() == Some(turn_id) {
        thread.active_turn = None;
    }
}

fn handle_turn_diff_updated(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some(thread_id) = envelope.thread_id.as_deref() else {
        return;
    };
    let thread = thread_mut(state, thread_id, seq);
    thread.last_diff = envelope
        .json
        .get("params")
        .and_then(|p| p.get("diff"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
}

fn handle_turn_plan_updated(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some(thread_id) = envelope.thread_id.as_deref() else {
        return;
    };
    let thread = thread_mut(state, thread_id, seq);
    thread.plan = envelope
        .json
        .get("params")
        .and_then(|p| p.get("plan"))
        .cloned();
}

fn handle_item_started(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some(item) = item_from_envelope(state, envelope, seq) else {
        return;
    };
    // Early Decomposition: extract params once after the guard so failed lookups pay no clone cost.
    // envelope.json and state are independent borrows so this compiles without conflict.
    let params = envelope.json.get("params");
    item.started = params.cloned();
    item.item_type = params
        .and_then(|p| p.get("itemType"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
}

fn handle_item_agent_message_delta(
    state: &mut RuntimeState,
    envelope: &Envelope,
    seq: u64,
    limits: &StateProjectionLimits,
) {
    // Early Decomposition: extract delta before entering mutable state traversal.
    let delta = envelope
        .json
        .get("params")
        .and_then(|p| p.get("delta"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let Some(item) = item_from_envelope(state, envelope, seq) else {
        return;
    };
    append_capped(
        &mut item.text_accum,
        delta,
        limits.max_text_bytes_per_item,
        &mut item.text_truncated,
    );
}

fn handle_item_command_output_delta(
    state: &mut RuntimeState,
    envelope: &Envelope,
    seq: u64,
    limits: &StateProjectionLimits,
) {
    // Early Decomposition: extract params once, then derive both fields from the same reference.
    let params = envelope.json.get("params");
    let stdout = params
        .and_then(|p| p.get("stdout"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let stderr = params
        .and_then(|p| p.get("stderr"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let Some(item) = item_from_envelope(state, envelope, seq) else {
        return;
    };
    append_capped(
        &mut item.stdout_accum,
        stdout,
        limits.max_stdout_bytes_per_item,
        &mut item.stdout_truncated,
    );
    append_capped(
        &mut item.stderr_accum,
        stderr,
        limits.max_stderr_bytes_per_item,
        &mut item.stderr_truncated,
    );
}

fn handle_item_completed(state: &mut RuntimeState, envelope: &Envelope, seq: u64) {
    let Some(item) = item_from_envelope(state, envelope, seq) else {
        return;
    };
    item.completed = envelope.json.get("params").cloned();
}

fn thread_and_turn_ids(envelope: &Envelope) -> Option<(&str, &str)> {
    let (Some(thread_id), Some(turn_id)) =
        (envelope.thread_id.as_deref(), envelope.turn_id.as_deref())
    else {
        return None;
    };
    Some((thread_id, turn_id))
}

fn thread_turn_item_ids(envelope: &Envelope) -> Option<(&str, &str, &str)> {
    let (thread_id, turn_id) = thread_and_turn_ids(envelope)?;
    let item_id = envelope.item_id.as_deref()?;
    Some((thread_id, turn_id, item_id))
}

fn item_from_envelope<'a>(
    state: &'a mut RuntimeState,
    envelope: &Envelope,
    seq: u64,
) -> Option<&'a mut ItemState> {
    let (thread_id, turn_id, item_id) = thread_turn_item_ids(envelope)?;
    let thread = thread_mut(state, thread_id, seq);
    let turn = turn_mut(thread, turn_id, seq);
    Some(item_mut(turn, item_id, seq))
}

/// Upsert a thread entry and advance its `last_seq`.
/// Each `*_mut` function is responsible only for its own level; callers chain them.
/// State Transparency: only this function writes `thread.last_seq`.
///
/// Hot-path optimisation: avoids allocating the entry key String on the common "already exists"
/// path by using `contains_key` (borrows `&str` via `Borrow<str>`) before calling `insert`.
/// `HashMap::entry(key)` requires an owned `K` before the lookup, so it would always allocate;
/// the explicit two-step avoids that allocation for existing threads.
fn thread_mut<'a>(state: &'a mut RuntimeState, thread_id: &str, seq: u64) -> &'a mut ThreadState {
    if !state.threads.contains_key(thread_id) {
        state.threads.insert(
            thread_id.to_owned(),
            ThreadState {
                id: thread_id.to_owned(),
                active_turn: None,
                turns: HashMap::new(),
                last_diff: None,
                plan: None,
                last_seq: seq,
            },
        );
    }
    let thread = state.threads.get_mut(thread_id).expect("just inserted");
    if seq > thread.last_seq {
        thread.last_seq = seq;
    }
    thread
}

/// Upsert a turn entry and advance its `last_seq`.
/// State Transparency: only this function writes `turn.last_seq`; does NOT touch thread.last_seq.
/// Callers must call `thread_mut` first to update the thread level.
fn turn_mut<'a>(thread: &'a mut ThreadState, turn_id: &str, seq: u64) -> &'a mut TurnState {
    if !thread.turns.contains_key(turn_id) {
        thread.turns.insert(
            turn_id.to_owned(),
            TurnState {
                id: turn_id.to_owned(),
                status: TurnStatus::InProgress,
                items: HashMap::new(),
                error: None,
                last_seq: seq,
            },
        );
    }
    let turn = thread.turns.get_mut(turn_id).expect("just inserted");
    if seq > turn.last_seq {
        turn.last_seq = seq;
    }
    turn
}

/// Upsert an item entry and advance its `last_seq`.
/// State Transparency: only this function writes `item.last_seq`; does NOT touch turn.last_seq.
/// Callers must call `turn_mut` first to update the turn level.
fn item_mut<'a>(turn: &'a mut TurnState, item_id: &str, seq: u64) -> &'a mut ItemState {
    if !turn.items.contains_key(item_id) {
        turn.items.insert(
            item_id.to_owned(),
            ItemState {
                id: item_id.to_owned(),
                item_type: "unknown".to_owned(),
                started: None,
                completed: None,
                text_accum: String::new(),
                stdout_accum: String::new(),
                stderr_accum: String::new(),
                text_truncated: false,
                stdout_truncated: false,
                stderr_truncated: false,
                last_seq: seq,
            },
        );
    }
    let item = turn.items.get_mut(item_id).expect("just inserted");
    if seq > item.last_seq {
        item.last_seq = seq;
    }
    item
}

fn append_capped(out: &mut String, delta: &str, max_bytes: usize, truncated: &mut bool) {
    if delta.is_empty() {
        return;
    }
    if out.len() >= max_bytes {
        *truncated = true;
        return;
    }
    let remain = max_bytes - out.len();
    if delta.len() <= remain {
        out.push_str(delta);
        return;
    }
    let mut cut = remain;
    while cut > 0 && !delta.is_char_boundary(cut) {
        cut -= 1;
    }
    if cut > 0 {
        out.push_str(&delta[..cut]);
    }
    *truncated = true;
}

fn prune_state(
    state: &mut RuntimeState,
    limits: &StateProjectionLimits,
    touched_thread_id: Option<&str>,
) {
    if state.threads.len() > limits.max_threads {
        let remove_count = state.threads.len() - limits.max_threads;
        let mut by_age: Vec<(String, u64)> = state
            .threads
            .iter()
            .map(|(id, thread)| (id.clone(), thread.last_seq))
            .collect();
        if remove_count > 0 {
            by_age.select_nth_unstable_by_key(remove_count - 1, |(_, seq)| *seq);
        }
        for (id, _) in by_age.into_iter().take(remove_count) {
            state.threads.remove(&id);
        }
    }

    let Some(thread_id) = touched_thread_id else {
        return;
    };
    let Some(thread) = state.threads.get_mut(thread_id) else {
        return;
    };

    prune_turns(thread, limits.max_turns_per_thread);
    for turn in thread.turns.values_mut() {
        prune_items(turn, limits.max_items_per_turn);
    }
}

fn prune_turns(thread: &mut ThreadState, max_turns: usize) {
    if thread.turns.len() <= max_turns {
        return;
    }

    let active = thread.active_turn.as_deref();
    let mut candidates: Vec<(String, u64)> = thread
        .turns
        .iter()
        .filter(|(id, _)| Some(id.as_str()) != active)
        .map(|(id, turn)| (id.clone(), turn.last_seq))
        .collect();

    let removable = thread.turns.len().saturating_sub(max_turns);
    if removable > 0 && !candidates.is_empty() {
        let partition_idx = std::cmp::min(removable - 1, candidates.len() - 1);
        candidates.select_nth_unstable_by_key(partition_idx, |(_, seq)| *seq);
    }

    for (id, _) in candidates.into_iter().take(removable) {
        thread.turns.remove(&id);
    }
}

fn prune_items(turn: &mut TurnState, max_items: usize) {
    if turn.items.len() <= max_items {
        return;
    }

    let remove_count = turn.items.len() - max_items;
    let mut by_age: Vec<(String, u64)> = turn
        .items
        .iter()
        .map(|(id, item)| (id.clone(), item.last_seq))
        .collect();
    if remove_count > 0 {
        let partition_idx = std::cmp::min(remove_count - 1, by_age.len() - 1);
        by_age.select_nth_unstable_by_key(partition_idx, |(_, seq)| *seq);
    }
    for (id, _) in by_age.into_iter().take(remove_count) {
        turn.items.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Arc;

    use crate::runtime::events::{Direction, Envelope, MsgKind};

    use super::*;

    fn envelope_with_seq(
        seq: u64,
        method: &str,
        thread: &str,
        turn: &str,
        item: Option<&str>,
        params: Value,
    ) -> Envelope {
        Envelope {
            seq,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from(method)),
            thread_id: Some(Arc::from(thread)),
            turn_id: Some(Arc::from(turn)),
            item_id: item.map(Arc::from),
            json: Arc::new(json!({"method": method, "params": params})),
        }
    }

    fn envelope(
        method: &str,
        thread: &str,
        turn: &str,
        item: Option<&str>,
        params: Value,
    ) -> Envelope {
        envelope_with_seq(1, method, thread, turn, item, params)
    }

    #[test]
    fn reduce_turn_lifecycle() {
        let state = RuntimeState::default();

        let state = reduce(
            state,
            &envelope("turn/started", "thr", "turn", None, json!({})),
        );
        assert_eq!(state.threads["thr"].active_turn.as_deref(), Some("turn"));
        assert_eq!(
            state.threads["thr"].turns["turn"].status,
            TurnStatus::InProgress
        );

        let state = reduce(
            state,
            &envelope("turn/completed", "thr", "turn", None, json!({})),
        );
        assert_eq!(state.threads["thr"].active_turn, None);
        assert_eq!(
            state.threads["thr"].turns["turn"].status,
            TurnStatus::Completed
        );
    }

    #[test]
    fn reduce_turn_cancelled_marks_cancelled_and_clears_active_turn() {
        let state = RuntimeState::default();

        let state = reduce(
            state,
            &envelope("turn/started", "thr", "turn", None, json!({})),
        );
        assert_eq!(state.threads["thr"].active_turn.as_deref(), Some("turn"));

        let state = reduce(
            state,
            &envelope("turn/cancelled", "thr", "turn", None, json!({})),
        );
        assert_eq!(state.threads["thr"].active_turn, None);
        assert_eq!(
            state.threads["thr"].turns["turn"].status,
            TurnStatus::Cancelled
        );
    }

    #[test]
    fn reduce_delta_and_output() {
        let state = RuntimeState::default();
        let state = reduce(
            state,
            &envelope("turn/started", "thr", "turn", None, json!({})),
        );
        let state = reduce(
            state,
            &envelope(
                "item/started",
                "thr",
                "turn",
                Some("item"),
                json!({"itemType":"agentMessage"}),
            ),
        );
        let state = reduce(
            state,
            &envelope(
                "item/agentMessage/delta",
                "thr",
                "turn",
                Some("item"),
                json!({"delta":"hello"}),
            ),
        );

        let state = reduce(
            state,
            &envelope(
                "item/commandExecution/outputDelta",
                "thr",
                "turn",
                Some("item"),
                json!({"stdout":"out","stderr":"err"}),
            ),
        );

        let item = &state.threads["thr"].turns["turn"].items["item"];
        assert_eq!(item.text_accum, "hello");
        assert_eq!(item.stdout_accum, "out");
        assert_eq!(item.stderr_accum, "err");
    }

    #[test]
    fn reduce_applies_text_caps_and_marks_truncated() {
        let mut state = RuntimeState::default();
        let limits = StateProjectionLimits {
            max_threads: 8,
            max_turns_per_thread: 8,
            max_items_per_turn: 8,
            max_text_bytes_per_item: 4,
            max_stdout_bytes_per_item: 3,
            max_stderr_bytes_per_item: 2,
        };

        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(
                1,
                "item/started",
                "thr",
                "turn",
                Some("item"),
                json!({"itemType":"agentMessage"}),
            ),
            &limits,
        );
        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(
                2,
                "item/agentMessage/delta",
                "thr",
                "turn",
                Some("item"),
                json!({"delta":"hello"}),
            ),
            &limits,
        );
        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(
                3,
                "item/commandExecution/outputDelta",
                "thr",
                "turn",
                Some("item"),
                json!({"stdout":"abcd","stderr":"xyz"}),
            ),
            &limits,
        );

        let item = &state.threads["thr"].turns["turn"].items["item"];
        assert_eq!(item.text_accum, "hell");
        assert!(item.text_truncated);
        assert_eq!(item.stdout_accum, "abc");
        assert!(item.stdout_truncated);
        assert_eq!(item.stderr_accum, "xy");
        assert!(item.stderr_truncated);
    }

    #[test]
    fn reduce_prunes_old_threads_turns_and_items() {
        let mut state = RuntimeState::default();
        let limits = StateProjectionLimits {
            max_threads: 2,
            max_turns_per_thread: 2,
            max_items_per_turn: 2,
            max_text_bytes_per_item: 1024,
            max_stdout_bytes_per_item: 1024,
            max_stderr_bytes_per_item: 1024,
        };

        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(1, "thread/started", "thr_1", "turn_a", None, json!({})),
            &limits,
        );
        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(2, "thread/started", "thr_2", "turn_a", None, json!({})),
            &limits,
        );
        reduce_in_place_with_limits(
            &mut state,
            &envelope_with_seq(3, "thread/started", "thr_3", "turn_a", None, json!({})),
            &limits,
        );
        assert!(!state.threads.contains_key("thr_1"));
        assert!(state.threads.contains_key("thr_2"));
        assert!(state.threads.contains_key("thr_3"));

        for seq in 10..=12 {
            let turn = format!("turn_{seq}");
            reduce_in_place_with_limits(
                &mut state,
                &envelope_with_seq(
                    seq,
                    "turn/started",
                    "thr_3",
                    &turn,
                    None,
                    json!({ "threadId":"thr_3", "turnId": turn }),
                ),
                &limits,
            );
        }
        let thr = state.threads.get("thr_3").expect("thread");
        assert!(thr.turns.len() <= 2);

        let turn_id = thr.active_turn.clone().expect("active turn");
        for seq in 20..=22 {
            let item = format!("item_{seq}");
            reduce_in_place_with_limits(
                &mut state,
                &envelope_with_seq(
                    seq,
                    "item/started",
                    "thr_3",
                    &turn_id,
                    Some(&item),
                    json!({"itemType":"agentMessage"}),
                ),
                &limits,
            );
        }

        let thr = state.threads.get("thr_3").expect("thread");
        let turn = thr.turns.get(&turn_id).expect("turn");
        assert!(turn.items.len() <= 2);
    }

    #[test]
    fn reduce_drops_stale_turn_event_by_sequence() {
        let mut state = RuntimeState::default();

        reduce_in_place(
            &mut state,
            &envelope_with_seq(10, "turn/started", "thr", "turn", None, json!({})),
        );
        reduce_in_place(
            &mut state,
            &envelope_with_seq(11, "turn/completed", "thr", "turn", None, json!({})),
        );
        reduce_in_place(
            &mut state,
            &envelope_with_seq(
                9,
                "turn/failed",
                "thr",
                "turn",
                None,
                json!({"error":{"message":"stale"}}),
            ),
        );

        let turn = &state.threads["thr"].turns["turn"];
        assert_eq!(turn.status, TurnStatus::Completed);
        assert_eq!(turn.error, None);
        assert_eq!(turn.last_seq, 11);
        assert_eq!(state.threads["thr"].last_seq, 11);
    }

    #[test]
    fn reduce_drops_stale_item_delta_by_sequence() {
        let mut state = RuntimeState::default();

        reduce_in_place(
            &mut state,
            &envelope_with_seq(
                1,
                "item/started",
                "thr",
                "turn",
                Some("item"),
                json!({"itemType":"agentMessage"}),
            ),
        );
        reduce_in_place(
            &mut state,
            &envelope_with_seq(
                3,
                "item/agentMessage/delta",
                "thr",
                "turn",
                Some("item"),
                json!({"delta":"new"}),
            ),
        );
        reduce_in_place(
            &mut state,
            &envelope_with_seq(
                2,
                "item/agentMessage/delta",
                "thr",
                "turn",
                Some("item"),
                json!({"delta":"old"}),
            ),
        );

        let item = &state.threads["thr"].turns["turn"].items["item"];
        assert_eq!(item.text_accum, "new");
        assert_eq!(item.last_seq, 3);
        assert_eq!(state.threads["thr"].last_seq, 3);
    }

    #[test]
    fn memory_state_store_round_trip() {
        let store = MemoryStateStore::new();
        let mut state = RuntimeState {
            connection: ConnectionState::Running { generation: 7 },
            ..RuntimeState::default()
        };
        state.pending_server_requests.insert(
            "p_1".to_owned(),
            PendingServerRequest {
                approval_id: "p_1".to_owned(),
                deadline_unix_ms: 1_700_000_000_000,
                method: "item/fileChange/requestApproval".to_owned(),
                params: json!({"threadId":"thr_1"}),
            },
        );

        let snapshot = RuntimeStateSnapshot::from_runtime_state(&state);
        store.save_snapshot(&snapshot).expect("save");
        let loaded = store.load_snapshot().expect("load");
        assert_eq!(loaded, snapshot);
    }

    #[test]
    fn json_file_state_store_round_trip() {
        let base = std::env::temp_dir()
            .join("codexus-state-store-tests")
            .join(format!("pid-{}", std::process::id()))
            .join("json-round-trip");
        let _ = std::fs::remove_dir_all(&base);

        let store = JsonFileStateStore::new(base.clone());
        let mut state = RuntimeState {
            connection: ConnectionState::Running { generation: 3 },
            ..RuntimeState::default()
        };
        state.threads.insert(
            "thr_1".to_owned(),
            ThreadState {
                id: "thr_1".to_owned(),
                active_turn: Some("turn_1".to_owned()),
                turns: HashMap::new(),
                last_diff: Some("diff".to_owned()),
                plan: Some(json!({"steps":[] })),
                last_seq: 1,
            },
        );
        state.pending_server_requests.insert(
            "req_1".to_owned(),
            PendingServerRequest {
                approval_id: "req_1".to_owned(),
                deadline_unix_ms: 1_700_000_000_000,
                method: "item/tool/call".to_owned(),
                params: json!({"name":"ls"}),
            },
        );

        let snapshot = RuntimeStateSnapshot::from_runtime_state(&state);
        let save_plan = JsonFileStateStore::plan_save(store.paths(), &snapshot).expect("plan save");
        JsonFileStateStore::apply_save(&save_plan).expect("apply save");
        let load_plan = JsonFileStateStore::plan_load(store.paths());
        let reads = JsonFileStateStore::apply_load(&load_plan).expect("apply load");
        let loaded = JsonFileStateStore::normalize_loaded(&reads).expect("normalize load");

        assert_eq!(loaded, snapshot);
        assert!(store.paths().snapshot_path.is_file());

        let _ = std::fs::remove_dir_all(base);
    }
}
