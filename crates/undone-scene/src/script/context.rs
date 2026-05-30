//! The per-call borrow-bridging context for Rhai handles.
//!
//! # Borrow-bridging decision (SPIKE, Task 2 — resolved)
//!
//! Rhai requires every registered type and every `Scope` variable to be
//! `Clone + 'static`. A handle therefore **cannot** hold a `&World` borrow —
//! the borrow is not `'static`. Both candidate mechanisms must launder the
//! lifetime through a raw pointer; the only question was *where the pointer
//! lives*:
//!
//! * **Candidate A** — the pointer lives inside the handle that is pushed into
//!   the `Scope` per call (`Gd { world: *const World }`). Each registered method
//!   dereferences `self.world`.
//! * **Candidate B** — the handles are zero-sized; the pointer lives in a
//!   thread-local "active eval context" set by a RAII guard around the
//!   `eval_ast_with_scope` call. Methods read it through `with_read_ctx`.
//!
//! Both are equally `unsafe` (a raw-pointer deref) and both confine that unsafe
//! to this module behind the same invariant:
//!
//! > **SAFETY invariant:** a context pointer is valid ONLY for the duration of
//! > one synchronous `eval_ast_with_scope` call on the single-threaded UI
//! > thread. The guard installs it immediately before the call and clears it
//! > immediately after; it is never stored, returned from a registered fn, or
//! > moved across threads.
//!
//! **Decision: Candidate B (thread-local context + `with_read_ctx`/`with_write_ctx`).**
//! The bench (`engine::tests::spike_binding`) showed the two within noise of each
//! other (AST evaluation dominates either pointer read), so the ~20%-perf escape
//! hatch in the decision rule selects on ergonomics. B wins decisively there: the
//! ~80 read + ~33 write methods ported in Tasks 4/5 become zero-boilerplate
//! `with_read_ctx(|world, reg, ctx| …)` closures instead of every one of the six
//! handle types having to carry and thread three raw pointers. It also models the
//! read/write split cleanly — the effect engine installs a `WriteCtx` (with
//! `&mut World`), the condition engine a `ReadCtx`, and the same ZST handles work
//! against whichever is installed.
//!
//! Candidate A is preserved below only as `#[cfg(test)]` spike code so the bench
//! that justified this decision keeps running; it is not part of the real path.

use std::cell::Cell;

use crate::scene_ctx::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

// ---------------------------------------------------------------------------
// Candidate B — the chosen mechanism: thread-local context + accessors.
// ---------------------------------------------------------------------------

/// Raw, lifetime-erased pointers to the state one script evaluation reads.
/// Stored in a thread-local for the duration of a single `eval` call only.
#[derive(Clone, Copy)]
pub(crate) struct ReadCtx {
    pub(crate) world: *const World,
    pub(crate) registry: *const PackRegistry,
    pub(crate) ctx: *const SceneCtx,
}

thread_local! {
    static READ_CTX: Cell<Option<ReadCtx>> = const { Cell::new(None) };
}

/// RAII guard that installs a [`ReadCtx`] for the lifetime of one eval call and
/// clears it on drop (even on panic / early return).
pub(crate) struct ReadCtxGuard {
    prev: Option<ReadCtx>,
}

impl ReadCtxGuard {
    /// Install a read context built from live borrows.
    ///
    /// # Safety
    /// The caller must keep `world`, `registry`, and `ctx` borrowed (and not
    /// moved) for the entire lifetime of the returned guard — i.e. for the whole
    /// `eval_ast_with_scope` call. The guard's `Drop` restores the previous
    /// context, so nested installs are sound.
    pub(crate) fn install(world: &World, registry: &PackRegistry, ctx: &SceneCtx) -> Self {
        let new = ReadCtx {
            world: world as *const World,
            registry: registry as *const PackRegistry,
            ctx: ctx as *const SceneCtx,
        };
        let prev = READ_CTX.with(|c| c.replace(Some(new)));
        ReadCtxGuard { prev }
    }
}

impl Drop for ReadCtxGuard {
    fn drop(&mut self) {
        READ_CTX.with(|c| c.set(self.prev));
    }
}

/// Build a Rhai runtime error carrying an "unknown &lt;kind&gt; '&lt;id&gt;'" message.
///
/// Returned by read methods when a content id fails to resolve at runtime. The
/// load-time gate normally catches these first (Task 6), but a runtime error
/// keeps fail-loud behaviour if one slips through.
pub(crate) fn unknown_id_err(kind: &str, id: &str) -> Box<rhai::EvalAltResult> {
    format!("unknown {kind} '{id}'").into()
}

