//! source-annotated diagnostics, rendered with `ariadne`.

use ariadne::{Config, Label, Report, ReportKind, Source};
use nex_lexer::Span;

pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

/// one report per diagnostic. colour is off so the output is stable when
/// piped or compared in tests.
pub fn render(path: &str, src: &str, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for diagnostic in diagnostics {
        out.push_str(&render_one(path, src, diagnostic));
    }
    out
}

/// our spans are byte offsets but ariadne counts characters, so a multi-byte
/// char earlier in the line would shift the reported column.
fn char_offset(src: &str, byte: usize) -> usize {
    let mut byte = byte.min(src.len());
    while byte > 0 && !src.is_char_boundary(byte) {
        byte -= 1;
    }
    src[..byte].chars().count()
}

fn render_one(path: &str, src: &str, diagnostic: &Diagnostic) -> String {
    let start = char_offset(src, diagnostic.span.start as usize);
    // ariadne rejects an empty range, and the Eof token's span is empty
    let end = char_offset(src, diagnostic.span.end as usize).max(start + 1);
    let span = (path, start..end);

    let mut report = Report::build(ReportKind::Error, span.clone())
        .with_config(Config::default().with_color(false))
        .with_message(&diagnostic.message)
        .with_label(Label::new(span).with_message(&diagnostic.message));

    if let Some(help) = &diagnostic.help {
        report = report.with_help(help);
    }

    let mut buf = Vec::new();
    let _ = report.finish().write((path, Source::from(src)), &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(src: &str, span: Span, help: Option<&str>) -> String {
        let diagnostics = vec![Diagnostic {
            message: "unexpected character `@`".into(),
            span,
            help: help.map(str::to_string),
        }];
        render("test.nex", src, &diagnostics)
    }

    #[test]
    fn render_names_the_file_and_the_line() {
        let output = one("let x = @;\n", Span::new(8, 9), Some("remove it"));
        assert!(output.contains("test.nex:1:9"), "{output}");
        assert!(output.contains("let x = @;"), "{output}");
        assert!(output.contains("unexpected character `@`"), "{output}");
        assert!(output.contains("remove it"), "{output}");
    }

    #[test]
    fn render_omits_the_help_line_when_there_is_none() {
        let output = one("let x = @;\n", Span::new(8, 9), None);
        assert!(!output.contains("Help"), "{output}");
    }

    #[test]
    fn render_points_at_the_right_line_of_a_multi_line_file() {
        let src = "let x = 1;\nlet y = @;\n";
        let at = src.find('@').expect("an @ in the fixture");
        let output = one(src, Span::from_usize(at, at + 1), None);
        assert!(output.contains("test.nex:2:9"), "{output}");
    }

    #[test]
    fn render_handles_multibyte_characters() {
        let src = "let café = @\n";
        let at = src.find('@').expect("an @ in the fixture");
        let output = one(src, Span::from_usize(at, at + 1), None);
        assert!(output.contains("test.nex:1:12"), "{output}");
    }

    #[test]
    fn render_survives_an_empty_span() {
        let src = "let x =";
        let end = src.len();
        let output = one(src, Span::from_usize(end, end), None);
        assert!(output.contains("test.nex"), "{output}");
    }
}
