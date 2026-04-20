use thiserror::Error;
use crate::span::Span;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LexError {
    #[error("unexpected character `{ch}` at {span:?}")]
    UnexpectedChar { ch: char, span: Span },

    #[error("unterminated string literal starting at {span:?}")]
    UnterminatedString { span: Span },

    #[error("unterminated block comment starting at {span:?}")]
    UnterminatedBlockComment { span: Span },

    #[error("invalid escape sequence `\\{ch}` at {span:?}")]
    InvalidEscape { ch: char, span: Span },

    #[error("integer literal out of range at {span:?}")]
    IntegerOverflow { span: Span },
}
