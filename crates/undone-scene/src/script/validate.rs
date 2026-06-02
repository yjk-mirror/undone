//! Static source-scan validation — the load-time fail-fast guarantee.
//!
//! Rhai's `compile()` + `strict_variables` catches syntax and unknown *variables*,
//! but NOT unknown content IDs, unknown *methods* (functions resolve at runtime),
//! or arg mistakes. The legacy loader guaranteed all of these at load via
//! `validate_call_signature` + `validate_condition_ids` + `validate_effects`. This
//! module reconstructs that guarantee by scanning the authored script source —
//! the design mandates content-id args be string literals precisely so a static
//! scan is complete across ALL branches (including short-circuited ones a runtime
//! dry-run would skip).
//!
//! What it checks (faithful to the legacy passes):
//! 1. Every `receiver.method(...)` is a known method for that receiver. For
//!    conditions only the READ methods are known; an effect mutator in a
//!    condition is therefore an "unknown method" — statically enforcing the
//!    read/write split (design §4.1).
//! 2. Arity matches the method signature.
//! 3. Content-id string-literal args resolve against the registry
//!    (`hasTrait`→trait, `getSkill`→skill, `addStat`→stat, `advanceArc`→arc+state, …).
//!    A content-id arg that is not a string literal is rejected (the string-literal rule).
//! 4. Step-delta args that the legacy `EffectDef` stored as `i8` must be in
//!    `i8` range (so an out-of-range delta fails at load instead of wrapping).

use undone_packs::PackRegistry;

use crate::script::compiled::ScriptError;

// ---------------------------------------------------------------------------
// Method spec table
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdKind {
    Trait,
    NpcTrait,
    Skill,
    Stat,
    Category,
    /// `advanceArc(arc, state)` — resolve arc at index 0 and validate the state
    /// literal at index 1 belongs to that arc.
    Arc,
}

/// What a single authored method call is allowed to look like.
struct MethodSpec {
    /// Minimum argument count.
    arity: usize,
    /// Maximum argument count (== `arity` unless the method is overloaded).
    arity_max: usize,
    /// `(arg_index, kind)` for an argument that must be a registry content-id
    /// string literal.
    id_arg: Option<(usize, IdKind)>,
    /// Argument indices that must be an integer literal (legacy typed-arg check).
    int_args: &'static [usize],
    /// Argument indices that the legacy `EffectDef` stored as `i8` and so must
    /// be an integer literal in `i8` range.
    i8_args: &'static [usize],
}

const fn spec(arity: usize) -> MethodSpec {
    MethodSpec {
        arity,
        arity_max: arity,
        id_arg: None,
        int_args: &[],
        i8_args: &[],
    }
}

/// A method whose arg count may range `min..=max` (overloaded arity).
const fn spec_arity(min: usize, max: usize) -> MethodSpec {
    MethodSpec {
        arity: min,
        arity_max: max,
        id_arg: None,
        int_args: &[],
        i8_args: &[],
    }
}

const fn spec_id(arity: usize, idx: usize, kind: IdKind) -> MethodSpec {
    MethodSpec {
        arity,
        arity_max: arity,
        id_arg: Some((idx, kind)),
        int_args: &[],
        i8_args: &[],
    }
}

const fn spec_id_int(arity: usize, idx: usize, kind: IdKind, int_idx: usize) -> MethodSpec {
    MethodSpec {
        arity,
        arity_max: arity,
        id_arg: Some((idx, kind)),
        int_args: int_idx_slice(int_idx),
        i8_args: &[],
    }
}

const fn int_idx_slice(idx: usize) -> &'static [usize] {
    match idx {
        1 => &[1],
        _ => &[0],
    }
}

const fn spec_i8(arity: usize, i8_args: &'static [usize]) -> MethodSpec {
    MethodSpec {
        arity,
        arity_max: arity,
        id_arg: None,
        int_args: &[],
        i8_args,
    }
}

/// Look up the READ method surface (valid in conditions). Derived from the
/// single-source-of-truth `REGISTRY`: a `(receiver, method)` is a "read method" for
/// the gate iff its descriptor is valid in the condition context.
fn read_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    let recv = crate::script::api::receiver_from_token(receiver)?;
    let d = crate::script::api::lookup(recv, method)?;
    if !d.contexts.condition {
        return None;
    }
    Some(method_spec_from_argshape(d.args))
}

