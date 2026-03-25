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

/// Apply a mutation atomically: read current state, apply closure, write new state — all under
/// the write lock. Returns the resulting state for callers that need to persist it.
///
/// Previously the pattern was read(lock) → clone → mutate → write(lock), which allowed a
/// concurrent writer to overwrite the first writer's changes (lost update). Holding the write
/// lock across the entire read-modify-write eliminates that race.
fn apply_state_mutation<F: FnOnce(&mut RuntimeState)>(
    inner: &Arc<RuntimeInner>,
    update: F,
) -> Arc<RuntimeState> {
    let mut guard = inner
        .snapshots
        .state
        .write()
        .unwrap_or_else(|p| p.into_inner());
    let mut next = guard.as_ref().clone();
    update(&mut next);
    let next = Arc::new(next);
    *guard = Arc::clone(&next);
    next
}

fn persist_state(inner: &Arc<RuntimeInner>, next: &RuntimeState) {
    let _ = inner
        .spec
        .state_store
        .save_snapshot(&RuntimeStateSnapshot::from_runtime_state(next));
}

pub(super) fn state_set_connection(inner: &Arc<RuntimeInner>, connection: ConnectionState) {
    let next = apply_state_mutation(inner, |state| {
        state.connection = connection;
    });
    persist_state(inner, &next);
}

pub(super) fn state_apply_envelope(inner: &Arc<RuntimeInner>, envelope: &Envelope) {
    let limits = &inner.spec.state_projection_limits;
    apply_state_mutation(inner, |state| {
        reduce_in_place_with_limits(state, envelope, limits);
    });
    // No persist — envelope projection fires on every inbound message and is too frequent for
    // per-call disk I/O. Meaningful state (connection, pending requests) is persisted on those
    // transitions. The in-memory projection is the source of truth for readers.
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
    persist_state(inner, &next);
}

pub(super) fn state_remove_pending_server_request(inner: &Arc<RuntimeInner>, rpc_id: &str) {
    let next = apply_state_mutation(inner, |state| {
        state.pending_server_requests.remove(rpc_id);
    });
    persist_state(inner, &next);
}

pub(super) fn state_clear_pending_server_requests(inner: &Arc<RuntimeInner>) {
    let next = apply_state_mutation(inner, |state| {
        state.pending_server_requests.clear();
    });
    persist_state(inner, &next);
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
