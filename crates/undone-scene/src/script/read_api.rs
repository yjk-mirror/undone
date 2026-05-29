//! `register_read_api()` + read receiver handles (`W`, `Gd`, `M`, `F`, `Role`, `Scene`).
//!
//! The receiver handles are zero-sized; they read the thread-local evaluation
//! context installed by `context::ReadCtxGuard` (see the borrow-bridging
//! decision in `context.rs`). The method bodies are ported from the read methods
//! dispatched in `undone_expr::eval` in Task 4.

/// Register every read receiver type and its methods onto `engine`.
///
/// Filled in Task 4. Registered on BOTH the condition engine and the effect
/// engine so effect scripts can branch on reads.
pub fn register_read_api(_engine: &mut rhai::Engine) {
    // Filled in Task 4.
}