/// Look up the WRITE (effect-mutator) surface, derived from `REGISTRY`.
fn write_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    let recv = crate::script::api::receiver_from_token(receiver)?;
    let d = crate::script::api::lookup(recv, method)?;
    if !d.contexts.effect {
        return None;
    }
    Some(method_spec_from_argshape(d.args))
}

/// Derive the gate's `MethodSpec` (arity, id-kind, int/i8 constraints) from a
/// descriptor's declarative `ArgShape` — the inverse of the old hand-written table,
/// now driven from the registry. NOTE: `IdInt` int-checks arg 1, so `skillIncrease`/
/// `addStat`/`setStat` are slightly stricter than the legacy `spec_id(2,..)` (which
/// did not). Live content passes either way (all call them with integer literals).
fn method_spec_from_argshape(args: crate::script::api::ArgShape) -> MethodSpec {
    use crate::script::api::ArgShape;
    match args {
        ArgShape::None => spec(0),
        // advanceArc(arc, state) — the only 2-source-arg Id (state validated at idx 1).
        ArgShape::Id(IdKind::Arc) => spec_id(2, 0, IdKind::Arc),
        ArgShape::Id(kind) => spec_id(1, 0, kind),
        ArgShape::IdInt(kind) => spec_id_int(2, 0, kind, 1),
        ArgShape::Int { i8_range: true } => spec_i8(1, &[0]),
        ArgShape::Int { i8_range: false } => spec(1),
        ArgShape::Str => spec(1),
        ArgShape::Bool => spec(1),
        ArgShape::StrInt => spec(2),
        ArgShape::StrStr => spec(2),
        ArgShape::StrOpt => spec_arity(1, 2),
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Str(String),
    Int(i64),
    Dot,
    LParen,
    RParen,
    /// Anything else (operators, commas, bools, floats…). We only need structure.
    Other,
}

/// Tokenize enough of the Rhai source to find call sites and their literal args.
/// Returns an error on an unterminated string (a syntax error `compile` also
/// reports, but we may run before/independently).
fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let bytes: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '"' => {
                // double-quoted string with \ escapes
                let mut s = String::new();
                i += 1;
                loop {
                    if i >= bytes.len() {
                        return Err("unterminated string literal".into());
                    }
                    match bytes[i] {
                        '\\' => {
                            i += 1;
                            if i < bytes.len() {
                                s.push(bytes[i]);
                                i += 1;
                            }
                        }
                        '"' => {
                            i += 1;
                            break;
                        }
                        ch => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                toks.push(Tok::Str(s));
            }
            '\'' => {
                // single-quoted string with \ escapes. Rhai conditions use double
                // quotes, but Minijinja prose accepts both — the live pack uses
                // single-quoted ids (e.g. `w.getSkill('FEMININITY')`), so the prose
                // gate must tokenize them as string literals too (design §5.4).
                let mut s = String::new();
                i += 1;
                loop {
                    if i >= bytes.len() {
                        return Err("unterminated string literal".into());
                    }
                    match bytes[i] {
                        '\\' => {
                            i += 1;
                            if i < bytes.len() {
                                s.push(bytes[i]);
                                i += 1;
                            }
                        }
                        '\'' => {
                            i += 1;
                            break;
                        }
                        ch => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                toks.push(Tok::Str(s));
            }
            '.' => {
                toks.push(Tok::Dot);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
                    i += 1;
                }
                toks.push(Tok::Ident(bytes[start..i].iter().collect()));
            }
            c if c.is_ascii_digit()
                || (c == '-' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit()) =>
            {
                let start = i;
                if bytes[i] == '-' {
                    i += 1;
                }
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let lit: String = bytes[start..i].iter().collect();
                match lit.parse::<i64>() {
                    Ok(n) => toks.push(Tok::Int(n)),
                    Err(_) => toks.push(Tok::Other),
                }
            }
            _ => {
                toks.push(Tok::Other);
                i += 1;
            }
        }
    }
    Ok(toks)
}

// ---------------------------------------------------------------------------
// Call extraction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Arg {
    Str(String),
    Int(i64),
    Other,
}

struct Call {
    receiver: Option<String>,
    method: String,
    args: Vec<Arg>,
}

