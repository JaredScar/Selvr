// selvr-typechecker — name resolution, type inference, and borrow checking.
//
// Pipeline:
//   1. `Resolver`    — binds all names to their definitions (scope analysis).
//   2. `Inferencer`  — Hindley-Milner style type inference with unification.
//   3. `BorrowCheck` — ownership validation (hidden-inference model).
//
// All passes take an AST Module and produce a typed HIR (high-level IR),
// or a Vec<TypeError> if the program is ill-typed.

pub mod resolver;
pub mod ty;
pub mod infer;
pub mod borrow;
pub mod error;

pub use error::TypeError;
pub use borrow::{check_fn, OwnershipCtx, VarState, is_copy_ty};

// Re-export for test/CLI drivers.
pub use resolver::Resolver;
pub use infer::Unifier;
