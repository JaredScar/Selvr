//! Converts `selvr-lint` diagnostics into LSP `Diagnostic` objects.

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use selvr_lint::{Linter, LintConfig, Severity};

pub fn lint_diagnostics(src: &str) -> Vec<Diagnostic> {
    let linter = Linter::new(LintConfig::default());
    let diags  = linter.check_src(src);

    diags.iter().map(|d| {
        let line = d.span.line.saturating_sub(1);
        let col  = d.span.col.saturating_sub(1);
        let range = Range {
            start: Position { line, character: col },
            end:   Position { line, character: col + (d.span.end - d.span.start).max(1) },
        };
        Diagnostic {
            range,
            severity: Some(match d.severity {
                Severity::Error   => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Info    => DiagnosticSeverity::INFORMATION,
                Severity::Hint    => DiagnosticSeverity::HINT,
            }),
            code: Some(lsp_types::NumberOrString::String(d.code.into())),
            source: Some("selvr".into()),
            message: d.message.clone(),
            ..Default::default()
        }
    }).collect()
}