/// Run `f` with shared access to the currently-installed evaluation context.
///
/// Returns a Rhai error if called outside an active eval (a programming bug —
/// it means a handle method ran without a guard installed).
///
/// # Safety
/// Relies on the [`ReadCtxGuard`] SAFETY invariant: the pointers are valid for
/// the duration of the eval call, which is exactly when handle methods run.
pub(crate) fn with_read_ctx<R>(
    f: impl FnOnce(&World, &PackRegistry, &SceneCtx) -> Result<R, Box<rhai::EvalAltResult>>,
) -> Result<R, Box<rhai::EvalAltResult>> {
    // During condition eval a ReadCtx is installed. During effect eval only a
    // WriteCtx is installed (a single &mut World borrow); read methods on the
    // effect engine fall back to reading through it. Exactly one is ever present.
    if let Some(ptrs) = READ_CTX.with(|c| c.get()) {
        // SAFETY: see module-level invariant. The guard that set these pointers
        // is still on the stack for the duration of this eval call.
        let world = unsafe { &*ptrs.world };
        let registry = unsafe { &*ptrs.registry };
        let ctx = unsafe { &*ptrs.ctx };
        return f(world, registry, ctx);
    }
    if let Some(ptrs) = WRITE_CTX.with(|c| c.get()) {
        // SAFETY: same invariant; reborrow the write context's mut pointers as
        // shared. Statements run sequentially on one thread, so no read overlaps
        // a write.
        let world = unsafe { &*ptrs.world };
        let registry = unsafe { &*ptrs.registry };
        let ctx = unsafe { &*ptrs.ctx };
        return f(world, registry, ctx);
    }
    Err("script evaluated with no evaluation context installed".into())
}

// ---------------------------------------------------------------------------
// Write context — installed during effect eval (effect engine only).
// ---------------------------------------------------------------------------

/// Raw, lifetime-erased pointers to the mutable state one effect call-list
/// touches, plus an error sink for continue-on-error semantics.
#[derive(Clone, Copy)]
pub(crate) struct WriteCtx {
    pub(crate) world: *mut World,
    pub(crate) ctx: *mut SceneCtx,
    pub(crate) registry: *const PackRegistry,
    pub(crate) errors: *mut Vec<String>,
}

thread_local! {
    static WRITE_CTX: Cell<Option<WriteCtx>> = const { Cell::new(None) };
}

/// RAII guard installing a [`WriteCtx`] for one effect-eval call.
pub(crate) struct WriteCtxGuard {
    prev: Option<WriteCtx>,
}

impl WriteCtxGuard {
    /// # Safety
    /// The caller must keep all four borrows live (and not move them) for the
    /// whole `eval_ast_with_scope` call — i.e. the lifetime of the guard.
    pub(crate) fn install(
        world: &mut World,
        ctx: &mut SceneCtx,
        registry: &PackRegistry,
        errors: &mut Vec<String>,
    ) -> Self {
        let new = WriteCtx {
            world: world as *mut World,
            ctx: ctx as *mut SceneCtx,
            registry: registry as *const PackRegistry,
            errors: errors as *mut Vec<String>,
        };
        let prev = WRITE_CTX.with(|c| c.replace(Some(new)));
        WriteCtxGuard { prev }
    }
}

impl Drop for WriteCtxGuard {
    fn drop(&mut self) {
        WRITE_CTX.with(|c| c.set(self.prev));
    }
}

/// Run an effect mutator `f` against the installed write context.
///
/// Preserves the engine's best-effort *continue-on-error* semantics: a mutator
/// that fails records its error in the shared sink and the call-list keeps
/// running (the mutator returns to Rhai as a no-op success). Mirrors the
/// pre-Rhai `apply_effect` loop that collected `effect_errors` and emitted one
/// `ErrorOccurred` per failure without aborting the remaining effects.
///
/// # Safety
/// Relies on the [`WriteCtxGuard`] invariant; the &mut borrows are exclusive for
/// the duration of one sequential eval call.
pub(crate) fn with_write_ctx(
    f: impl FnOnce(&mut World, &mut SceneCtx, &PackRegistry) -> Result<(), crate::effects::EffectError>,
) {
    let Some(ptrs) = WRITE_CTX.with(|c| c.get()) else {
        // No write context: a write mutator ran on a non-effect engine. This is a
        // programming error; record nothing actionable here (the load-time gate
        // prevents write calls in conditions).
        return;
    };
    // SAFETY: see invariant. Sequential single-threaded eval — no aliasing.
    let world = unsafe { &mut *ptrs.world };
    let ctx = unsafe { &mut *ptrs.ctx };
    let registry = unsafe { &*ptrs.registry };
    if let Err(e) = f(world, ctx, registry) {
        let errors = unsafe { &mut *ptrs.errors };
        errors.push(format!("[scene-engine] effect error: {e}"));
    }
}

/// Candidate A (rejected) — pointer carried inside the scope-injected handle.
/// Kept only so the bench in `engine::tests::spike_binding` can compare it.
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct GdA {
    pub(crate) world: *const World,
}

#[cfg(test)]
impl GdA {
    pub(crate) fn week(&mut self) -> i64 {
        // SAFETY: spike-only; the pointer comes from a `&World` that outlives the
        // single eval call in the bench.
        unsafe { (*self.world).game_data.week as i64 }
    }
}
