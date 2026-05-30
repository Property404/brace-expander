use crate::Token;
use std::{fmt, num::NonZeroUsize};

/// Error type for this crate
#[derive(Clone, Debug)]
pub enum Error {
    /// Found backslash at end of input
    IncompleteEscape,
    /// Encountered error while parsing
    ParserError {
        column: Option<NonZeroUsize>,
        message: &'static str,
    },
}

impl Error {
    pub(crate) fn new(message: &'static str) -> Self {
        Self::ParserError {
            column: None,
            message,
        }
    }

    pub(crate) fn with_context(message: &'static str, token: &Token) -> Self {
        Self::ParserError {
            column: NonZeroUsize::new(token.pos.strict_add(1)),
            message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteEscape => {
                write!(f, "Incomplete input - backslash found at end of input")?;
            }
            Self::ParserError { column, message } => {
                write!(f, "{}", message)?;
                if let Some(column) = column {
                    write!(f, " on column {column}")?;
                }
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
