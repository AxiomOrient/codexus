use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::time::{sleep, Duration};

use crate::runtime::state::ConnectionState;

use super::lifecycle::{detach_generation, spawn_connection_generation};
use super::state_projection::state_set_connection;
use super::{now_millis, RestartPolicy, RuntimeInner};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransportCloseKind {
    CleanExit,
    CrashExit,
    Unknown,
}

pub(super) async fn start_supervisor_task(inner: &Arc<RuntimeInner>) {
    let supervisor_inner = Arc::clone(inner);
    let supervisor_task = tokio::spawn(supervisor_loop(supervisor_inner));
    inner
        .tasks
        .supervisor_task
        .lock()
        .await
        .replace(supervisor_task);
}

pub(super) async fn wait_for_transport_close_signal(inner: &Arc<RuntimeInner>) -> bool {
    if inner.counters.shutting_down.load(Ordering::Acquire) {
        return false;
    }

    tokio::select! {
        _ = inner.io.transport_closed_signal.notified() => true,
        _ = inner.io.shutdown_signal.notified() => false,
    }
}

/// Exponential restart backoff with optional jitter.
///
/// `jitter_ms` adds a random offset to the base delay — callers derive it from subsecond
/// system time to spread fleet-wide restarts. Pass 0 in tests for deterministic assertions.
/// Allocation: none. Complexity: O(1).
pub(super) fn compute_restart_delay(
    attempt: u32,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
    jitter_ms: u64,
) -> Duration {
    let exp = attempt.min(20);
    let scaled = base_backoff_ms.saturating_mul(1u64 << exp);
    Duration::from_millis(scaled.min(max_backoff_ms).saturating_add(jitter_ms))
}

pub(super) async fn supervisor_loop(inner: Arc<RuntimeInner>) {
    let mut restart_attempts = 0u32;
    let mut generation_started_at_ms = now_millis();

    loop {
        if !wait_for_transport_close_signal(&inner).await {
            break;
        }

        if inner.counters.shutting_down.load(Ordering::Acquire) {
            break;
        }

        let close_kind = classify_transport_close(&inner).await;
        inner.counters.initialized.store(false, Ordering::Release);
        let generation = inner.counters.generation.load(Ordering::Acquire);
        if detach_generation(&inner).await.is_err() {
            state_set_connection(&inner, ConnectionState::Dead);
            break;
        }

        match inner.spec.supervisor_cfg.restart {
            RestartPolicy::Never => {
                state_set_connection(&inner, ConnectionState::Dead);
                break;
            }
            RestartPolicy::OnCrash {
                max_restarts,
                base_backoff_ms,
                max_backoff_ms,
            } => {
                if close_kind == TransportCloseKind::CleanExit {
                    state_set_connection(&inner, ConnectionState::Dead);
                    break;
                }

                let uptime_ms = now_millis().saturating_sub(generation_started_at_ms);
                if uptime_ms >= inner.spec.supervisor_cfg.restart_budget_reset_ms as i64 {
                    restart_attempts = 0;
                }

                if restart_attempts >= max_restarts {
                    state_set_connection(&inner, ConnectionState::Dead);
                    break;
                }

                state_set_connection(&inner, ConnectionState::Restarting { generation });
                // Derive jitter from subsecond system time: up to 25% of the base delay.
                // Prevents all agents sharing the same config from hammering the server
                // simultaneously after a crash (thundering herd).
                let jitter_ms = {
                    let base = base_backoff_ms.saturating_mul(1u64 << restart_attempts.min(20));
                    let range = base.min(max_backoff_ms) / 4;
                    if range > 0 {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.subsec_nanos() as u64 % range)
                            .unwrap_or(0)
                    } else {
                        0
                    }
                };
                let delay = compute_restart_delay(
                    restart_attempts,
                    base_backoff_ms,
                    max_backoff_ms,
                    jitter_ms,
                );
                restart_attempts = restart_attempts.saturating_add(1);
                tokio::select! {
                    _ = sleep(delay) => {}
                    _ = inner.io.shutdown_signal.notified() => break,
                }

                if inner.counters.shutting_down.load(Ordering::Acquire) {
                    break;
                }

                if spawn_connection_generation(&inner, generation.saturating_add(1))
                    .await
                    .is_err()
                {
                    state_set_connection(&inner, ConnectionState::Dead);
                    break;
                }
                generation_started_at_ms = now_millis();
            }
        }
    }
}

async fn classify_transport_close(inner: &Arc<RuntimeInner>) -> TransportCloseKind {
    let mut transport_guard = inner.tasks.transport.lock().await;
    let Some(transport) = transport_guard.as_mut() else {
        return TransportCloseKind::Unknown;
    };

    match transport.try_wait_exit() {
        Ok(Some(status)) if status.success() => TransportCloseKind::CleanExit,
        Ok(Some(_)) => TransportCloseKind::CrashExit,
        Ok(None) => TransportCloseKind::Unknown,
        Err(_) => TransportCloseKind::Unknown,
    }
}
