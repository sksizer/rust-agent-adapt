//! Pure content models — the shapes that runtime capability traits consume.
//!
//! Every struct in this module is a **subset** of what an agentpants-style
//! application persists: we carry only the fields that a runtime needs to
//! render an export. Database metadata (`id`, `created_at`, `version`) is
//! the caller's responsibility — they keep a row type that contains a
//! `content: agent_adapt_core::Skill` (or similar) and hand the content
//! subset to the crate at the API boundary.
//!
//! All models derive `Serialize`/`Deserialize` so pack manifests and
//! authored content can round-trip through JSON, YAML, or TOML without
//! custom glue. None of them own any I/O or validation state.

mod agent;
mod hook;
mod pack;
mod role;
mod script;
mod skill;

pub use agent::Agent;
pub use hook::Hook;
pub use pack::{Pack, PackBundle};
pub use role::Role;
pub use script::{Script, ScriptLanguage};
pub use skill::Skill;
