//! Rhai registration driven by `REGISTRY`. Replaces `read_api`/`write_api`.
//!
//! The receiver handles are zero-sized (`W`, `Gd`, …) except `Npc`, which carries
//! the unresolved ref from the `npc(ref)` constructor. Rhai keys methods on the
//! receiver Rust type, so each `MethodDescriptor` is registered against the ZST for
//! its `Receiver`, with a typed adapter that marshals Rhai args into `[ApiArg]`,
//! funnels through `with_read_ctx` / `with_write_ctx`, and converts the result.

use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString};

use super::table::REGISTRY;
use super::{
    Accessor, ApiArg, ApiError, ApiValue, ArgShape, MethodDescriptor, ReadFn, Receiver, WriteFn,
};
use crate::script::context::{with_read_ctx, with_write_ctx};
use crate::script::validate::IdKind;

// ── Receiver handles (replace the read_api/write_api ZSTs) ────────────────────

#[derive(Clone)]
pub struct W;
#[derive(Clone)]
pub struct Gd;
#[derive(Clone)]
pub struct M;
#[derive(Clone)]
pub struct F;
#[derive(Clone)]
pub struct Role;
#[derive(Clone)]
pub struct Scene;

/// `npc(ref)` handle — carries the unresolved NPC reference for chained calls.
#[derive(Clone)]
pub struct Npc {
    id: String,
}

fn make_npc(id: &str) -> Npc {
    Npc { id: id.to_string() }
}

// ── Read marshalling ──────────────────────────────────────────────────────────

fn read_call(f: ReadFn, args: &[ApiArg]) -> Result<Dynamic, Box<EvalAltResult>> {
    with_read_ctx(|w, r, c| f(w, r, c, args).map_err(ApiError::into_rhai))
        .map(ApiValue::into_dynamic)
}

/// Register every read descriptor for receiver `T` (the ZST). Reads register on
/// both the condition and effect engines.
fn reg_reads<T: Clone + 'static>(engine: &mut Engine) {
    engine.register_type::<T>();
}

fn reg_read<T: Clone + 'static>(engine: &mut Engine, d: &'static MethodDescriptor, f: ReadFn) {
    let name = d.name;
    match d.args {
        ArgShape::None => {
            engine.register_fn(name, move |_t: &mut T| read_call(f, &[]));
        }
        ArgShape::Id(_) | ArgShape::Str => {
            engine.register_fn(name, move |_t: &mut T, a: ImmutableString| {
                read_call(f, &[ApiArg::Str(&a)])
            });
        }
        ArgShape::IdInt(_) => {
            engine.register_fn(name, move |_t: &mut T, a: ImmutableString, b: i64| {
                read_call(f, &[ApiArg::Str(&a), ApiArg::Int(b)])
            });
        }
        ArgShape::StrStr => {
            engine.register_fn(
                name,
                move |_t: &mut T, a: ImmutableString, b: ImmutableString| {
                    read_call(f, &[ApiArg::Str(&a), ApiArg::Str(&b)])
                },
            );
        }
        // Reads never use Int / Bool / StrOpt.
        _ => {}
    }
}

// ── Write marshalling ─────────────────────────────────────────────────────────

/// Register a write descriptor for a value-receiver ZST `T` (`W`/`Gd`/`Scene`).
fn reg_write<T: Clone + 'static>(engine: &mut Engine, d: &'static MethodDescriptor, f: WriteFn) {
    let name = d.name;
    match d.args {
        ArgShape::Int { .. } => {
            engine.register_fn(name, move |_t: &mut T, a: i64| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Int(a)]));
            });
        }
        ArgShape::IdInt(_) => {
            engine.register_fn(name, move |_t: &mut T, a: ImmutableString, b: i64| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&a), ApiArg::Int(b)]));
            });
        }
        // advanceArc(arc, state) — the only 2-string-arg Id.
        ArgShape::Id(IdKind::Arc) => {
            engine.register_fn(
                name,
                move |_t: &mut T, a: ImmutableString, b: ImmutableString| {
                    with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&a), ApiArg::Str(&b)]));
                },
            );
        }
        ArgShape::Id(_) | ArgShape::Str => {
            engine.register_fn(name, move |_t: &mut T, a: ImmutableString| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&a)]));
            });
        }
        ArgShape::Bool => {
            engine.register_fn(name, move |_t: &mut T, a: bool| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Bool(a)]));
            });
        }
        // setVirgin: overloaded arity → two native registrations, one accessor.
        ArgShape::StrOpt => {
            engine.register_fn(name, move |_t: &mut T, a: bool| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Bool(a)]));
            });
            engine.register_fn(name, move |_t: &mut T, a: bool, b: ImmutableString| {
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Bool(a), ApiArg::Str(&b)]));
            });
        }
        ArgShape::None | ArgShape::StrStr | ArgShape::StrInt => {}
    }
}

/// Register an `npc(ref).method` write. The ref is taken from the handle and
/// injected as `ApiArg` index 0; the method's own arg follows.
fn reg_write_npc(engine: &mut Engine, d: &'static MethodDescriptor, f: WriteFn) {
    let name = d.name;
    if name == "npc" {
        // The constructor — a free fn returning the chained handle (not a mutator).
        engine.register_fn("npc", make_npc);
        return;
    }
    match d.args {
        ArgShape::Int { .. } => {
            engine.register_fn(name, move |this: &mut Npc, a: i64| {
                let id = this.id.clone();
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&id), ApiArg::Int(a)]));
            });
        }
        ArgShape::Id(_) | ArgShape::Str => {
            engine.register_fn(name, move |this: &mut Npc, a: ImmutableString| {
                let id = this.id.clone();
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&id), ApiArg::Str(&a)]));
            });
        }
        ArgShape::Bool => {
            engine.register_fn(name, move |this: &mut Npc, a: bool| {
                let id = this.id.clone();
                with_write_ctx(|w, c, r| f(w, c, r, &[ApiArg::Str(&id), ApiArg::Bool(a)]));
            });
        }
        _ => {}
    }
}

// ── Public registration entry points ──────────────────────────────────────────

/// Register the read surface (valid on both the condition and effect engines).
pub fn register_reads(engine: &mut Engine) {
    reg_reads::<W>(engine);
    reg_reads::<Gd>(engine);
    reg_reads::<M>(engine);
    reg_reads::<F>(engine);
    reg_reads::<Role>(engine);
    reg_reads::<Scene>(engine);
    for d in REGISTRY {
        let Accessor::Read(f) = d.accessor else {
            continue;
        };
        match d.receiver {
            Receiver::W => reg_read::<W>(engine, d, f),
            Receiver::Gd => reg_read::<Gd>(engine, d, f),
            Receiver::M => reg_read::<M>(engine, d, f),
            Receiver::F => reg_read::<F>(engine, d, f),
            Receiver::Role => reg_read::<Role>(engine, d, f),
            Receiver::Scene => reg_read::<Scene>(engine, d, f),
            Receiver::Npc => {} // no npc reads
        }
    }
}

/// Register the write surface (effect engine only).
pub fn register_writes(engine: &mut Engine) {
    engine.register_type::<Npc>();
    for d in REGISTRY {
        let Accessor::Write(f) = d.accessor else {
            continue;
        };
        match d.receiver {
            Receiver::W => reg_write::<W>(engine, d, f),
            Receiver::Gd => reg_write::<Gd>(engine, d, f),
            Receiver::Scene => reg_write::<Scene>(engine, d, f),
            Receiver::Npc => reg_write_npc(engine, d, f),
            // m/f/role have no writes.
            Receiver::M | Receiver::F | Receiver::Role => {}
        }
    }
}
