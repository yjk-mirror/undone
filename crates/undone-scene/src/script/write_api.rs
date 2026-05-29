//! `register_write_api()` + effect mutator calls.
//!
//! Registered ONLY on the effect engine, so a mutating call inside a condition
//! is not resolvable on the condition engine — surfaced as a load error by the
//! Task-6 dry-run gate. The method bodies are ported from the `apply_effect`
//! arms in `effects.rs` in Task 5.

/// Register every effect mutator onto the effect `engine`.
///
/// Filled in Task 5.
pub fn register_write_api(_engine: &mut rhai::Engine) {
    // Filled in Task 5.
}
