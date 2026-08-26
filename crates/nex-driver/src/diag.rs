//! tiny source-snippet diagnostic renderer.
//!
//! dependency-free on purpose for now. swap it for `ariadne` in phase 3 once the
//! parser starts producing richer diagnostics.

use nex_lexer::Span;
use std::fmt::Write as _;

pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

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

        // both offsets are byte offsets into line_text; the caret row is
        // measured in display columns, so convert through display_width
        // rather than counting bytes (a multi-byte char is one column here,
        // not one column per byte)
        let start_col = diagnostic.span.start as usize - line_start;
        let end_col = ((diagnostic.span.end as usize).min(line_start + line_text.len()))
            .saturating_sub(line_start)
            .max(start_col);
        let width = display_width(&line_text[start_col..end_col]).max(1);
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

/// line and col are 1-based
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

/// char count, so the caret lines up under multi-byte text
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

    // the caret row is measured in display columns, so a multi-byte char
    // gets one caret, not one per byte
    #[test]
    fn carets_count_characters_not_bytes() {
        let src = "let emoji = 😀\n";
        let span = Span::from_usize(12, 12 + '😀'.len_utf8());
        let diagnostics = vec![Diagnostic {
            message: "unexpected character `😀`".into(),
            span,
            help: None,
        }];
        let output = render("test.nex", src, &diagnostics);
        let caret_line = output
            .lines()
            .find(|line| line.contains('^'))
            .expect("a caret row");
        assert_eq!(caret_line.matches('^').count(), 1, "{output}");
    }

    // the leading pad is also display columns, so the caret lands under the
    // offending text even when earlier chars on the line are multi-byte
    #[test]
    fn caret_lines_up_after_multibyte_characters() {
        let src = "let café = @\n";
        let at = src.find('@').expect("an @ in the fixture");
        let diagnostics = vec![Diagnostic {
            message: "unexpected character `@`".into(),
            span: Span::from_usize(at, at + 1),
            help: None,
        }];
        let output = render("test.nex", src, &diagnostics);
        let caret_line = output
            .lines()
            .find(|line| line.contains('^'))
            .expect("a caret row");
        let source_line = output
            .lines()
            .find(|line| line.contains("let café"))
            .expect("the source row");

        // both rows share the `N | ` gutter, so the caret's column in the
        // caret row must equal the `@`'s column in the source row
        let caret_col = caret_line.chars().position(|c| c == '^').unwrap();
        let at_col = source_line.chars().position(|c| c == '@').unwrap();
        assert_eq!(caret_col, at_col, "{output}");
    }
}
