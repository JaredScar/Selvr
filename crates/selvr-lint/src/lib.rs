//! `selvr-lint` — static analysis for Selvr source code.
//!
//! # Built-in rules
//!
//! | Code      | Name                | Severity |
//! |-----------|---------------------|----------|
//! | SL001     | unused_variable     | Warning  |
//! | SL002     | dead_code           | Warning  |
//! | SL003     | missing_return      | Error    |
//! | SL004     | shadow_variable     | Warning  |
//! | SL005     | force_target_hint   | Info     |
//! | SL006     | wasm_dom_call       | Error    |
//! | SL007     | js_heavy_compute    | Warning  |
//!
//! # Usage
//! ```no_run
//! use selvr_lint::{Linter, LintConfig};
//! let config = LintConfig::default();
//! let diags = Linter::new(config).check_src(source);
//! for d in &diags { eprintln!("{d}"); }
//! ```

pub mod rules;

use selvr_lexer::Lexer;
use selvr_parser::{Parser, ast::Module};
use selvr_ir::lower_module;
use selvr_target::{infer_targets, propagate_targets};
use serde::{Serialize, Deserialize};

// ── Diagnostic types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity { Error, Warning, Info, Hint }

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error   => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info    => write!(f, "info"),
            Self::Hint    => write!(f, "hint"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintSpan {
    /// 0-based byte offset in source.
    pub start: u32,
    pub end:   u32,
    /// 1-based for display.
    pub line:  u32,
    pub col:   u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintDiagnostic {
    /// Rule code, e.g. "SL001".
    pub code:     &'static str,
    pub name:     &'static str,
    pub severity: Severity,
    pub message:  String,
    pub span:     LintSpan,
    /// Optional fix suggestion shown in IDE quick-fix.
    pub fix:      Option<String>,
}

impl std::fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}[{}] at {}:{} — {}",
            self.severity, self.code, self.span.line, self.span.col, self.message
        )
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Rules explicitly disabled.
    pub deny:  Vec<&'static str>,
    /// Rules treated as errors even if normally warnings.
    pub allow: Vec<&'static str>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self { deny: Vec::new(), allow: Vec::new() }
    }
}

// ── Linter ────────────────────────────────────────────────────────────────────

pub struct Linter {
    config: LintConfig,
}

impl Linter {
    pub fn new(config: LintConfig) -> Self { Self { config } }

    /// Parse and lint source, returning all diagnostics.
    pub fn check_src(&self, src: &str) -> Vec<LintDiagnostic> {
        let (tokens, lex_errors) = Lexer::new(src, 0).tokenize();
        if !lex_errors.is_empty() {
            return lex_errors.iter().map(|e| LintDiagnostic {
                code:     "SL000",
                name:     "lex_error",
                severity: Severity::Error,
                message:  e.to_string(),
                span:     LintSpan { start: 0, end: 0, line: 1, col: 1 },
                fix:      None,
            }).collect();
        }

        let (module, parse_errors) = Parser::new(tokens, 0).parse();
        if !parse_errors.is_empty() {
            return parse_errors.iter().map(|e| LintDiagnostic {
                code:     "SL000",
                name:     "parse_error",
                severity: Severity::Error,
                message:  e.to_string(),
                span:     LintSpan { start: 0, end: 0, line: 1, col: 1 },
                fix:      None,
            }).collect();
        }

        self.check_module(src, &module)
    }

    /// Lint an already-parsed module.
    pub fn check_module(&self, src: &str, module: &Module) -> Vec<LintDiagnostic> {
        let mut diags = Vec::new();

        // AST-level rules
        rules::unused_var::check(src, module, &mut diags);
        rules::dead_code::check(src, module, &mut diags);
        rules::missing_return::check(src, module, &mut diags);
        rules::shadow_variable::check(src, module, &mut diags);

        // IR + targeting rules
        let mut ir  = lower_module(module);
        let mut map = infer_targets(&mut ir);
        propagate_targets(&mut ir, &mut map);
        rules::target_hint::check(src, &map, &mut diags);

        // Filter out explicitly allowed / denied rules
        diags.retain(|d| {
            let allowed = self.config.allow.contains(&d.code);
            let denied  = self.config.deny.contains(&d.code);
            !allowed && !denied
        });

        diags.sort_by_key(|d| (d.span.line, d.span.col));
        diags
    }

    /// Emit diagnostics as newline-delimited JSON (for LSP / tooling).
    pub fn to_ndjson(diags: &[LintDiagnostic]) -> String {
        diags.iter()
            .map(|d| serde_json::to_string(d).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
