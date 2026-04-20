// selvr-lexer — tokenizer for the Selvr language.
//
// Entry point: `Lexer::new(source).tokenize()` returns `Vec<Spanned<Token>>`.
// All token variants are defined in `token.rs`.
// Lexer errors are recoverable; invalid tokens produce `Token::Error` so the
// parser can continue and report multiple errors in one pass.

pub mod token;
pub mod span;
pub mod lexer;
pub mod error;

pub use lexer::Lexer;
pub use token::Token;
pub use span::{Span, Spanned};
pub use error::LexError;
