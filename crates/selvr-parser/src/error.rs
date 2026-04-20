use thiserror::Error;
use selvr_lexer::{span::Span, token::Token};

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("expected {expected}, found {found} at {span:?}")]
    Expected {
        expected: &'static str,
        found: String,
        span: Span,
    },

    #[error("unexpected end of file; expected {expected}")]
    UnexpectedEof { expected: &'static str },

    #[error("trailing tokens after end of item at {span:?}")]
    TrailingTokens { span: Span },
}

impl ParseError {
    pub fn expected(expected: &'static str, found: &Token, span: Span) -> Self {
        ParseError::Expected {
            expected,
            found: found.describe().to_owned(),
            span,
        }
    }
}
