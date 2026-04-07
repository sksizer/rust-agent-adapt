use serde::{Deserialize, Serialize};

/// The language a [`Script`] is written in. Runtimes use this to pick the
/// right shebang and file extension at render time.
///
/// The enum is deliberately small — bash, python, node — because those are
/// the only languages agentpants' adapters support today. [`ScriptLanguage::Other`]
/// is the escape hatch for runtimes with broader support; the string is
/// taken verbatim as the file extension and no shebang is emitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptLanguage {
    /// `#!/usr/bin/env bash`, extension `.sh`.
    Bash,
    /// `#!/usr/bin/env python3`, extension `.py`.
    Python,
    /// `#!/usr/bin/env node`, extension `.js`.
    Node,
    /// Caller-specified language; extension is the contained string and no
    /// shebang is emitted.
    Other(String),
}

impl ScriptLanguage {
    /// Filename extension (without leading dot) used when writing scripts to disk.
    pub fn extension(&self) -> &str {
        match self {
            Self::Bash => "sh",
            Self::Python => "py",
            Self::Node => "js",
            Self::Other(ext) => ext,
        }
    }

    /// Shebang line (without trailing newline) for this language, or `None`
    /// if the language has no standard shebang.
    pub fn shebang(&self) -> Option<&'static str> {
        match self {
            Self::Bash => Some("#!/usr/bin/env bash"),
            Self::Python => Some("#!/usr/bin/env python3"),
            Self::Node => Some("#!/usr/bin/env node"),
            Self::Other(_) => None,
        }
    }
}

/// A standalone script shipped as part of a pack.
///
/// Scripts are the most runtime-agnostic pack component — most runtimes
/// just drop them into a `scripts/` directory with the right shebang and
/// leave execution to the hook or command that invokes them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// Human-readable name.
    pub name: String,
    /// Stable kebab-case identifier used as the filename base.
    pub slug: String,
    /// Optional description.
    pub description: Option<String>,
    /// Script language — determines extension and shebang.
    pub language: ScriptLanguage,
    /// Full script body (without shebang — the shebang is prepended at
    /// render time based on [`Script::language`]).
    pub body: String,
    /// Execution timeout in milliseconds. `0` means "no timeout / runtime default."
    pub timeout_ms: i64,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category.
    pub category: Option<String>,
    /// If `true`, this script is a template to be cloned.
    pub is_template: bool,
}