/// Extract every `name(args)` and `recv.name(args)` and `).name(args)` call.
/// `receiver` = the handle token for `recv.name`, `Some("npc")` for a `).name`
/// chained call (only `npc(...)` produces a chained method call in our
/// vocabulary), or `None` for a bare `name(...)` free call.
fn extract_calls(toks: &[Tok]) -> Vec<Call> {
    let mut calls = Vec::new();
    let mut idx = 0;
    while idx < toks.len() {
        if let Tok::Ident(name) = &toks[idx] {
            if matches!(toks.get(idx + 1), Some(Tok::LParen)) {
                // Determine the receiver from what precedes this Ident.
                let receiver = match (idx >= 2, idx >= 1) {
                    (true, _) if toks[idx - 1] == Tok::Dot => match &toks[idx - 2] {
                        Tok::Ident(r) => Some(r.clone()),
                        Tok::RParen => Some("npc".to_string()),
                        _ => None,
                    },
                    (false, true) if toks[idx - 1] == Tok::Dot => None,
                    _ => None,
                };
                // Parse args between the matching parens.
                let (args, end) = parse_args(toks, idx + 1);
                calls.push(Call {
                    receiver,
                    method: name.clone(),
                    args,
                });
                idx = end;
                continue;
            }
        }
        idx += 1;
    }
    calls
}

