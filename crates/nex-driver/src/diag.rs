//! A minimal source-snippet diagnostic renderer.
//!
//! This is deliberately dependency-free for now; Phase 3 of the roadmap
//! replaces it with `ariadne` once the parser produces richer diagnostics.

use nex_lexer::Span;
use std::fmt::Write as _;

pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

/// Renders `diagnostics` against `src`, annotating the offending lines.
pub fn render(path: &str, src: &str, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for diagnostic in diagnostics {
        let (line_no, col_no, line_start, line_text) = locate(src, diagnostic.span.start as usize);

        let _ = writeln!(out, "error: {}", diagnostic.message);
        let _ = writeln!(out, "  --> {path}:{line_no}:{col_no}");

        let gutter_width = line_no.to_string().len();
        let pad = " ".repeat(gutter_width);
        let _ = writeln!(out, "{pad} |");
        let _ = writeln!(out, "{line_no} | {line_text}");

        // Underline the span, clamped to this line.
        let start_col = diagnostic.span.start as usize - line_start;
        let end_on_line = (diagnostic.span.end as usize).min(line_start + line_text.len());
        let width = end_on_line
            .saturating_sub(diagnostic.span.start as usize)
            .max(1);
        let _ = writeln!(
            out,
            "{pad} | {}{}",
            " ".repeat(display_width(&line_text[..start_col])),
            "^".repeat(width)
        );

        if let Some(help) = &diagnostic.help {
            let _ = writeln!(out, "{pad} = help: {help}");
        }
        out.push('\n');
    }
    out
}

/// Returns `(line_number, column_number, line_start_offset, line_text)` for a
/// byte offset. Both numbers are 1-based.
fn locate(src: &str, offset: usize) -> (usize, usize, usize, &str) {
    let offset = offset.min(src.len());
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[line_start..]
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(src.len());
    let line_text = &src[line_start..line_end];
    let line_no = src[..line_start].matches('\n').count() + 1;
    let col_no = src[line_start..offset].chars().count() + 1;
    (line_no, col_no, line_start, line_text)
}

/// Character count, used so the caret lines up under multi-byte text.
fn display_width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_finds_line_and_column() {
        let src = "let x = 1;\nlet y = 2;\n";
        let (line, col, _, text) = locate(src, 15);
        assert_eq!((line, col), (2, 5));
        assert_eq!(text, "let y = 2;");
    }

    #[test]
    fn render_points_at_the_span() {
        let src = "let x = @;\n";
        let diagnostics = vec![Diagnostic {
            message: "unexpected character `@`".into(),
            span: Span::new(8, 9),
            help: Some("remove it".into()),
        }];
        let output = render("test.nex", src, &diagnostics);
        assert!(output.contains("--> test.nex:1:9"), "{output}");
        assert!(output.contains("1 | let x = @;"), "{output}");
        assert!(output.contains("        ^"), "{output}");
        assert!(output.contains("help: remove it"), "{output}");
    }
}
