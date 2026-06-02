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

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum IdKind {
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

/// Look up the READ method surface (valid in conditions AND effects).
/// `receiver` is the handle token (`w`/`gd`/`m`/`f`/`role`/`scene`).
fn read_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    match (receiver, method) {
        // ── w (player) ───────────────────────────────────────────────────────
        (
            "w",
            "isVirgin"
            | "isAnalVirgin"
            | "isDrunk"
            | "isVeryDrunk"
            | "isMaxDrunk"
            | "isSingle"
            | "isOnPill"
            | "isPregnant"
            | "alwaysFemale"
            | "wasMale"
            | "wasTransformed"
            | "hasSmoothLegs"
            | "getMoney"
            | "getStress"
            | "getAnxiety"
            | "pcOrigin"
            | "beforeName"
            | "beforeRace"
            | "beforeAge"
            | "beforeSexuality"
            | "getName"
            | "getRace"
            | "getAge"
            | "getArousal"
            | "getAlcohol"
            | "getHeight"
            | "getFigure"
            | "getBreasts"
            | "getButt"
            | "getWaist"
            | "getLips"
            | "getHairColour"
            | "getHairLength"
            | "getEyeColour"
            | "getSkinTone"
            | "getComplexion"
            | "getAppearance"
            | "getNippleSensitivity"
            | "getClitSensitivity"
            | "getPubicHair"
            | "getNaturalPubicHair"
            | "getInnerLabia"
            | "getWetness"
            | "beforeVoice"
            | "beforeHeight"
            | "beforeHairColour"
            | "beforeEyeColour"
            | "beforeSkinTone"
            | "beforePenisSize"
            | "beforeFigure",
        ) => Some(spec(0)),
        ("w", "hasTrait" | "hadTraitBefore") => Some(spec_id(1, 0, IdKind::Trait)),
        ("w", "inCategory" | "beforeInCategory") => Some(spec_id(1, 0, IdKind::Category)),
        ("w", "getSkill") => Some(spec_id(1, 0, IdKind::Skill)),
        ("w", "composure") => Some(spec(0)),
        // hasStuff id is intentionally NOT registry-validated (legacy: missing = can't have it).
        ("w", "hasStuff") => Some(spec(1)),
        ("w", "checkSkill" | "checkSkillRed") => Some(spec_id_int(2, 0, IdKind::Skill, 1)),

        // ── m (active male) ───────────────────────────────────────────────────
        (
            "m",
            "isPartner"
            | "isFriend"
            | "isCohabiting"
            | "isContactable"
            | "hadOrgasm"
            | "isNpcAttractionOk"
            | "isNpcAttractionLust"
            | "isWAttractionOk"
            | "isNpcLoveCrush"
            | "isNpcLoveSome"
            | "isWLoveCrush"
            | "getLiking"
            | "getLove"
            | "getAttraction"
            | "getBehaviour",
        ) => Some(spec(0)),
        ("m", "hasTrait") => Some(spec_id(1, 0, IdKind::NpcTrait)),
        ("m", "hasFlag" | "hasRole") => Some(spec(1)),

        // ── f (active female) ─────────────────────────────────────────────────
        (
            "f",
            "isPartner" | "isFriend" | "isPregnant" | "isVirgin" | "getLiking" | "getLove"
            | "getAttraction" | "getBehaviour",
        ) => Some(spec(0)),
        ("f", "hasFlag" | "hasRole") => Some(spec(1)),

        // ── scene ─────────────────────────────────────────────────────────────
        ("scene", "hasFlag") => Some(spec(1)),

        // ── gd (game data) ────────────────────────────────────────────────────
        (
            "gd",
            "isWeekday" | "isWeekend" | "week" | "day" | "desire" | "timeSlot" | "getJobTitle",
        ) => Some(spec(0)),
        // getStat / hasGameFlag / arcStarted / arcState / npcLiking ids are not
        // registry-validated in the legacy condition pass.
        ("gd", "hasGameFlag" | "arcStarted" | "getStat" | "arcState" | "npcLiking") => {
            Some(spec(1))
        }
        ("gd", "npcLikingAtLeast") => Some(spec(2)),

        // ── role ──────────────────────────────────────────────────────────────
        (
            "role",
            "isPartner" | "isFriend" | "isContactable" | "isPregnant" | "isVirgin" | "hadOrgasm"
            | "getName" | "getLiking" | "getLove" | "getAttraction" | "getBehaviour",
        ) => Some(spec(1)),
        ("role", "hasFlag" | "hasRole") => Some(spec(2)),

        _ => None,
    }
}

/// Look up the WRITE (effect-mutator) surface. `receiver` is `w`/`gd`/`scene`, or
/// `npc` for a `npc("m"|"f"|role).method(...)` chained call, or the bare `npc(ref)`
/// free call (receiver `npc`, method `npc`).
fn write_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    match (receiver, method) {
        // ── w (player) ───────────────────────────────────────────────────────
        ("w", "changeStress" | "changeMoney" | "changeAnxiety" | "changeComposure") => {
            Some(spec(1))
        }
        ("w", "addArousal" | "changeAlcohol") => Some(spec_i8(1, &[0])),
        ("w", "skillIncrease") => Some(spec_id(2, 0, IdKind::Skill)),
        ("w", "addTrait" | "removeTrait") => Some(spec_id(1, 0, IdKind::Trait)),
        // stuff is not registry-validated at load (legacy validate_effects skips it).
        ("w", "addStuff" | "removeStuff") => Some(spec(1)),
        // setVirgin(value) or setVirgin(value, "type") — overloaded arity.
        ("w", "setVirgin") => Some(spec_arity(1, 2)),
        ("w", "setPartner" | "addFriend") => Some(spec(1)),

        // ── gd (game data) ────────────────────────────────────────────────────
        ("gd", "setGameFlag" | "removeGameFlag") => Some(spec(1)),
        ("gd", "addStat" | "setStat") => Some(spec_id(2, 0, IdKind::Stat)),
        ("gd", "setJobTitle") => Some(spec(1)),
        ("gd", "addDesire" | "setDesire") => Some(spec(1)),
        ("gd", "advanceTime") => Some(spec(1)),
        ("gd", "advanceArc") => Some(spec_id(2, 0, IdKind::Arc)),
        ("gd", "failRedCheck") => Some(spec_id(1, 0, IdKind::Skill)),

        // ── scene ─────────────────────────────────────────────────────────────
        ("scene", "setFlag" | "removeFlag") => Some(spec(1)),

        // ── npc(ref).* ────────────────────────────────────────────────────────
        ("npc", "addLiking" | "addLove" | "addWLiking" | "setAttraction") => Some(spec_i8(1, &[0])),
        (
            "npc",
            "setFlag" | "setRelationship" | "setBehaviour" | "addSexualActivity" | "setRole"
            | "setName",
        ) => Some(spec(1)),
        ("npc", "addTrait") => Some(spec_id(1, 0, IdKind::NpcTrait)),
        ("npc", "setContactable") => Some(spec(1)),
        // the free `npc(ref)` constructor.
        ("npc", "npc") => Some(spec(1)),

        _ => None,
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
