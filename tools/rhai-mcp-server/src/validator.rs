//! Pure validation logic — testable without the MCP layer.

use rhai::{Engine, EvalAltResult, ParseError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: Option<u16>,
    pub column: Option<u16>,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Check syntax only — fast, no evaluation.
pub fn check_syntax(source: &str) -> Vec<Diagnostic> {
    let engine = Engine::new();
    match engine.compile(source) {
        Ok(_) => vec![],
        Err(e) => vec![parse_error_to_diagnostic(&e)],
    }
}

/// Validate by running the script against a minimal engine instance.
pub fn validate_with_engine(source: &str, engine: &Engine) -> Vec<Diagnostic> {
    match engine.compile(source) {
        Err(e) => vec![parse_error_to_diagnostic(&e)],
        Ok(ast) => {
            let mut scope = rhai::Scope::new();
            match engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast) {
                Ok(_) => vec![],
                Err(e) => vec![eval_error_to_diagnostic(&e)],
            }
        }
    }
}

/// List all functions registered in the given engine.
pub fn list_registered_functions(engine: &Engine) -> Vec<String> {
    engine.gen_fn_signatures(true)
}

/// Read a file and validate its syntax.
pub fn validate_file(path: &Path) -> Vec<Diagnostic> {
    match std::fs::read_to_string(path) {
        Err(e) => vec![Diagnostic {
            line: None,
            column: None,
            message: format!("Cannot read file: {}", e),
            severity: DiagnosticSeverity::Error,
        }],
        Ok(source) => check_syntax(&source),
    }
}

// ---------------------------------------------------------------------------
// Game-engine validation — matches the loader's fail-fast gate exactly.
//
// These run the authored source through the SAME `compile_condition` /
// `compile_effect` gate the game loader uses (syntax + strict-variables + the
// source-scan that resolves content ids and rejects unknown methods / effect
// mutators in conditions), against a probe registry loaded from the pack.
// ---------------------------------------------------------------------------

/// Validate a condition string against the real condition engine + registry.
pub fn validate_game_condition(
    source: &str,
    registry: &undone_packs::PackRegistry,
) -> Vec<Diagnostic> {
    match undone_scene::compile_condition(source, registry, "mcp") {
        Ok(_) => vec![],
        Err(e) => vec![script_error_to_diagnostic(&e)],
    }
}

/// Validate an effect call-list string against the real effect engine + registry.
pub fn validate_game_effect(
    source: &str,
    registry: &undone_packs::PackRegistry,
) -> Vec<Diagnostic> {
    match undone_scene::compile_effect(source, registry, "mcp") {
        Ok(_) => vec![],
        Err(e) => vec![script_error_to_diagnostic(&e)],
    }
}

fn script_error_to_diagnostic(e: &undone_scene::script::ScriptError) -> Diagnostic {
    Diagnostic {
        line: None,
        column: None,
        message: e.to_string(),
        severity: DiagnosticSeverity::Error,
    }
}

fn parse_error_to_diagnostic(e: &ParseError) -> Diagnostic {
    let pos = e.1;
    Diagnostic {
        line: pos.line().map(|l| l as u16),
        column: pos.position().map(|c| c as u16),
        message: e.0.to_string(),
        severity: DiagnosticSeverity::Error,
    }
}

fn eval_error_to_diagnostic(e: &EvalAltResult) -> Diagnostic {
    let pos = e.position();
    Diagnostic {
        line: pos.line().map(|l| l as u16),
        column: pos.position().map(|c| c as u16),
        message: e.to_string(),
        severity: DiagnosticSeverity::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn valid_script_returns_no_diagnostics() {
        let diags = check_syntax("let x = 1 + 2; x");
        assert!(
            diags.is_empty(),
            "expected no diagnostics, got: {:?}",
            diags
        );
    }

    #[test]
    fn syntax_error_returns_diagnostic_with_location() {
        let diags = check_syntax("let x = "); // incomplete expression
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        assert!(matches!(diags[0].severity, DiagnosticSeverity::Error));
        assert!(
            diags[0].line.is_some(),
            "expected a line number in the diagnostic"
        );
    }

    #[test]
    fn runtime_error_is_caught_by_engine_validation() {
        let engine = Engine::new();
        // Type mismatch: syntax is valid but runtime evaluation fails
        let diags = validate_with_engine("let x: i64 = \"not a number\";", &engine);
        assert!(
            !diags.is_empty(),
            "expected runtime diagnostic for type mismatch"
        );
        assert!(matches!(diags[0].severity, DiagnosticSeverity::Error));
    }

    #[test]
    fn list_functions_returns_builtins() {
        let engine = Engine::new();
        let fns = list_registered_functions(&engine);
        assert!(!fns.is_empty(), "engine should have built-in functions");
    }

    fn registry_with_shy() -> undone_packs::PackRegistry {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        reg
    }

    #[test]
    fn game_condition_rejects_unknown_trait_id() {
        let reg = registry_with_shy();
        let diags = validate_game_condition(r#"w.hasTrait("NOPE")"#, &reg);
        assert!(!diags.is_empty(), "unknown trait id must be rejected");
        assert!(diags[0].message.contains("NOPE"));
    }

    #[test]
    fn game_condition_accepts_known_trait_id() {
        let reg = registry_with_shy();
        let diags = validate_game_condition(r#"w.hasTrait("SHY")"#, &reg);
        assert!(diags.is_empty(), "known trait should validate: {diags:?}");
    }

    #[test]
    fn game_condition_rejects_effect_mutator() {
        // w.addArousal is a WRITE mutator — not allowed in a condition.
        let reg = registry_with_shy();
        let diags = validate_game_condition(r#"w.addArousal(1)"#, &reg);
        assert!(
            !diags.is_empty(),
            "effect mutator in a condition must be rejected"
        );
    }

    #[test]
    fn game_effect_accepts_known_mutator() {
        let reg = registry_with_shy();
        let diags = validate_game_effect(r#"w.addArousal(1); gd.setGameFlag("X");"#, &reg);
        assert!(diags.is_empty(), "valid effect should validate: {diags:?}");
    }

    #[test]
    fn validate_file_returns_error_for_missing_file() {
        let diags = validate_file(Path::new("/nonexistent/file.rhai"));
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("Cannot read file"));
    }

    #[test]
    fn validate_file_returns_empty_for_valid_script() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "let x = 42;").unwrap();
        let diags = validate_file(f.path());
        assert!(diags.is_empty(), "got: {:?}", diags);
    }
}
