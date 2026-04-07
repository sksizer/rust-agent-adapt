use std::collections::HashMap;

/// A single MCP server entry to be registered with an agent.
#[derive(Debug, Clone)]
pub struct McpServer {
    /// The key under which this server will be registered (e.g. `"ontological"`).
    pub name: String,
    /// How the agent should connect to the server.
    pub transport: McpTransport,
}

/// The connection transport for an MCP server.
#[derive(Debug, Clone)]
pub enum McpTransport {
    /// Streamable HTTP transport: the agent connects to a URL.
    Http { url: String },
    /// Stdio transport: the agent spawns a subprocess.
    Stdio {
        command: String,
        args: Vec<String>,
        /// Optional environment variables to pass to the subprocess.
        env: HashMap<String, String>,
    },
}

impl McpServer {
    /// Convenience constructor for an HTTP transport server.
    pub fn http(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self { name: name.into(), transport: McpTransport::Http { url: url.into() } }
    }

    /// Convenience constructor for a stdio transport server (no env vars).
    pub fn stdio(
        name: impl Into<String>,
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Stdio {
                command: command.into(),
                args: args.into_iter().map(Into::into).collect(),
                env: HashMap::new(),
            },
        }
    }
}
