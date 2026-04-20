use thiserror::Error;
use smol_str::SmolStr;
use selvr_lexer::span::Span;

#[derive(Debug, Error, Clone)]
pub enum TypeError {
    #[error("type mismatch: expected `{expected}`, found `{found}` at {span:?}")]
    TypeMismatch { expected: String, found: String, span: Span },

    #[error("unresolved name `{name}` at {span:?}")]
    UnresolvedName { name: SmolStr, span: Span },

    #[error("infinite type detected (occurs check failed) at {span:?}")]
    InfiniteType { span: Span },

    #[error("field `{field}` does not exist on type `{ty}` at {span:?}")]
    NoSuchField { field: SmolStr, ty: String, span: Span },

    #[error("variant `{variant}` does not exist on enum `{ty}` at {span:?}")]
    NoSuchVariant { variant: SmolStr, ty: String, span: Span },

    #[error("function `{name}` expects {expected} arguments but got {found} at {span:?}")]
    ArgCountMismatch { name: SmolStr, expected: usize, found: usize, span: Span },

    #[error("cannot assign to immutable binding `{name}` at {span:?}")]
    ImmutableAssign { name: SmolStr, span: Span },

    #[error("`{name}` is not a function at {span:?}")]
    NotCallable { name: SmolStr, span: Span },

    #[error("missing return value in function `{name}` at {span:?}")]
    MissingReturn { name: SmolStr, span: Span },

    #[error("cannot use `await` outside of an `async fn` at {span:?}")]
    AwaitOutsideAsync { span: Span },

    #[error("pattern is not exhaustive at {span:?}")]
    NonExhaustiveMatch { span: Span },

    #[error("use of moved value `{name}` at {span:?}")]
    UseAfterMove { name: SmolStr, span: Span },
}
