//! Minijinja prose dispatch driven by `REGISTRY`. Replaces the owned snapshot
//! objects in `template_ctx.rs`.
//!
//! Each receiver is a zero-sized `Object` view. `call_method` looks the call up in
//! `REGISTRY`, requires `contexts.prose`, marshals the Minijinja args into `[ApiArg]`,
//! and runs the SAME read accessor the Rhai engine uses — against live `World` via
//! the thread-local read borrows (`with_read_borrows`). No snapshot is materialized.

use std::sync::Arc;

use minijinja::value::{Object, ObjectRepr, Value};
use minijinja::{Error, ErrorKind, State};

use super::{lookup, Accessor, ApiArg, ArgShape, Receiver};
use crate::script::context::with_read_borrows;

/// Convert the Minijinja call args into `[ApiArg]` per the descriptor's `ArgShape`.
///
/// The prose-callable read surface only ever takes string args or none (the only
/// int-bearing reads — `checkSkill`/`checkSkillRed` — are condition-only and filtered
/// out before this runs), so this handles the string shapes and rejects the rest
/// defensively.
fn marshal_args<'a>(
    shape: ArgShape,
    method: &str,
    args: &'a [Value],
) -> Result<Vec<ApiArg<'a>>, Error> {
    let str_at = |i: usize| -> Result<&'a str, Error> {
        args.get(i).and_then(|v| v.as_str()).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                format!("'{method}' expects a string argument at position {i}"),
            )
        })
    };
    Ok(match shape {
        ArgShape::None => Vec::new(),
        ArgShape::Id(_) | ArgShape::Str => vec![ApiArg::Str(str_at(0)?)],
        ArgShape::StrStr => vec![ApiArg::Str(str_at(0)?), ApiArg::Str(str_at(1)?)],
        // Not prose-reachable (contexts.prose == false): handle defensively.
        ArgShape::IdInt(_)
        | ArgShape::Int { .. }
        | ArgShape::Bool
        | ArgShape::StrInt
        | ArgShape::StrOpt => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("'{method}' is not callable in prose"),
            ))
        }
    })
}

/// Shared dispatch: look up `(recv, method)`, require prose-availability, marshal,
/// run the accessor against live `World`, and convert the result.
fn dispatch(recv: Receiver, token: &str, method: &str, args: &[Value]) -> Result<Value, Error> {
    let Some(d) = lookup(recv, method).filter(|d| d.contexts.prose) else {
        return Err(Error::new(
            ErrorKind::UnknownMethod,
            format!("{token} has no prose method '{method}'"),
        ));
    };
    let Accessor::Read(f) = d.accessor else {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            format!("'{token}.{method}' is a write method, not callable in prose"),
        ));
    };
    let api_args = marshal_args(d.args, method, args)?;
    match with_read_borrows(|w, r, c| f(w, r, c, &api_args)) {
        Some(Ok(v)) => Ok(v.into_minijinja()),
        Some(Err(e)) => Err(e.into_minijinja()),
        None => Err(Error::new(
            ErrorKind::InvalidOperation,
            "prose rendered with no evaluation context installed",
        )),
    }
}

macro_rules! view {
    ($name:ident, $recv:expr, $token:literal) => {
        #[derive(Debug)]
        pub struct $name;
        impl Object for $name {
            fn repr(self: &Arc<Self>) -> ObjectRepr {
                ObjectRepr::Plain
            }
            fn call_method(
                self: &Arc<Self>,
                _state: &State<'_, '_>,
                method: &str,
                args: &[Value],
            ) -> Result<Value, Error> {
                dispatch($recv, $token, method, args)
            }
        }
    };
}

view!(WView, Receiver::W, "w");
view!(GdView, Receiver::Gd, "gd");
view!(MView, Receiver::M, "m");
view!(FView, Receiver::F, "f");
view!(RoleView, Receiver::Role, "role");
view!(SceneView, Receiver::Scene, "scene");
