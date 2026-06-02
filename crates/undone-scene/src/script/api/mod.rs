//! The single-source-of-truth registry for the content-facing scripting API.
//!
//! `REGISTRY` (in `table.rs`) lists every method authors can call in conditions,
//! effect call-lists, and prose. Each entry names a pure accessor fn over borrowed
//! engine state; the Rhai engines (`rhai_bind`), the static gate (`validate.rs`),
//! the Minijinja prose objects (`minijinja_bind`), and the prose load gate
//! (`prose_validate`) are all driven from this one table.

use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::EffectError;
use crate::scene_ctx::SceneCtx;

/// A value produced by a read accessor, convertible to both script backends.
#[derive(Clone, Debug, PartialEq)]
pub enum ApiValue {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl ApiValue {
    pub fn into_dynamic(self) -> rhai::Dynamic {
        match self {
            ApiValue::Bool(b) => rhai::Dynamic::from(b),
            ApiValue::Int(i) => rhai::Dynamic::from(i),
            ApiValue::Str(s) => rhai::Dynamic::from(s),
        }
    }

    pub fn into_minijinja(self) -> minijinja::Value {
        match self {
            ApiValue::Bool(b) => minijinja::Value::from(b),
            ApiValue::Int(i) => minijinja::Value::from(i),
            ApiValue::Str(s) => minijinja::Value::from(s),
        }
    }
}

/// A literal argument as seen by an accessor, borrowed from the call site.
/// (The resolved npc/role ref is supplied as `ApiArg::Str` at index 0 by the
/// adapters; see `Receiver` doc.)
#[derive(Clone, Copy, Debug)]
pub enum ApiArg<'a> {
    Str(&'a str),
    Int(i64),
    Bool(bool),
}

impl<'a> ApiArg<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            ApiArg::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ApiArg::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ApiArg::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Why an accessor failed. Every variant must convert to BOTH a Rhai error and a
/// Minijinja error so the two adapters share one error vocabulary.
#[derive(Clone, Debug)]
pub enum ApiError {
    UnknownId { kind: &'static str, id: String },
    NoActiveNpc { sex: &'static str },
    UnboundRole { role: String },
    BadArgs { method: &'static str },
}

impl ApiError {
    pub fn message(&self) -> String {
        match self {
            ApiError::UnknownId { kind, id } => format!("unknown {kind} '{id}'"),
            ApiError::NoActiveNpc { sex } => format!("no active {sex} NPC in scene context"),
            ApiError::UnboundRole { role } => format!("no NPC bound to role '{role}'"),
            ApiError::BadArgs { method } => format!("bad arguments to '{method}'"),
        }
    }
    pub fn into_rhai(self) -> Box<rhai::EvalAltResult> {
        self.message().into()
    }
    pub fn into_minijinja(self) -> minijinja::Error {
        minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, self.message())
    }
}

/// Accessor signatures. Reads are pure over their borrows EXCEPT `checkSkill`/
/// `checkSkillRed`, which mutate the per-scene roll cache via `SceneCtx` interior
/// mutability — which is exactly why those two are condition-only (see `table.rs`).
pub type ReadFn = fn(&World, &PackRegistry, &SceneCtx, &[ApiArg]) -> Result<ApiValue, ApiError>;
pub type WriteFn =
    fn(&mut World, &mut SceneCtx, &PackRegistry, &[ApiArg]) -> Result<(), EffectError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_value_converts_to_rhai_and_minijinja() {
        let b = ApiValue::Bool(true);
        let i = ApiValue::Int(7);
        let s = ApiValue::Str("hi".to_string());

        // Rhai conversion
        assert!(b.clone().into_dynamic().as_bool().unwrap());
        assert_eq!(i.clone().into_dynamic().as_int().unwrap(), 7);
        assert_eq!(s.clone().into_dynamic().into_string().unwrap(), "hi");

        // Minijinja conversion
        assert_eq!(b.into_minijinja(), minijinja::Value::from(true));
        assert_eq!(i.into_minijinja(), minijinja::Value::from(7i64));
        assert_eq!(s.into_minijinja(), minijinja::Value::from("hi"));
    }
}
