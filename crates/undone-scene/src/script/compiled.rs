//! Compiled-script type + the load-time error taxonomy + the compile/validate gate.

use std::sync::Arc;

use thiserror::Error;
use undone_packs::PackRegistry;

use crate::script::engine::read_scope;
use crate::script::validate;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script compile error in {context}: {message}\n  source: {source_text}")]
    Compile {
        context: String,
        message: String,
        source_text: String,
    },
    #[error("unknown content id '{id}' ({kind}) in {context}\n  source: {source_text}")]
    UnknownId {
        context: String,
        kind: String,
        id: String,
        source_text: String,
    },
    #[error("script runtime error in {context}: {message}")]
    Runtime { context: String, message: String },
}

/// A compiled condition or effect script. The AST is the direct analog of the
/// pre-parsed `undone_expr::Expr` it replaces: compiled once at pack load,
/// evaluated many times at runtime.
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub ast: Arc<rhai::AST>,
    pub source: String,
}

/// Compile + validate a condition at pack load. The two-layer gate:
/// 1. `compile_with_scope` (the handle vars in scope) → syntax + unknown-variable
///    errors via `strict_variables`.
/// 2. static source scan → unknown method / arity / unknown content id / range,
///    across ALL branches (the legacy `validate_condition_ids` guarantee). Only
///    READ methods are valid; an effect mutator in a condition fails here.
pub fn compile_condition(
    src: &str,
    engine: &rhai::Engine,
    registry: &PackRegistry,
    context: &str,
) -> Result<CompiledScript, ScriptError> {
    let ast = engine
        .compile_with_scope(&read_scope(), src)
        .map_err(|e| ScriptError::Compile {
            context: context.into(),
            message: e.to_string(),
            source_text: src.into(),
        })?;
    validate::validate_condition_source(src, registry, context)?;
    Ok(CompiledScript {
        ast: Arc::new(ast),
        source: src.into(),
    })
}

/// Compile + validate an effect call-list at pack load. Same two layers as
/// [`compile_condition`], but READ ∪ WRITE methods are valid (it compiles against
/// the effect engine).
pub fn compile_effect(
    src: &str,
    engine: &rhai::Engine,
    registry: &PackRegistry,
    context: &str,
) -> Result<CompiledScript, ScriptError> {
    let ast = engine
        .compile_with_scope(&read_scope(), src)
        .map_err(|e| ScriptError::Compile {
            context: context.into(),
            message: e.to_string(),
            source_text: src.into(),
        })?;
    validate::validate_effect_source(src, registry, context)?;
    Ok(CompiledScript {
        ast: Arc::new(ast),
        source: src.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::engine::build_engines;

    fn base_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        reg.register_skills(vec![undone_packs::SkillDef {
            id: "FEMININITY".into(),
            name: "Femininity".into(),
            description: "...".into(),
            min: 0,
            max: 100,
        }]);
        reg
    }

    #[test]
    fn typo_trait_id_fails_at_compile_not_runtime() {
        let engines = build_engines();
        let reg = base_registry();
        let err = compile_condition(
            r#"w.hasTrait("TYPpO_NOT_A_TRAIT")"#,
            &engines.cond,
            &reg,
            "test",
        )
        .unwrap_err();
        assert!(
            matches!(err, ScriptError::UnknownId { .. }),
            "unknown trait id must fail at LOAD, got: {err:?}"
        );
    }

    #[test]
    fn valid_condition_compiles() {
        let engines = build_engines();
        let reg = base_registry();
        assert!(compile_condition(r#"w.hasTrait("SHY")"#, &engines.cond, &reg, "test").is_ok());
        assert!(compile_condition(
            r#"w.hasTrait("SHY") && w.getSkill("FEMININITY") < 15"#,
            &engines.cond,
            &reg,
            "test"
        )
        .is_ok());
    }

    #[test]
    fn typo_skill_in_get_skill_fails_at_load() {
        let engines = build_engines();
        let reg = base_registry();
        let err = compile_condition(r#"w.getSkill("NOPE") > 5"#, &engines.cond, &reg, "test")
            .unwrap_err();
        assert!(matches!(err, ScriptError::UnknownId { kind, .. } if kind == "skill"));
    }

    #[test]
    fn typo_id_in_short_circuited_branch_still_fails() {
        // The branch never executes (false && ...), but the static scan still
        // catches the typo — the all-branch guarantee a runtime dry-run lacks.
        let engines = build_engines();
        let reg = base_registry();
        let err = compile_condition(
            r#"false && w.hasTrait("TYPO")"#,
            &engines.cond,
            &reg,
            "test",
        )
        .unwrap_err();
        assert!(matches!(err, ScriptError::UnknownId { .. }));
    }

    #[test]
    fn unknown_method_fails_at_load() {
        let engines = build_engines();
        let reg = base_registry();
        let err =
            compile_condition(r#"w.notARealMethod()"#, &engines.cond, &reg, "test").unwrap_err();
        assert!(matches!(err, ScriptError::Compile { .. }));
    }

    #[test]
    fn effect_mutator_in_condition_is_rejected() {
        // Read/write split enforced at LOAD: an effect call in a condition is
        // unknown on the condition surface.
        let engines = build_engines();
        let reg = base_registry();
        let err = compile_condition(r#"w.addArousal(1)"#, &engines.cond, &reg, "test").unwrap_err();
        assert!(matches!(err, ScriptError::Compile { .. }));
    }

    #[test]
    fn valid_effect_compiles() {
        let engines = build_engines();
        let reg = base_registry();
        assert!(compile_effect(
            r#"w.addArousal(1); gd.setGameFlag("X"); npc("m").addLiking(2);"#,
            &engines.effect,
            &reg,
            "test"
        )
        .is_ok());
    }

    #[test]
    fn out_of_range_delta_fails_at_load() {
        // Legacy EffectDef stored i8; an out-of-range delta wrapped silently with
        // the i64 Rhai arg, so the gate rejects it (review finding #2).
        let engines = build_engines();
        let reg = base_registry();
        let err =
            compile_effect(r#"w.addArousal(200)"#, &engines.effect, &reg, "test").unwrap_err();
        assert!(
            matches!(err, ScriptError::Compile { ref message, .. } if message.contains("i8")),
            "got: {err:?}"
        );
    }

    #[test]
    fn effect_unknown_skill_fails_at_load() {
        let engines = build_engines();
        let reg = base_registry();
        let err = compile_effect(
            r#"w.skillIncrease("NOPE", 5)"#,
            &engines.effect,
            &reg,
            "test",
        )
        .unwrap_err();
        assert!(matches!(err, ScriptError::UnknownId { kind, .. } if kind == "skill"));
    }
}
