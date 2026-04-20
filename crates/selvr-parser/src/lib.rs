// selvr-parser — produces a typed AST from a token stream.
//
// Syntax is TypeScript-adjacent:
//   fn name(params): ReturnType { ... }   — colon return type
//   const x = 5;                          — immutable binding
//   let x = 5;                            — mutable binding
//   (x: i32) => x * 2                     — arrow closures
//   `Hello ${name}!`                      — template literals
//   import { X } from "module"            — ES-module style imports
//   export fn / export const              — visibility
//   Enum.Variant                          — dot enum access
//
// Entry: `Parser::new(tokens, file_id).parse()` → `(Module, Vec<ParseError>)`.

pub mod ast;
pub mod parser;
pub mod error;

pub use parser::Parser;
pub use error::ParseError;
