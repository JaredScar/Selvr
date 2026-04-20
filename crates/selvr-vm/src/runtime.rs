//! Async event loop integration.
//!
//! Selvr's `async fn` / `.await` model maps to the browser's microtask queue:
//!
//!  1. An `async fn` returns a *coroutine handle* (`CoroutineId`).
//!  2. `.await` suspends the current coroutine and yields to the event loop.
//!  3. When the awaited value resolves the coroutine is resumed.
//!
//! On WASM the event loop is the browser's own.  JS promise resolution calls
//! `SELVR_resume(id)` (exported from `wasm.rs`) which pokes the scheduler.
//! On native this is a cooperative "mini event loop" that runs all queued
//! coroutines to completion synchronously.

use std::collections::VecDeque;

// ── Coroutine handle ──────────────────────────────────────────────────────────

/// Identifies an in-flight async call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoroutineId(pub u32);

// ── Scheduler ────────────────────────────────────────────────────────────────

/// Holds the run-queue of coroutines ready to be resumed.
#[derive(Default)]
pub struct Scheduler {
    next_id: u32,
    /// Coroutines ready to run: (id, fn_idx, resume_value).
    ready:   VecDeque<(CoroutineId, u16, Option<Vec<u8>>)>,
}

impl Scheduler {
    pub fn new() -> Self { Self::default() }

    /// Enqueue a new coroutine.  Returns its ID.
    pub fn spawn(&mut self, fn_idx: u16) -> CoroutineId {
        let id = CoroutineId(self.next_id);
        self.next_id += 1;
        self.ready.push_back((id, fn_idx, None));
        id
    }

    /// Mark a coroutine as ready to resume (called from JS via `SELVR_resume`).
    pub fn resume(&mut self, id: CoroutineId, value: Option<Vec<u8>>) {
        self.ready.push_back((id, 0, value));
    }

    /// Drain one coroutine from the queue.
    pub fn poll(&mut self) -> Option<(CoroutineId, u16, Option<Vec<u8>>)> {
        self.ready.pop_front()
    }

    pub fn is_idle(&self) -> bool { self.ready.is_empty() }
}

/// Global (thread-local on native; single-threaded on WASM) scheduler.
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    pub static SCHEDULER: std::cell::RefCell<Scheduler> =
        std::cell::RefCell::new(Scheduler::new());
}

/// Run all pending coroutines to completion (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn run_until_idle() {
    // In a real implementation this would drive the VM for each coroutine.
    // For now it drains the queue (the VM integration happens in `vm.rs`).
    SCHEDULER.with(|s| {
        while !s.borrow().is_idle() {
            s.borrow_mut().poll();
        }
    });
}
