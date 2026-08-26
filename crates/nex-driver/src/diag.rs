//! source-annotated diagnostic rendering, backed by `ariadne`.
//!
//! the hand-rolled renderer this replaced (steps 1.11a-3.10) kept the crate
//! dependency-free while only the lexer produced errors. now that the parser
//! reports spans too, `ariadne` handles the labels, gutters and multi-line
//! spans instead.

use ariadne::{Config, Label, Report, ReportKind, Source};
use nex_lexer::Span;

pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

/// renders every diagnostic against `src`, one report each.
///
/// colour is off: the output is compared in tests and piped to files at
/// least as often as it is read on a terminal.
pub fn render(path: &str, src: &str, diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for diagnostic in diagnostics {
        out.push_str(&render_one(path, src, diagnostic));
    }
    out
}

/// nex spans are byte offsets; `ariadne`'s `Source` indexes by character.
/// they only coincide for ascii, so a multi-byte character earlier in the
/// line would otherwise push the reported column to the right.
///
/// a byte offset that isn't on a character boundary is rounded down rather
/// than panicking - spans should always land on one, but a renderer is the
/// wrong place to enforce that.
fn char_offset(src: &str, byte: usize) -> usize {
    let mut byte = byte.min(src.len());
    while byte > 0 && !src.is_char_boundary(byte) {
        byte -= 1;
    }
    src[..byte].chars().count()
}

fn render_one(path: &str, src: &str, diagnostic: &Diagnostic) -> String {
    let start = char_offset(src, diagnostic.span.start as usize);
    // `ariadne` panics on an empty or reversed range, and the `Eof` token's
    // span is empty by construction, so widen it to one character.
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
    // writing into a Vec cannot fail, and a diagnostic is the wrong place
    // to surface an io error anyway
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

    // multi-byte characters must not shift the reported column, and must
    // not panic the renderer
    #[test]
    fn render_handles_multibyte_characters() {
        let src = "let café = @\n";
        let at = src.find('@').expect("an @ in the fixture");
        let output = one(src, Span::from_usize(at, at + 1), None);
        assert!(output.contains("test.nex:1:12"), "{output}");
    }

    // the Eof token's span is empty; ariadne rejects an empty range, so
    // render widens it rather than panicking
    #[test]
    fn render_survives_an_empty_span() {
        let src = "let x =";
        let end = src.len();
        let output = one(src, Span::from_usize(end, end), None);
        assert!(output.contains("test.nex"), "{output}");
    }
}
