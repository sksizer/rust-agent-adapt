use serde::{Deserialize, Serialize};

/// A user-authored skill: the atomic unit of capability a runtime can
/// install and invoke.
///
/// Skills map to different on-disk layouts per runtime (e.g.
/// `.claude/skills/<slug>/SKILL.md`). This struct is the runtime-agnostic
/// content; the runtime's [`crate::SkillCapability`] impl decides the
/// filename and frontmatter dialect.
///
/// # Field notes
///
/// * [`Skill::name`] is the *human* name. Slugging to kebab-case happens
///   inside [`crate::naming::slugify_skill_name`] when the runtime needs
///   a spec-compliant filename — the struct stores the original so
///   authoring UIs can round-trip it.
/// * [`Skill::allowed_tools`] holds *canonical* tool names (see
///   [`crate::tools`]). Per-runtime translation happens at render time.
/// * [`Skill::context_mode`] matches the Agent Skills spec's `context:`
///   field — `Some("fork")` emits `context: fork`, `None` omits it.
/// * [`Skill::user_invocable`] defaults to `true`. Render impls emit
///   `user-invocable: false` only when it's `false`.
/// * [`Skill::disable_model_invocation`] is a direct mapping — `true`
///   emits the frontmatter field, `false` omits it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    /// Human-readable name, e.g. `"Requirements Interview"`.
    pub name: String,
    /// Short description shown to the model.
    pub description: String,
    /// Optional invocation hint, e.g. `"<file-path>"`.
    pub argument_hint: Option<String>,
    /// The markdown body following the YAML frontmatter in `SKILL.md`.
    pub body: String,
    /// Canonical tool names this skill is allowed to use.
    pub allowed_tools: Vec<String>,
    /// Optional per-skill model override (e.g. `"opus"`).
    pub model_override: Option<String>,
    /// Optional `context:` frontmatter value. `Some("fork")` creates a
    /// forked subagent context.
    pub context_mode: Option<String>,
    /// Whether the user can invoke this skill directly.
    pub user_invocable: bool,
    /// If `true`, the model itself cannot autonomously invoke this skill.
    pub disable_model_invocation: bool,
    /// Free-form tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Skill {
    /// Convenience constructor for a minimal skill.
    pub fn new(name: impl Into<String>, description: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            argument_hint: None,
            body: body.into(),
            allowed_tools: Vec::new(),
            model_override: None,
            context_mode: None,
            user_invocable: true,
            disable_model_invocation: false,
            tags: Vec::new(),
        }
    }
}
