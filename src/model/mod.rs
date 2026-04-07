//! Pure content models — the shapes runtime capability traits consume.
//!
//! Every struct here is a **subset** of what an application might persist:
//! we carry only the fields a runtime needs to render an export. Database
//! metadata (`id`, `created_at`, `version`) is the caller's responsibility —
//! they keep a row type containing a `content: agent_adapt::Skill` and hand
//! the content subset to the crate at the API boundary.
//!
//! All models derive `Serialize`/`Deserialize` so pack manifests and
//! authored content can round-trip through JSON, YAML, or TOML without
//! custom glue.

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