/// Parse the argument list starting at `lparen` (index of the `(`). Returns the
/// top-level args and the index just past the matching `)`.
fn parse_args(toks: &[Tok], lparen: usize) -> (Vec<Arg>, usize) {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut i = lparen;
    let mut current: Option<Arg> = None;
    let mut saw_any = false;
    while i < toks.len() {
        match &toks[i] {
            Tok::LParen => {
                depth += 1;
                if depth > 1 {
                    current = Some(Arg::Other);
                }
                i += 1;
            }
            Tok::RParen => {
                depth -= 1;
                if depth == 0 {
                    if saw_any {
                        args.push(current.take().unwrap_or(Arg::Other));
                    }
                    return (args, i + 1);
                }
                i += 1;
            }
            Tok::Other if depth == 1 => {
                // could be a comma (arg separator) or an operator inside the arg.
                // We approximate: treat every top-level `Other` as a separator
                // boundary only when it is a comma. We can't see the char, so we
                // conservatively flush on any top-level Other that follows a
                // value — good enough since args here are simple literals/refs.
                if current.is_some() {
                    args.push(current.take().unwrap());
                }
                i += 1;
            }
            tok if depth == 1 => {
                saw_any = true;
                current = Some(match tok {
                    Tok::Str(s) => Arg::Str(s.clone()),
                    Tok::Int(n) => Arg::Int(*n),
                    _ => Arg::Other,
                });
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    (args, i)
}

// ---------------------------------------------------------------------------
// Validation entry points
// ---------------------------------------------------------------------------

/// Validate a condition source. Only READ methods are valid (effect mutators in
/// a condition are "unknown method" → enforces the read/write split at load).
pub(crate) fn validate_condition_source(
    src: &str,
    registry: &PackRegistry,
    context: &str,
) -> Result<(), ScriptError> {
    validate(src, registry, context, false)
}

/// Validate an effect source. READ and WRITE methods are valid.
pub(crate) fn validate_effect_source(
    src: &str,
    registry: &PackRegistry,
    context: &str,
) -> Result<(), ScriptError> {
    validate(src, registry, context, true)
}

fn validate(
    src: &str,
    registry: &PackRegistry,
    context: &str,
    allow_write: bool,
) -> Result<(), ScriptError> {
    let toks = tokenize(src).map_err(|message| ScriptError::Compile {
        context: context.into(),
        message,
        source_text: src.into(),
    })?;
    for call in extract_calls(&toks) {
        validate_call(&call, registry, context, src, allow_write)?;
    }
    Ok(())
}

fn validate_call(
    call: &Call,
    registry: &PackRegistry,
    context: &str,
    src: &str,
    allow_write: bool,
) -> Result<(), ScriptError> {
    let receiver = match &call.receiver {
        Some(r) => r.as_str(),
        // A bare free call: only `npc(...)` is legal, and only in effects.
        None => {
            if call.method == "npc" {
                "npc"
            } else {
                // Not a handle method (could be a Rhai builtin) — leave to compile.
                return Ok(());
            }
        }
    };

    // Look up the method. Conditions: read only. Effects: read ∪ write.
    let found = read_spec(receiver, &call.method).or_else(|| {
        if allow_write {
            write_spec(receiver, &call.method)
        } else {
            None
        }
    });

    let Some(spec) = found else {
        // If it's a known WRITE method but we're in a condition, give the precise
        // read/write-split message; otherwise it's simply unknown.
        if !allow_write && write_spec(receiver, &call.method).is_some() {
            return Err(compile_err(
                context,
                src,
                format!(
                    "effect mutator '{}.{}' is not allowed in a condition",
                    receiver, call.method
                ),
            ));
        }
        return Err(compile_err(
            context,
            src,
            format!("unknown method '{}.{}'", receiver, call.method),
        ));
    };

    if call.args.len() < spec.arity || call.args.len() > spec.arity_max {
        let expect = if spec.arity == spec.arity_max {
            format!("{}", spec.arity)
        } else {
            format!("{}..={}", spec.arity, spec.arity_max)
        };
        return Err(compile_err(
            context,
            src,
            format!(
                "method '{}.{}' expects {} arg(s), got {}",
                receiver,
                call.method,
                expect,
                call.args.len()
            ),
        ));
    }

    // Content-id resolution.
    if let Some((idx, kind)) = spec.id_arg {
        let Some(Arg::Str(id)) = call.args.get(idx) else {
            return Err(compile_err(
                context,
                src,
                format!(
                    "method '{}.{}' arg {} must be a string-literal content id",
                    receiver,
                    call.method,
                    idx + 1
                ),
            ));
        };
        resolve_id(kind, id, call, registry, context, src)?;
    }

    // Plain integer-literal args (legacy typed-arg check).
    for &idx in spec.int_args {
        if !matches!(call.args.get(idx), Some(Arg::Int(_))) {
            return Err(compile_err(
                context,
                src,
                format!(
                    "method '{}.{}' arg {} must be an integer literal",
                    receiver,
                    call.method,
                    idx + 1
                ),
            ));
        }
    }

    // i8-range checks for legacy `i8` step deltas — must be an integer in range.
    for &idx in spec.i8_args {
        match call.args.get(idx) {
            Some(Arg::Int(n)) if *n >= i8::MIN as i64 && *n <= i8::MAX as i64 => {}
            Some(Arg::Int(n)) => {
                return Err(compile_err(
                    context,
                    src,
                    format!(
                        "method '{}.{}' arg {} = {} is out of range (must fit in i8: -128..=127)",
                        receiver,
                        call.method,
                        idx + 1,
                        n
                    ),
                ));
            }
            _ => {
                return Err(compile_err(
                    context,
                    src,
                    format!(
                        "method '{}.{}' arg {} must be an integer literal",
                        receiver,
                        call.method,
                        idx + 1
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn resolve_id(
    kind: IdKind,
    id: &str,
    call: &Call,
    registry: &PackRegistry,
    context: &str,
    src: &str,
) -> Result<(), ScriptError> {
    let unknown = |kind_str: &str| ScriptError::UnknownId {
        context: context.into(),
        kind: kind_str.into(),
        id: id.into(),
        source_text: src.into(),
    };
    match kind {
        IdKind::Trait => registry
            .resolve_trait(id)
            .map(|_| ())
            .map_err(|_| unknown("trait")),
        IdKind::NpcTrait => registry
            .resolve_npc_trait(id)
            .map(|_| ())
            .map_err(|_| unknown("npc_trait")),
        IdKind::Skill => registry
            .resolve_skill(id)
            .map(|_| ())
            .map_err(|_| unknown("skill")),
        IdKind::Stat => {
            if registry.is_registered_stat(id) {
                Ok(())
            } else {
                Err(unknown("stat"))
            }
        }
        IdKind::Category => {
            if registry.get_category(id).is_some() {
                Ok(())
            } else {
                Err(unknown("category"))
            }
        }
        IdKind::Arc => {
            let arc_def = registry.get_arc(id).ok_or_else(|| unknown("arc"))?;
            // arg 1 is the target state; validate it belongs to this arc.
            if let Some(Arg::Str(state)) = call.args.get(1) {
                if !arc_def.states.contains(state) {
                    return Err(ScriptError::UnknownId {
                        context: context.into(),
                        kind: "arc_state".into(),
                        id: format!("{id} -> {state}"),
                        source_text: src.into(),
                    });
                }
            }
            Ok(())
        }
    }
}

fn compile_err(context: &str, src: &str, message: String) -> ScriptError {
    ScriptError::Compile {
        context: context.into(),
        message,
        source_text: src.into(),
    }
}

// ---------------------------------------------------------------------------
// Source-scan helpers reused by the migrated static-analysis tooling
// (`Scheduler::references_game_flag`, `validate-pack` persistent-mutation lint,
// reachability). These reconstruct the legacy `Expr`/`EffectDef` walks over the
// authored Rhai source — sound because content-id/flag args are string literals.
// ---------------------------------------------------------------------------

/// True if the condition source calls `gd.hasGameFlag("<flag>")` for this flag.
/// Reconstructs the legacy `expr_references_game_flag` walk.
pub fn source_references_game_flag(src: &str, flag: &str) -> bool {
    let Ok(toks) = tokenize(src) else {
        return false;
    };
    extract_calls(&toks).iter().any(|c| {
        c.method == "hasGameFlag" && matches!(c.args.first(), Some(Arg::Str(s)) if s == flag)
    })
}

/// All `gd.setGameFlag("X")` flag args in an effect source (reachability facts).
pub fn source_set_game_flags(src: &str) -> Vec<String> {
    let Ok(toks) = tokenize(src) else {
        return Vec::new();
    };
    extract_calls(&toks)
        .iter()
        .filter(|c| c.method == "setGameFlag")
        .filter_map(|c| match c.args.first() {
            Some(Arg::Str(s)) => Some(s.clone()),
            _ => None,
        })
        .collect()
}

/// All `gd.advanceArc("ARC", "STATE")` pairs in an effect source.
pub fn source_advance_arcs(src: &str) -> Vec<(String, String)> {
    let Ok(toks) = tokenize(src) else {
        return Vec::new();
    };
    extract_calls(&toks)
        .iter()
        .filter(|c| c.method == "advanceArc")
        .filter_map(|c| match (c.args.first(), c.args.get(1)) {
            (Some(Arg::Str(a)), Some(Arg::Str(s))) => Some((a.clone(), s.clone())),
            _ => None,
        })
        .collect()
}

/// True if any `npc(...).addLiking(N)` in the effect source has `|N| > 1`
/// (an overshoot that could skip an exact npc-liking equality gate).
pub fn source_has_liking_overshoot(src: &str) -> bool {
    let Ok(toks) = tokenize(src) else {
        return false;
    };
    extract_calls(&toks).iter().any(|c| {
        c.method == "addLiking"
            && matches!(c.args.first(), Some(Arg::Int(n)) if n.unsigned_abs() > 1)
    })
}

/// True if the effect source mutates persistent world state — i.e. contains any
/// effect call OTHER than the scene-local `scene.setFlag`/`scene.removeFlag`
/// (and the `npc(ref)` constructor, which on its own mutates nothing).
/// Reconstructs `EffectDef::mutates_persistent_world` over the call-list source.
pub fn source_has_persistent_mutation(src: &str) -> bool {
    let Ok(toks) = tokenize(src) else {
        return false;
    };
    extract_calls(&toks).iter().any(|c| {
        let m = c.method.as_str();
        // The constructor and the two scene-local mutators are not persistent.
        if m == "npc"
            || (c.receiver.as_deref() == Some("scene") && (m == "setFlag" || m == "removeFlag"))
        {
            return false;
        }
        // Any other recognised write mutator is persistent.
        write_spec(c.receiver.as_deref().unwrap_or(""), m).is_some()
            || (c.receiver.as_deref() == Some("npc") && write_spec("npc", m).is_some())
    })
}

// ---------------------------------------------------------------------------
// Prose load gate (design §5.4) — validates the method surface of authored prose
// templates at load. Single-quote-aware (the tokenizer accepts both quote styles).
// Arity is validated leniently in prose (filters / arithmetic / nested calls defeat
// the comma splitter); identity (receiver.method exists, is prose-contexted) and
// string-literal content-id resolution are what's enforced.
// ---------------------------------------------------------------------------

/// Validate every `receiver.method(...)` call site in a Minijinja prose template
/// against the registry's prose surface. Surfaced through
/// `script::api::prose_validate::validate_prose`.
pub fn validate_prose(
    template: &str,
    registry: &PackRegistry,
    context: &str,
) -> Result<(), ScriptError> {
    for region in expression_regions(template) {
        let toks = tokenize(&region).map_err(|message| ScriptError::Compile {
            context: context.into(),
            message,
            source_text: region.clone(),
        })?;
        for call in extract_calls(&toks) {
            let Some(recv_tok) = call.receiver.as_deref() else {
                continue; // bare call: a Minijinja filter/function/test — out of scope (§5.4)
            };
            let Some(recv) = crate::script::api::receiver_from_token(recv_tok) else {
                continue; // not one of our receivers — leave to Minijinja
            };
            let Some(d) = crate::script::api::lookup(recv, &call.method) else {
                return Err(compile_err(
                    context,
                    &region,
                    format!("unknown prose method '{}.{}'", recv_tok, call.method),
                ));
            };
            if !d.contexts.prose {
                return Err(compile_err(
                    context,
                    &region,
                    format!(
                        "method '{}.{}' is not callable in prose",
                        recv_tok, call.method
                    ),
                ));
            }
            validate_prose_id_arg(d.args, &call, registry, context, &region)?;
        }
    }
    Ok(())
}

/// Validate a prose call's leading string-literal content-id arg, if its shape
/// carries one. Non-literal ids are left to render-time (lenient, like arity).
fn validate_prose_id_arg(
    shape: crate::script::api::ArgShape,
    call: &Call,
    registry: &PackRegistry,
    context: &str,
    src: &str,
) -> Result<(), ScriptError> {
    use crate::script::api::ArgShape;
    let kind = match shape {
        ArgShape::Id(k) | ArgShape::IdInt(k) => k,
        _ => return Ok(()),
    };
    if let Some(Arg::Str(id)) = call.args.first() {
        resolve_id(kind, id, call, registry, context, src)?;
    }
    Ok(())
}

/// Extract the contents of each `{{ … }}` / `{% … %}` region, skipping `{# … #}`
/// comments and `{% raw %}…{% endraw %}` blocks and stripping whitespace-control
/// markers (`{%-`, `-%}`, …). The live corpus uses none of the exotic forms, but
/// the scan must not false-positive on them (design §5.4).
fn expression_regions(template: &str) -> Vec<String> {
    let b = template.as_bytes();
    let n = b.len();
    let mut regions = Vec::new();
    let mut i = 0;
    let mut in_raw = false;
    while i + 1 < n {
        if b[i] == b'{' && matches!(b[i + 1], b'{' | b'%' | b'#') {
            let kind = b[i + 1];
            let (ca, cb) = match kind {
                b'{' => (b'}', b'}'),
                b'%' => (b'%', b'}'),
                _ => (b'#', b'}'),
            };
            let start = i + 2;
            let mut j = start;
            while j + 1 < n && !(b[j] == ca && b[j + 1] == cb) {
                j += 1;
            }
            let end = if j + 1 < n { j } else { n };
            let content = template[start..end]
                .trim()
                .trim_start_matches('-')
                .trim_end_matches('-')
                .trim()
                .to_string();
            i = (end + 2).min(n);
            match kind {
                b'#' => {} // comment — skip
                b'%' => {
                    let head = content.split_whitespace().next().unwrap_or("");
                    if head == "raw" {
                        in_raw = true;
                    } else if head == "endraw" {
                        in_raw = false;
                    } else if !in_raw {
                        regions.push(content);
                    }
                }
                _ => {
                    if !in_raw {
                        regions.push(content);
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    regions
}

#[cfg(test)]
mod prose_gate_tests {
    use super::*;

    fn registry_with_skills(ids: &[&str]) -> PackRegistry {
        let mut r = PackRegistry::new();
        r.register_skills(
            ids.iter()
                .map(|id| undone_packs::SkillDef {
                    id: (*id).into(),
                    name: (*id).into(),
                    description: String::new(),
                    min: 0,
                    max: 100,
                })
                .collect(),
        );
        r
    }

    #[test]
    fn prose_gate_accepts_single_quoted_id() {
        let r = registry_with_skills(&["FEMININITY"]);
        // single quotes are legal in minijinja and used in the live pack
        assert!(validate_prose(
            r#"{% if w.getSkill('FEMININITY') < 20 %}x{% endif %}"#,
            &r,
            "test"
        )
        .is_ok());
    }

    #[test]
    fn prose_gate_accepts_double_quoted_id() {
        let r = registry_with_skills(&["FEMININITY"]);
        assert!(validate_prose(r#"{{ w.getSkill("FEMININITY") }}"#, &r, "test").is_ok());
    }

    #[test]
    fn prose_gate_rejects_unknown_method() {
        let r = registry_with_skills(&["FEMININITY"]);
        assert!(validate_prose(r#"{{ w.notAReal() }}"#, &r, "test").is_err());
    }

    #[test]
    fn prose_gate_rejects_write_in_prose() {
        let r = registry_with_skills(&["FEMININITY"]);
        assert!(validate_prose(r#"{{ w.changeMoney(5) }}"#, &r, "test").is_err());
    }

    #[test]
    fn prose_gate_rejects_checkskill_in_prose() {
        let r = registry_with_skills(&["CHARM"]);
        // checkSkill is condition-only (RNG side effect) — barred from prose.
        assert!(validate_prose(
            r#"{% if w.checkSkill('CHARM', 10) %}x{% endif %}"#,
            &r,
            "test"
        )
        .is_err());
    }

    #[test]
    fn prose_gate_rejects_unknown_content_id() {
        let r = registry_with_skills(&["FEMININITY"]);
        // CHARM not registered → id resolution fails.
        assert!(validate_prose(r#"{{ w.getSkill('CHARM') }}"#, &r, "test").is_err());
    }

    #[test]
    fn prose_gate_ignores_filters_and_plain_text() {
        let r = registry_with_skills(&["FEMININITY"]);
        // bare filters/functions and plain prose are not method-surface calls.
        assert!(validate_prose("Just some plain prose with no calls.", &r, "test").is_ok());
        assert!(validate_prose(r#"{{ "x" | upper }}"#, &r, "test").is_ok());
        // comments are skipped, not parsed.
        assert!(validate_prose(r#"{# w.notAReal() #}plain"#, &r, "test").is_ok());
    }
}

// ---------------------------------------------------------------------------
// Negative-surface tests — the per-receiver / per-context sets the registry
// keys must NOT union (design §2, §9). These must STAY unknown methods.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod surface_tests {
    use super::*;

    #[test]
    fn f_has_trait_is_unknown_method() {
        // `f` deliberately lacks hasTrait (only `m` has it).
        assert!(read_spec("f", "hasTrait").is_none());
    }

    #[test]
    fn m_is_pregnant_is_unknown_method() {
        // isPregnant is on `f`/`role`, never `m`.
        assert!(read_spec("m", "isPregnant").is_none());
    }

    #[test]
    fn m_is_virgin_is_unknown_method() {
        assert!(read_spec("m", "isVirgin").is_none());
    }

    #[test]
    fn f_had_orgasm_is_unknown_method() {
        assert!(read_spec("f", "hadOrgasm").is_none());
    }

    #[test]
    fn write_in_condition_is_unknown() {
        // A write mutator is not a read method (read_spec gates on contexts.condition).
        assert!(read_spec("w", "changeMoney").is_none());
        assert!(read_spec("gd", "setGameFlag").is_none());
    }

    #[test]
    fn read_in_effect_still_resolves() {
        // Reads remain valid in effect call-lists (effects branch on reads).
        assert!(write_spec("w", "isVirgin").is_none()); // not a write…
        assert!(read_spec("w", "isVirgin").is_some()); // …but a read.
    }

    #[test]
    fn known_methods_resolve_with_expected_arity() {
        // Spot-check the argshape→spec derivation against the legacy specs.
        assert_eq!(read_spec("w", "hasTrait").unwrap().arity, 1);
        assert_eq!(read_spec("gd", "npcLikingAtLeast").unwrap().arity, 2);
        assert_eq!(read_spec("role", "getName").unwrap().arity, 1);
        assert_eq!(write_spec("gd", "advanceArc").unwrap().arity, 2);
        let sv = write_spec("w", "setVirgin").unwrap();
        assert_eq!((sv.arity, sv.arity_max), (1, 2));
    }
}
