//! Data model and provider implementations for writing MCP server entries into
//! AI agent config files.
//!
//! # Usage
//!
//! ```rust,no_run
//! use agent_adapt::{McpServer, providers::install_to_all};
//! use std::path::Path;
//!
//! let server = McpServer::http("my-server", "http://localhost:4243/mcp");
//! let failures = install_to_all(Path::new("/my/project"), &server);
//! for (agent, err) in failures {
//!     eprintln!("Failed to configure {agent}: {err}");
//! }
//! ```

mod error;
mod model;
pub mod providers;

pub use error::Error;
pub use model::{McpServer, McpTransport};
pub use providers::{AgentConfigProvider, all_providers, install_to_all};
