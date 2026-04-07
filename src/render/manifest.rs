//! Render a [`PackBundle`] into a human-readable manifest markdown file.
//!
//! Claude Code calls this `CLAUDE.md`, other runtimes call it `README.md`
//! or `MANIFEST.md`. The content is the same: pack metadata, then a list
//! of each asset kind with its name and description. The caller picks the
//! filename.

use crate::{PackBundle, naming::slugify_skill_name};

/// Render a pack bundle into a markdown manifest string.
///
/// Sections appear in a fixed order (skills → agents → hooks → scripts)
/// and entries within each section follow the order they appear in the
/// bundle. Empty sections are omitted entirely.
pub fn render_pack_manifest(bundle: &PackBundle) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", bundle.pack.name));

    if let Some(desc) = bundle.pack.description.as_deref()
        && !desc.is_empty()
    {
        out.push_str(desc);
        out.push_str("\n\n");
    }

    if let Some(author) = bundle.pack.author.as_deref()
        && !author.is_empty()
    {
        out.push_str(&format!("**Author:** {author}\n"));
    }

    out.push_str(&format!("**Version:** {}\n\n", bundle.pack.version_label));

    if !bundle.skills.is_empty() {
        out.push_str("## Skills\n\n");
        for skill in &bundle.skills {
            let slug = slugify_skill_name(&skill.name);
            out.push_str(&format!("- `/{slug}`"));
            if !skill.description.is_empty() {
                let first_line = skill.description.lines().next().unwrap_or("");
                out.push_str(&format!(" — {first_line}"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !bundle.agents.is_empty() {
        out.push_str("## Agents\n\n");
        for agent in &bundle.agents {
            out.push_str(&format!("- `{}`", agent.slug));
            if let Some(desc) = agent.description.as_deref()
                && !desc.is_empty()
            {
                let first_line = desc.lines().next().unwrap_or("");
                out.push_str(&format!(" — {first_line}"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !bundle.hooks.is_empty() {
        out.push_str("## Hooks\n\n");
        for hook in &bundle.hooks {
            out.push_str(&format!("- `{}` ({})", hook.name, hook.event));
            if let Some(desc) = hook.description.as_deref()
                && !desc.is_empty()
            {
                out.push_str(&format!(" — {desc}"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !bundle.scripts.is_empty() {
        out.push_str("## Scripts\n\n");
        for script in &bundle.scripts {
            out.push_str(&format!("- `{}`", script.slug));
            if let Some(desc) = script.description.as_deref()
                && !desc.is_empty()
            {
                out.push_str(&format!(" — {desc}"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Agent, Hook, Pack, PackBundle, Script, ScriptLanguage, Skill};

    fn sample_pack() -> Pack {
        Pack {
            name: "Test Pack".into(),
            slug: "test-pack".into(),
            namespace: None,
            description: Some("A sample pack".into()),
            version_label: "1.0.0".into(),
            author: Some("Author".into()),
            license: None,
            tags: vec![],
            category: None,
        }
    }

    fn sample_bundle() -> PackBundle {
        let mut b = PackBundle::new(sample_pack());
        b.skills.push(Skill::new("My Skill", "A short description", "body"));
        b.agents.push(Agent::new("Reviewer", "reviewer"));
        b.hooks.push(Hook {
            name: "On edit".into(),
            slug: "on-edit".into(),
            description: Some("Lint after edit".into()),
            event: "PostToolUse".into(),
            matcher: Some("Edit".into()),
            command: "just lint".into(),
            timeout_ms: 5000,
            tags: vec![],
            category: None,
            is_template: false,
        });
        b.scripts.push(Script {
            name: "Lint".into(),
            slug: "lint".into(),
            description: Some("Runs linter".into()),
            language: ScriptLanguage::Bash,
            body: "cargo clippy".into(),
            timeout_ms: 0,
            tags: vec![],
            category: None,
            is_template: false,
        });
        b
    }

    #[test]
    fn manifest_contains_pack_header() {
        let m = render_pack_manifest(&sample_bundle());
        assert!(m.starts_with("# Test Pack\n"));
        assert!(m.contains("A sample pack"));
        assert!(m.contains("**Author:** Author"));
        assert!(m.contains("**Version:** 1.0.0"));
    }

    #[test]
    fn manifest_lists_skills_with_slug_and_description() {
        let m = render_pack_manifest(&sample_bundle());
        assert!(m.contains("## Skills"));
        assert!(m.contains("- `/my-skill` — A short description"));
    }

    #[test]
    fn manifest_lists_all_sections() {
        let m = render_pack_manifest(&sample_bundle());
        assert!(m.contains("## Skills"));
        assert!(m.contains("## Agents"));
        assert!(m.contains("## Hooks"));
        assert!(m.contains("## Scripts"));
    }

    #[test]
    fn manifest_omits_empty_sections() {
        let m = render_pack_manifest(&PackBundle::new(sample_pack()));
        assert!(!m.contains("## Skills"));
        assert!(!m.contains("## Agents"));
        assert!(!m.contains("## Hooks"));
        assert!(!m.contains("## Scripts"));
    }

    #[test]
    fn manifest_hook_entry_has_event() {
        let m = render_pack_manifest(&sample_bundle());
        assert!(m.contains("- `On edit` (PostToolUse)"));
    }
}
