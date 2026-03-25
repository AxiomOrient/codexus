use std::sync::Arc;

use crate::runtime::approvals::PendingServerRequest;
use crate::runtime::events::Envelope;
use crate::runtime::state::ConnectionState;
use crate::runtime::state::{reduce_in_place_with_limits, RuntimeState, RuntimeStateSnapshot};

use super::RuntimeInner;

pub(super) fn state_snapshot_arc(inner: &Arc<RuntimeInner>) -> Arc<RuntimeState> {
    match inner.snapshots.state.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Apply a mutation atomically: acquire the write lock, mutate in-place via `Arc::make_mut`,
/// and return a cheap Arc clone of the updated state.
///
/// `Arc::make_mut` avoids cloning the full `RuntimeState` when no snapshot holders exist
/// (Arc strong count == 1), which is the common case during streaming. It falls back to a
/// clone only when an external caller is actively holding a snapshot Arc.
///
/// Holding the write lock across the entire read-modify-write prevents lost-update races.
fn apply_state_mutation<F: FnOnce(&mut RuntimeState)>(
    inner: &Arc<RuntimeInner>,
    update: F,
) -> Arc<RuntimeState> {
    let mut guard = inner
        .snapshots
        .state
        .write()
        .unwrap_or_else(|p| p.into_inner());
    update(Arc::make_mut(&mut *guard));
    Arc::clone(&*guard)
}

/// Offload snapshot persistence to the blocking thread pool so the async dispatch loop is
/// never stalled by file I/O. Fire-and-forget: the latest persisted state on disk may lag
/// by one scheduling quantum, which is acceptable for crash-recovery semantics.
fn persist_state(inner: &Arc<RuntimeInner>, next: Arc<RuntimeState>) {
    let store = Arc::clone(&inner.spec.state_store);
    tokio::task::spawn_blocking(move || {
        let _ = store.save_snapshot(&RuntimeStateSnapshot::from_runtime_state(&next));
    });
}

pub(super) fn state_set_connection(inner: &Arc<RuntimeInner>, connection: ConnectionState) {
    let next = apply_state_mutation(inner, |state| {
        state.connection = connection;
    });
    persist_state(inner, next);
}

/// Hot path: called on every inbound envelope. Uses `Arc::make_mut` directly to avoid the
/// unnecessary `Arc::clone` return value that `apply_state_mutation` would produce.
/// No persistence — see comment in the previous implementation for rationale.
pub(super) fn state_apply_envelope(inner: &Arc<RuntimeInner>, envelope: &Envelope) {
    let limits = &inner.spec.state_projection_limits;
    let mut guard = inner
        .snapshots
        .state
        .write()
        .unwrap_or_else(|p| p.into_inner());
    reduce_in_place_with_limits(Arc::make_mut(&mut *guard), envelope, limits);
}

pub(super) fn state_insert_pending_server_request(
    inner: &Arc<RuntimeInner>,
    rpc_id: &str,
    request: PendingServerRequest,
) {
    let next = apply_state_mutation(inner, |state| {
        state
            .pending_server_requests
            .insert(rpc_id.to_owned(), request);
    });
    persist_state(inner, next);
}

pub(super) fn state_remove_pending_server_request(inner: &Arc<RuntimeInner>, rpc_id: &str) {
    let next = apply_state_mutation(inner, |state| {
        state.pending_server_requests.remove(rpc_id);
    });
    persist_state(inner, next);
}

pub(super) fn state_clear_pending_server_requests(inner: &Arc<RuntimeInner>) {
    let next = apply_state_mutation(inner, |state| {
        state.pending_server_requests.clear();
    });
    persist_state(inner, next);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::approvals::PendingServerRequest;
    use crate::runtime::events::{Direction, Envelope, MsgKind};
    use crate::runtime::state::StateProjectionLimits;
    use serde_json::json;

    fn test_envelope(method: &str) -> Envelope {
        Envelope {
            seq: 1,
            ts_millis: 0,
            direction: Direction::Inbound,
            kind: MsgKind::Notification,
            rpc_id: None,
            method: Some(Arc::from(method)),
            thread_id: Some(Arc::from("thr_1")),
            turn_id: None,
            item_id: None,
            json: Arc::new(json!({"params": {"threadId": "thr_1"}})),
        }
    }

    // Pure state-transition helpers used in tests below.
    // These mirror what apply_state_mutation closures do, kept here for focused unit testing.

    fn apply_connection(state: &RuntimeState, connection: ConnectionState) -> RuntimeState {
        let mut next = state.clone();
        next.connection = connection;
        next
    }

    fn apply_pending_insert(
        state: &RuntimeState,
        rpc_id: &str,
        request: PendingServerRequest,
    ) -> RuntimeState {
        let mut next = state.clone();
        next.pending_server_requests
            .insert(rpc_id.to_owned(), request);
        next
    }

    fn apply_envelope(
        state: &RuntimeState,
        envelope: &Envelope,
        limits: &StateProjectionLimits,
    ) -> RuntimeState {
        let mut next = state.clone();
        reduce_in_place_with_limits(&mut next, envelope, limits);
        next
    }

    #[test]
    fn connection_transition_does_not_mutate_input() {
        let current = RuntimeState::default();
        let next = apply_connection(&current, ConnectionState::Running { generation: 7 });
        assert_eq!(current.connection, ConnectionState::Starting);
        assert_eq!(next.connection, ConnectionState::Running { generation: 7 });
    }

    #[test]
    fn pending_request_insert_does_not_mutate_input() {
        let current = RuntimeState::default();
        let next = apply_pending_insert(
            &current,
            "approval_1",
            PendingServerRequest {
                approval_id: "approval_1".to_owned(),
                deadline_unix_ms: 42,
                method: "item/fileChange/requestApproval".to_owned(),
                params: json!({"threadId":"thr_1"}),
            },
        );
        assert!(current.pending_server_requests.is_empty());
        assert!(next.pending_server_requests.contains_key("approval_1"));
    }

    #[test]
    fn envelope_apply_projects_next_state() {
        let current = RuntimeState::default();
        let next = apply_envelope(
            &current,
            &test_envelope("thread/started"),
            &StateProjectionLimits::default(),
        );
        assert!(current.threads.is_empty());
        assert!(next.threads.contains_key("thr_1"));
    }
}
