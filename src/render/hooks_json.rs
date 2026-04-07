//! Render a collection of [`Hook`]s into a Claude-Code-compatible
//! `hooks.json` file.
//!
//! The format is the JSON superset: `{ "hooks": [ { event, matcher,
//! command, timeout_ms }, ... ] }`. Runtimes with the same shape (Gemini,
//! Codex, OpenCode, Amp) reuse this directly. Runtimes with divergent
//! shapes use their own serializer.
//!
//! Matchers are pipe-delimited tool names (e.g. `"Edit|Write"`). Each
//! half is translated through the registry so Gemini's `hooks.json` sees
//! `edit_file|write_file`, etc.

use crate::{Error, Hook, Result, RuntimeId, ToolRegistry};

/// Render `hooks` as the complete contents of a `hooks.json` file.
///
/// Matcher translation: the pipe-delimited string is split on `|`, each
/// half is trimmed and passed through
/// [`ToolRegistry::translate_tool_name`], and the result is rejoined
/// with `|`. Empty matchers produce no `matcher` field.
pub fn render_hooks_json(hooks: &[Hook], runtime: RuntimeId, registry: &ToolRegistry) -> Result<String> {
    let mut entries = Vec::new();
    for hook in hooks {
        let mut entry = serde_json::Map::new();
        entry.insert("event".into(), serde_json::Value::String(hook.event.clone()));

        if let Some(matcher) = hook.matcher.as_deref()
            && !matcher.is_empty()
        {
            let translated: String = matcher
                .split('|')
                .map(|m| registry.translate_tool_name(m.trim(), runtime))
                .collect::<Vec<_>>()
                .join("|");
            entry.insert("matcher".into(), serde_json::Value::String(translated));
        }

        entry.insert("command".into(), serde_json::Value::String(hook.command.clone()));

        if hook.timeout_ms > 0 {
            entry.insert("timeout_ms".into(), serde_json::json!(hook.timeout_ms));
        }

        entries.push(serde_json::Value::Object(entry));
    }

    let root = serde_json::json!({ "hooks": entries });
    serde_json::to_string_pretty(&root).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hook() -> Hook {
        Hook {
            name: "Lint".into(),
            slug: "lint".into(),
            description: None,
            event: "PreToolUse".into(),
            matcher: Some("Edit|Write".into()),
            command: "just lint".into(),
            timeout_ms: 5000,
            tags: vec![],
            category: None,
            is_template: false,
        }
    }

    #[test]
    fn emits_event_and_command() {
        let out = render_hooks_json(&[sample_hook()], RuntimeId::ClaudeCode, &ToolRegistry::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["hooks"][0]["event"], "PreToolUse");
        assert_eq!(v["hooks"][0]["command"], "just lint");
    }

    #[test]
    fn matcher_translated_for_gemini() {
        let out = render_hooks_json(&[sample_hook()], RuntimeId::GeminiCli, &ToolRegistry::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["hooks"][0]["matcher"], "edit_file|write_file");
    }

    #[test]
    fn matcher_passthrough_for_claude_code() {
        let out = render_hooks_json(&[sample_hook()], RuntimeId::ClaudeCode, &ToolRegistry::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["hooks"][0]["matcher"], "Edit|Write");
    }

    #[test]
    fn zero_timeout_omitted() {
        let mut h = sample_hook();
        h.timeout_ms = 0;
        let out = render_hooks_json(&[h], RuntimeId::ClaudeCode, &ToolRegistry::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["hooks"][0]["timeout_ms"].is_null());
    }

    #[test]
    fn empty_matcher_omitted() {
        let mut h = sample_hook();
        h.matcher = Some(String::new());
        let out = render_hooks_json(&[h], RuntimeId::ClaudeCode, &ToolRegistry::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["hooks"][0]["matcher"].is_null());
    }
}
