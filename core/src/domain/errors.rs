use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum If97Error {
    OutOfBounds(String),
    NotImplemented(String),
    ConvergenceError(String),
    PhaseBoundaryError(String),
    InvalidInput(String),
}

impl std::fmt::Display for If97Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(msg) => write!(f, "Out of bounds: {}", msg),
            Self::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Self::ConvergenceError(msg) => write!(f, "Convergence error: {}", msg),
            Self::PhaseBoundaryError(msg) => write!(f, "Phase boundary error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}
impl std::error::Error for If97Error {}