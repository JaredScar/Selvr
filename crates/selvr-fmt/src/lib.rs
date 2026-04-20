//! `selvr-fmt` — canonical pretty-printer for Selvr source.
//!
//! # Usage
//! ```no_run
//! # fn main() -> Result<(), selvr_fmt::FmtError> {
//! use selvr_fmt::Formatter;
//! let source = "fn main() { }";
//! let _formatted = Formatter::new().format_src(source)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Style rules (canonical Selvr)
//! - 4-space indentation, no tabs.
//! - One blank line between top-level items.
//! - `const`/`let` keyword, colon type annotation with a space on each side.
//! - Braces on the same line as the keyword (`fn foo() {`, not a new line).
//! - Trailing newline.

pub mod printer;

use selvr_lexer::Lexer;
use selvr_parser::Parser;
use selvr_parser::ast::Module;
use printer::Printer;

#[derive(thiserror::Error, Debug)]
pub enum FmtError {
    #[error("parse error(s): {0}")]
    Parse(String),
}

/// Top-level entry point.
pub struct Formatter {
    /// Indentation width in spaces.
    indent: usize,
}

impl Default for Formatter {
    fn default() -> Self { Self::new() }
}

impl Formatter {
    pub fn new() -> Self { Self { indent: 4 } }

    /// Parse `src`, format it, return the canonical representation.
    pub fn format_src(&self, src: &str) -> Result<String, FmtError> {
        let (tokens, lex_errors) = Lexer::new(src, 0).tokenize();
        let (module, parse_errors) = Parser::new(tokens, 0).parse();

        let mut msgs = Vec::new();
        for e in &lex_errors   { msgs.push(e.to_string()); }
        for e in &parse_errors { msgs.push(e.to_string()); }
        if !msgs.is_empty() {
            return Err(FmtError::Parse(msgs.join("; ")));
        }

        Ok(self.format_module(&module))
    }

    /// Format an already-parsed module.
    pub fn format_module(&self, module: &Module) -> String {
        let mut p = Printer::new(self.indent);
        p.print_module(module);
        p.finish()
    }
}
