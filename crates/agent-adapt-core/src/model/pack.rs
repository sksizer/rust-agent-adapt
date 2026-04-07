use serde::{Deserialize, Serialize};

use super::{Agent, Hook, Role, Script, Skill};

/// Pack metadata — the identity and provenance of a bundle of agent
/// assets, without the assets themselves. The assets live in
/// [`PackBundle`].
///
/// This split exists so authoring UIs can list packs cheaply (just metadata)
/// and only load the full [`PackBundle`] when exporting.
///
/// # Field notes
///
/// * [`Pack::slug`] is the stable identifier used in export paths and
///   manifests; keep it kebab-case.
/// * [`Pack::namespace`] is an optional grouping prefix — e.g. a user or
///   org handle — used by registries that allow multiple packs with the
///   same slug.
/// * [`Pack::version_label`] is the *human-facing* version string, e.g.
///   `"1.2.0"` or `"2024-03-edge"`. Runtimes render it into manifests as-is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    /// Human-readable name.
    pub name: String,
    /// Stable kebab-case identifier.
    pub slug: String,
    /// Optional namespace prefix, e.g. `"acme-corp"`.
    pub namespace: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Human-facing version label.
    pub version_label: String,
    /// Optional author string — free-form, not parsed.
    pub author: Option<String>,
    /// Optional SPDX-style license identifier.
    pub license: Option<String>,
    /// Free-form tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category.
    pub category: Option<String>,
}

/// A complete authoring bundle: pack metadata plus every asset it owns.
///
/// This is what gets handed to a runtime's render function. Runtimes walk
/// the fields in order (skills → agents → hooks → scripts → manifest) and
/// produce a [`crate::ExportedTree`] of virtual files.
///
/// # Partial capability support
///
/// Not every runtime supports every asset type. For example, the hypothetical
/// `npm_package` runtime may not have hooks. When a runtime lacks a
/// capability, the composition function at the crate root skips that asset
/// type rather than erroring — see `render_pack` (in a downstream crate)
/// for the exact semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackBundle {
    /// The pack's identity/metadata.
    pub pack: Pack,
    /// All skills owned by this pack.
    #[serde(default)]
    pub skills: Vec<Skill>,
    /// All roles referenced by agents or skills in this pack.
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
