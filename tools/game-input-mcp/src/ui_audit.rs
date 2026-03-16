use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSummary {
    pub tab: String,
    pub current_scene_id: Option<String>,
    pub awaiting_continue: bool,
    pub visible_action_ids: Vec<String>,
    pub story_paragraphs: Vec<String>,
}

pub fn summarize_runtime(value: &Value) -> Result<RuntimeSummary> {
    let data = value
        .get("data")
        .ok_or_else(|| anyhow!("runtime response missing data payload"))?;
    let tab = data
        .get("tab")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("runtime response missing tab"))?;
    let awaiting_continue = data
        .get("awaiting_continue")
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("runtime response missing awaiting_continue"))?;
    let current_scene_id = data
        .get("current_scene_id")
        .and_then(|scene| scene.as_str().map(ToOwned::to_owned));
    let visible_action_ids = data
        .get("visible_actions")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("runtime response missing visible_actions"))?
        .iter()
        .filter_map(|action| {
            action
                .get("id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .collect();
    let story_paragraphs = data
        .get("story_paragraphs")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("runtime response missing story_paragraphs"))?
        .iter()
        .filter_map(|paragraph| paragraph.as_str().map(ToOwned::to_owned))
        .collect();

    Ok(RuntimeSummary {
        tab: tab.to_string(),
        current_scene_id,
        awaiting_continue,
        visible_action_ids,
        story_paragraphs,
    })
}

pub fn assert_no_runtime_change(
    _surface: &str,
    _before: &RuntimeSummary,
    _after: &RuntimeSummary,
) -> Result<()> {
    if _before == _after {
        Ok(())
    } else {
        Err(anyhow!(
            "{} changed visible runtime state:\nbefore: {:?}\nafter: {:?}",
            _surface,
            _before,
            _after
        ))
    }
}

pub fn assert_tab(expected_tab: &str, summary: &RuntimeSummary) -> Result<()> {
    if summary.tab == expected_tab {
        Ok(())
    } else {
        Err(anyhow!(
            "expected tab '{expected_tab}', got '{}'",
            summary.tab
        ))
    }
}

pub fn assert_runtime_change(
    surface: &str,
    before: &RuntimeSummary,
    after: &RuntimeSummary,
) -> Result<()> {
    if before != after {
        Ok(())
    } else {
        Err(anyhow!(
            "{surface} did not change visible runtime state: {:?}",
            after
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        assert_no_runtime_change, assert_runtime_change, assert_tab, summarize_runtime,
        RuntimeSummary,
    };
    use serde_json::json;

    fn runtime_response(tab: &str) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Runtime state captured",
            "data": {
                "tab": tab,
                "current_scene_id": "base::workplace_arrival",
                "awaiting_continue": false,
                "visible_actions": [
                    { "id": "wait", "label": "Wait", "detail": "Hold steady." }
                ],
                "story_paragraphs": ["Paragraph 1", "Paragraph 2"]
            }
        })
    }

    #[test]
    fn summarize_runtime_extracts_only_stable_ui_fields() {
        let summary = summarize_runtime(&runtime_response("game")).unwrap();

        assert_eq!(
            summary,
            RuntimeSummary {
                tab: "game".to_string(),
                current_scene_id: Some("base::workplace_arrival".to_string()),
                awaiting_continue: false,
                visible_action_ids: vec!["wait".to_string()],
                story_paragraphs: vec!["Paragraph 1".to_string(), "Paragraph 2".to_string()],
            }
        );
    }

    #[test]
    fn dead_space_assertion_fails_when_runtime_changes() {
        let before = summarize_runtime(&runtime_response("game")).unwrap();
        let after = summarize_runtime(&runtime_response("saves")).unwrap();

        let error = assert_no_runtime_change("title bar dead space", &before, &after).unwrap_err();

        assert!(error.to_string().contains("title bar dead space"));
    }

    #[test]
    fn tab_assertion_requires_expected_visible_tab() {
        let summary = summarize_runtime(&runtime_response("game")).unwrap();

        assert!(assert_tab("game", &summary).is_ok());
        assert!(assert_tab("saves", &summary).is_err());
    }

    #[test]
    fn visible_control_assertion_requires_runtime_change() {
        let before = summarize_runtime(&runtime_response("game")).unwrap();
        let after = summarize_runtime(&runtime_response("saves")).unwrap();

        assert!(assert_runtime_change("visible tab click", &before, &after).is_ok());
        assert!(assert_runtime_change("visible tab click", &before, &before).is_err());
    }
}
