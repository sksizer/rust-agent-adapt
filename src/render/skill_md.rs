//! Render a [`Skill`] into a complete `SKILL.md` body (frontmatter + markdown).
//!
//! This is the function each runtime's [`crate::SkillCapability`] impl
//! calls. It handles:
//!
//! * Slugging the human skill name to kebab-case via
//!   [`crate::naming::slugify_skill_name`] for the `name:` field.
//! * Translating each canonical tool name to the target runtime's
//!   vocabulary via [`ToolRegistry::translate_tool_name`].
//! * Honoring the runtime's [`FrontmatterDialect`] (kebab vs snake,
//!   omitted fields, `user-invocable` default handling).
//! * Emitting spec-compliant optional fields — `argument-hint`, `model`,
//!   `context: fork`, `user-invocable: false`, `disable-model-invocation: true`.

use crate::render::FrontmatterBuilder;
use crate::{FrontmatterDialect, RuntimeId, Skill, ToolRegistry, naming::slugify_skill_name};

/// Render `skill` into the full `SKILL.md` contents for the given runtime.
///
/// # Tool translation
///
/// Each entry in [`Skill::allowed_tools`] is passed through
/// `registry.translate_tool_name(_, runtime)` so authors write canonical
/// names once and every runtime sees its own vocabulary.
pub fn render_skill_md(
    skill: &Skill,
    runtime: RuntimeId,
    dialect: &FrontmatterDialect,
    registry: &ToolRegistry,
) -> String {
    let slug = slugify_skill_name(&skill.name);

    let translated_tools: Vec<String> =
        skill.allowed_tools.iter().map(|t| registry.translate_tool_name(t, runtime)).collect();

    let mut b = FrontmatterBuilder::new(dialect);
    b.scalar("name", &slug);
    b.scalar("description", &skill.description);
    b.list("allowed_tools", &translated_tools);

    if let Some(hint) = skill.argument_hint.as_deref() {
        b.scalar_quoted("argument_hint", hint);
    }

    if let Some(model) = skill.model_override.as_deref() {
        b.scalar("model", model);
    }

    // Only `context: fork` is spec-recognized today.
    if let Some(mode) = skill.context_mode.as_deref()
        && mode == "fork"
    {
        b.scalar("context", "fork");
    }

    // user-invocable defaults to true. Emit `user-invocable: false` only
    // when the skill explicitly disables it; dialects that want the
    // default stated explicitly opt in via `emit_user_invocable_default`.
    if !skill.user_invocable {
        b.boolean("user_invocable", false);
    } else if dialect.emit_user_invocable_default {
        b.boolean("user_invocable", true);
    }

    if skill.disable_model_invocation {
        b.boolean("disable_model_invocation", true);
    }

    let frontmatter = b.build();
    format!("{frontmatter}\n{body}\n", body = skill.body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FieldNaming, Skill};

    fn kebab_dialect() -> FrontmatterDialect {
        FrontmatterDialect { field_naming: FieldNaming::Kebab, omit_fields: &[], emit_user_invocable_default: false }
    }

    fn minimal_skill() -> Skill {
        let mut s = Skill::new("My Skill", "A test skill", "Do the thing.");
        s.allowed_tools = vec!["Read".into(), "Write".into()];
        s
    }

    #[test]
    fn renders_name_as_slug() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("name: my-skill"));
    }

    #[test]
    fn translates_tools_for_gemini() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::GeminiCli, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("  - read_file"));
        assert!(out.contains("  - write_file"));
    }

    #[test]
    fn claude_code_keeps_native_tool_names() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("  - Read"));
        assert!(out.contains("  - Write"));
    }

    #[test]
    fn kebab_dialect_uses_hyphenated_fields() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("allowed-tools:"));
        assert!(!out.contains("allowed_tools:"));
    }

    #[test]
    fn user_invocable_true_is_omitted_by_default() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(!out.contains("user-invocable"));
    }

    #[test]
    fn user_invocable_false_is_emitted() {
        let mut s = minimal_skill();
        s.user_invocable = false;
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("user-invocable: false"));
    }

    #[test]
    fn disable_model_invocation_emitted_when_true() {
        let mut s = minimal_skill();
        s.disable_model_invocation = true;
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("disable-model-invocation: true"));
    }

    #[test]
    fn argument_hint_quoted() {
        let mut s = minimal_skill();
        s.argument_hint = Some("<file>".into());
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("argument-hint: \"<file>\""));
    }

    #[test]
    fn model_override_emitted() {
        let mut s = minimal_skill();
        s.model_override = Some("opus".into());
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("model: opus"));
    }

    #[test]
    fn context_fork_emitted_when_set() {
        let mut s = minimal_skill();
        s.context_mode = Some("fork".into());
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(out.contains("context: fork"));
    }

    #[test]
    fn context_non_fork_is_ignored() {
        let mut s = minimal_skill();
        s.context_mode = Some("something-else".into());
        let out = render_skill_md(&s, RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        assert!(!out.contains("context:"));
    }

    #[test]
    fn body_follows_frontmatter() {
        let out = render_skill_md(&minimal_skill(), RuntimeId::ClaudeCode, &kebab_dialect(), &ToolRegistry::default());
        let (_, after) = out.split_once("---\n").unwrap();
        let (_, body) = after.split_once("---\n").unwrap();
        assert!(body.contains("Do the thing."));
    }
}
