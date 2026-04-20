use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("code generation encountered an untyped node (type checker must run first)")]
    UntypedNode,

    #[error("unsupported feature in JS backend: {0}")]
    Unsupported(String),
}
