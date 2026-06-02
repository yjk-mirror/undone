use minijinja::{Environment, Error as MiniJinjaError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateError {
    pub line: Option<u32>,
    pub message: String,
}

/// Parse a template string and return syntax errors.
pub fn validate_template(source: &str) -> Vec<TemplateError> {
    let mut env = Environment::new();
    match env.add_template("__check__", source) {
        Ok(_) => vec![],
        Err(e) => vec![to_template_error(&e)],
    }
}

/// Render a template with a JSON context. Returns rendered string or error.
pub fn render_template(source: &str, context_json: &str) -> Result<String, TemplateError> {
    let ctx: serde_json::Value = serde_json::from_str(context_json).map_err(|e| TemplateError {
        line: None,
        message: format!("Invalid JSON context: {}", e),
    })?;
    let mut env = Environment::new();
    env.add_template("__render__", source)
        .map_err(|e| to_template_error(&e))?;
    let tmpl = env
        .get_template("__render__")
        .map_err(|e| to_template_error(&e))?;
    tmpl.render(ctx).map_err(|e| to_template_error(&e))
}

/// List all built-in Minijinja filters available with default features.
///
/// Verified against minijinja 2.16.0 `src/defaults.rs` `get_builtin_filters()`.
/// Excludes `tojson` (requires `json` feature) and `urlencode` (requires `urlencode` feature).
pub fn list_builtin_filters() -> Vec<&'static str> {
    vec![
        // Always available
        "safe",
        "escape",
        "e",
        // builtins feature (enabled by default)
        "abs",
        "attr",
        "batch",
        "bool",
        "capitalize",
        "chain",
        "count",
        "d",
        "default",
        "dictsort",
        "first",
        "float",
        "format",
        "groupby",
        "indent",
        "int",
        "items",
        "join",
        "last",
        "length",
        "lines",
        "list",
        "lower",
        "map",
        "max",
        "min",
        "pprint",
        "reject",
        "rejectattr",
        "replace",
        "reverse",
        "round",
        "select",
        "selectattr",
        "slice",
        "sort",
        "split",
        "string",
        "sum",
        "title",
        "trim",
        "unique",
        "upper",
        "zip",
    ]
}

fn to_template_error(e: &MiniJinjaError) -> TemplateError {
    TemplateError {
        line: e.line().map(|l| l as u32),
        message: e.to_string(),
    }
}

/// Validate a prose template against the game's method surface (the same load-time
/// prose gate the loader runs). Checks every `receiver.method(...)` call-site is a
/// known, prose-callable method with resolvable string-literal content ids. Runs the
/// minijinja syntax check first; only reports method-surface errors when syntax is OK.
pub fn validate_prose(source: &str, registry: &undone_packs::PackRegistry) -> Vec<TemplateError> {
    let syntax = validate_template(source);
    if !syntax.is_empty() {
        return syntax;
    }
    match undone_scene::script::api::prose_validate::validate_prose(source, registry, "mcp") {
        Ok(()) => vec![],
        Err(e) => vec![TemplateError {
            line: None,
            message: e.to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_template_returns_no_errors() {
        let errs = validate_template("Hello {{ name }}!");
        assert!(errs.is_empty(), "got: {:?}", errs);
    }

    #[test]
    fn unclosed_tag_returns_error() {
        let errs = validate_template("Hello {{ name !");
        assert!(!errs.is_empty(), "expected an error for malformed template");
    }

    #[test]
    fn render_with_context_produces_correct_output() {
        let result = render_template("Hello {{ name }}!", r#"{"name": "World"}"#);
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn list_filters_includes_common_filters() {
        let filters = list_builtin_filters();
        assert!(filters.contains(&"upper"));
        assert!(filters.contains(&"lower"));
        assert!(filters.contains(&"join"));
        // Filters present in Jinja2 (Python) but NOT in minijinja 2.x defaults
        assert!(
            !filters.contains(&"filesizeformat"),
            "filesizeformat is not in minijinja 2"
        );
        assert!(
            !filters.contains(&"striptags"),
            "striptags is not in minijinja 2"
        );
        assert!(
            !filters.contains(&"wordwrap"),
            "wordwrap is not in minijinja 2"
        );
    }

    #[test]
    fn render_with_invalid_json_context_returns_error() {
        let result = render_template("Hello {{ name }}!", "not valid json");
        assert!(result.is_err(), "expected error for invalid JSON context");
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON context"));
    }

    fn registry_with_femininity() -> undone_packs::PackRegistry {
        let mut r = undone_packs::PackRegistry::new();
        r.register_skills(vec![undone_packs::SkillDef {
            id: "FEMININITY".into(),
            name: "Femininity".into(),
            description: String::new(),
            min: 0,
            max: 100,
        }]);
        r
    }

    #[test]
    fn prose_validate_flags_unknown_method() {
        let r = registry_with_femininity();
        assert!(
            !validate_prose("{{ w.notAReal() }}", &r).is_empty(),
            "unknown prose method should be reported"
        );
    }

    #[test]
    fn prose_validate_accepts_valid_single_quoted_id() {
        let r = registry_with_femininity();
        assert!(
            validate_prose("{{ w.getSkill('FEMININITY') }}", &r).is_empty(),
            "valid prose method + registered id should pass"
        );
    }

    #[test]
    fn prose_validate_flags_write_in_prose() {
        let r = registry_with_femininity();
        assert!(
            !validate_prose("{{ w.changeMoney(5) }}", &r).is_empty(),
            "write mutator should be rejected in prose"
        );
    }

    #[test]
    fn prose_validate_reports_syntax_errors() {
        let r = registry_with_femininity();
        assert!(
            !validate_prose("{{ w.getSkill('FEMININITY'", &r).is_empty(),
            "malformed template should report syntax error"
        );
    }
}
