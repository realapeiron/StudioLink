use std::fmt;

#[derive(Debug)]
pub enum StudioLinkError {
    /// Plugin is not connected or not responding
    PluginNotConnected,
    /// Request timed out waiting for plugin response
    RequestTimeout(String),
    /// Plugin returned an error
    PluginError(String),
    /// Invalid tool arguments
    InvalidArguments(String),
    /// HTTP server error
    ServerError(String),
    /// MCP protocol error
    McpError(String),
    /// Serialization error
    SerializationError(String),
    /// IO error
    IoError(std::io::Error),
}

impl fmt::Display for StudioLinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PluginNotConnected => write!(f, "Studio plugin is not connected"),
            Self::RequestTimeout(id) => write!(f, "Request {} timed out", id),
            Self::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            Self::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            Self::ServerError(msg) => write!(f, "Server error: {}", msg),
            Self::McpError(msg) => write!(f, "MCP error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for StudioLinkError {}

impl From<std::io::Error> for StudioLinkError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_json::Error> for StudioLinkError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, StudioLinkError>;
