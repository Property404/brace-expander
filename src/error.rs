use crate::Token;
use std::{fmt, num::NonZeroUsize};

/// Error type for this crate
#[derive(Clone, Debug)]
pub struct Error {
    column: Option<NonZeroUsize>,
    message: &'static str,
}

impl Error {
    pub(crate) fn new(message: &'static str) -> Self {
        Self {
            column: None,
            message,
        }
    }

    pub(crate) fn with_context(message: &'static str, token: Token) -> Self {
        Self {
            column: NonZeroUsize::new(token.pos.strict_add(1)),
            message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(column) = self.column {
            write!(f, " on column {column}")?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
