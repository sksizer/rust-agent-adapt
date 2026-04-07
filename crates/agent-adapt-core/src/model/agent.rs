use serde::{Deserialize, Serialize};

/// An agent *persona* — a preset combining a model, system prompt, temperature,
/// and a tool allowlist. Not to be confused with a coding-agent *runtime*
/// (Claude Code, Codex CLI), which is a different concept represented by
/// [`crate::CodingAgentRuntime`].
///
/// Runtimes that support agent personas (e.g. Claude Code's `agents/` dir)
/// implement [`crate::AgentCapability`]; those that don't simply skip
/// agents when rendering a [`crate::PackBundle`].
///
/// # Field notes
///
/// * [`Agent::slug`] is the stable identifier used in filenames
///   (`agents/<slug>.md`). It is the author's responsibility to keep it
///   kebab-case and unique within a pack; runtimes assume it's already
///   valid and do not re-slug it.
/// * [`Agent::tools`] holds canonical tool names, translated per-runtime at
///   render time — same rules as [`crate::Skill::allowed_tools`].
/// * [`Agent::is_template`] marks an agent that is meant to be *cloned* by
///   authoring UIs rather than invoked directly. Runtimes may still render
///   template agents so they can be shared.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    /// Human-readable name, e.g. `"Code Reviewer"`.
    pub name: String,
    /// Stable kebab-case identifier used as the filename base.
    pub slug: String,
    /// Optional description shown in authoring UIs and manifests.
    pub description: Option<String>,
    /// Optional model identifier, e.g. `"sonnet"`, `"opus"`, `"gpt-4"`.
    /// `None` means "use the runtime's default."
    pub model: Option<String>,
    /// The system prompt rendered as the body of the agent file.
    pub system_prompt: Option<String>,
    /// Optional sampling temperature. `None` means "use the runtime's default."
    pub temperature: Option<f64>,
    /// Canonical tool names this agent is allowed to use.
    pub tools: Vec<String>,
    /// Free-form tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category for grouping in authoring UIs.
    pub category: Option<String>,
    /// If `true`, this agent is a template to be cloned rather than used
    /// directly.
    pub is_template: bool,
}

impl Agent {
    /// Convenience constructor with only the fields that have no sensible
    /// default.
    pub fn new(name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slug: slug.into(),
            description: None,
            model: None,
            system_prompt: None,
            temperature: None,
            tools: Vec::new(),
            tags: Vec::new(),
            category: None,
            is_template: false,
        }
    }
}
