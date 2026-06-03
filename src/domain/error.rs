use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum AppError {
    CliNotFound,
    CommandFailed(String),
    ParseFailed(String),
    Timeout,
    PermissionDenied,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CliNotFound => write!(f, "tailscale CLI not found in PATH"),
            Self::CommandFailed(msg) => write!(f, "command failed: {msg}"),
            Self::ParseFailed(msg) => write!(f, "failed to parse tailscale output: {msg}"),
            Self::Timeout => write!(f, "tailscale command timed out"),
            Self::PermissionDenied => write!(f, "permission denied while executing tailscale"),
        }
    }
}
