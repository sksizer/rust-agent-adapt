use serde::{Deserialize, Serialize};

use super::{Agent, Hook, Role, Script, Skill};

/// Pack metadata — identity and provenance without the assets. The assets
/// live in [`PackBundle`].
///
/// This split exists so authoring UIs can list packs cheaply (just
/// metadata) and load the full bundle only when exporting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    /// Human-readable name.
    pub name: String,
    /// Stable kebab-case identifier.
    pub slug: String,
    /// Optional namespace prefix (e.g. `"acme-corp"`).
    pub namespace: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Human-facing version label (e.g. `"1.2.0"`).
    pub version_label: String,
    /// Optional author string.
    pub author: Option<String>,
    /// Optional SPDX license identifier.
    pub license: Option<String>,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category.
    pub category: Option<String>,
}

/// A complete authoring bundle: pack metadata plus every asset it owns.
///
/// This is what gets handed to a runtime's render function. Runtimes walk
/// the fields in order (skills → agents → hooks → scripts) and produce a
/// [`crate::ExportedTree`] of virtual files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackBundle {
    /// The pack's identity/metadata.
    pub pack: Pack,
    /// All skills owned by this pack.
    #[serde(default)]
    pub skills: Vec<Skill>,
    /// All roles referenced by agents or skills.
    #[serde(default)]
    pub roles: Vec<Role>,
    /// All agent personas.
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// All hooks.
    #[serde(default)]
    pub hooks: Vec<Hook>,
    /// All standalone scripts.
    #[serde(default)]
    pub scripts: Vec<Script>,
}

impl PackBundle {
    /// Create an empty bundle wrapping the given pack metadata.
    pub fn new(pack: Pack) -> Self {
        Self { pack, skills: Vec::new(), roles: Vec::new(), agents: Vec::new(), hooks: Vec::new(), scripts: Vec::new() }
    }
}
