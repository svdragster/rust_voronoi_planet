//! Error types for voronoi planet generation

use std::fmt;

/// Errors that can occur during planet generation or queries
#[derive(Debug, Clone)]
pub enum VoronoiError {
    /// Configuration validation failed
    InvalidConfig(String),
    /// Generation failed due to geometry issues
    GenerationFailed(String),
    /// Requested cell ID does not exist
    CellNotFound(usize),
}

impl fmt::Display for VoronoiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoronoiError::InvalidConfig(msg) => write!(f, "invalid configuration: {}", msg),
            VoronoiError::GenerationFailed(msg) => write!(f, "generation failed: {}", msg),
            VoronoiError::CellNotFound(id) => write!(f, "cell not found: {}", id),
        }
    }
}

impl std::error::Error for VoronoiError {}

/// Result type alias for voronoi operations
pub type Result<T> = std::result::Result<T, VoronoiError>;
