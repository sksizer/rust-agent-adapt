use serde::{Deserialize, Serialize};

/// A user-authored skill: the atomic unit of capability that a coding-agent
/// runtime can install and invoke.
///
/// Skills map onto different on-disk layouts per runtime (e.g. Claude Code's
/// `.claude/skills/<name>/SKILL.md`, Codex CLI's `.codex/skills/<name>/SKILL.md`).
/// This struct is the runtime-agnostic content; the runtime's
/// [`crate::SkillCapability`] impl decides the filename and frontmatter
/// dialect.
///
/// # Field notes
///
/// * [`Skill::name`] is the *human* name. Slugging to kebab-case happens
///   inside [`crate::naming::slugify_skill_name`] when the runtime needs a
///   spec-compliant filename — the struct stores the original so the
///   authoring UI can round-trip it.
/// * [`Skill::allowed_tools`] holds *canonical* tool names (see
///   `agent-adapt-tools`). Per-runtime translation happens at render time.
/// * [`Skill::context_mode`] matches the Agent Skills spec's `context:`
///   field — `Some("fork")` emits `context: fork`, `None` omits it.
/// * [`Skill::user_invocable`] defaults to `true` in authoring UIs; render
///   impls emit `user-invocable: false` only when it's `false`, to match the
///   spec's "omit defaults" convention.
/// * [`Skill::disable_model_invocation`] is a direct mapping — `true`
///   emits `disable-model-invocation: true`, `false` omits the field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    /// Human-readable name, e.g. `"Requirements Interview"`.
    pub name: String,
    /// One-line (or short multi-line) description shown to the model.
    pub description: String,
    /// Optional hint shown next to the skill in invocation UIs, e.g.
    /// `"<file-path>"`. `None` or empty string omits the frontmatter field.
    pub argument_hint: Option<String>,
    /// The markdown body that follows the YAML frontmatter in `SKILL.md`.
    pub body: String,
    /// Canonical tool names this skill is allowed to use. Rendered into the
    /// per-runtime dialect (e.g. `Read` → `read_file` for Gemini CLI) at
    /// export time.
    pub allowed_tools: Vec<String>,
    /// Optional per-skill model override (e.g. `"opus"`). `None` means "use
    /// whatever the runtime's default is."
    pub model_override: Option<String>,
    /// Optional `context:` frontmatter value. `Some("fork")` creates a
    /// forked subagent context; `None` omits the field.
    pub context_mode: Option<String>,
    /// Whether the user can invoke this skill directly (via `/skill-name`).
    /// `true` is the default; `false` restricts invocation to the model.
    pub user_invocable: bool,
    /// If `true`, the model itself is prevented from autonomously invoking
    /// this skill — only the user can trigger it.
    pub disable_model_invocation: bool,
    /// Free-form tags for categorization in authoring UIs. Not rendered
    /// into frontmatter by default.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Skill {
    /// Convenience constructor for a minimal skill with just name, description,
    /// and body. All other fields take their defaults.
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
