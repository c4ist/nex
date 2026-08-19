use crate::span::Span;
use std::fmt;

/// What went wrong while scanning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    /// A character that cannot begin any token.
    UnknownChar(char),
    /// A string literal that ran to end of line or end of file.
    UnterminatedString,
    /// `\q` and friends.
    InvalidEscape(char),
    /// `\x` not followed by two hex digits.
    InvalidHexEscape,
    /// `0x` / `0b` with no digits, or `1.` followed by a non-digit.
    MalformedNumber,
    /// An integer literal too large for `i64`.
    IntegerOverflow,
    /// A float literal that could not be parsed.
    InvalidFloat,
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::UnknownChar(c) => write!(f, "unexpected character `{c}`"),
            LexErrorKind::UnterminatedString => write!(f, "unterminated string literal"),
            LexErrorKind::InvalidEscape(c) => write!(f, "unknown escape sequence `\\{c}`"),
            LexErrorKind::InvalidHexEscape => {
                write!(f, "`\\x` must be followed by exactly two hex digits")
            }
            LexErrorKind::MalformedNumber => write!(f, "malformed numeric literal"),
            LexErrorKind::IntegerOverflow => {
                write!(f, "integer literal is too large for `i64`")
            }
            LexErrorKind::InvalidFloat => write!(f, "invalid float literal"),
        }
    }
}

/// A recoverable lexing error. The lexer never stops on one of these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

impl LexError {
    pub fn new(kind: LexErrorKind, span: Span) -> Self {
        LexError { kind, span }
    }

    /// A short hint shown under the offending span.
    pub fn help(&self) -> Option<&'static str> {
        match self.kind {
            LexErrorKind::UnterminatedString => Some("add a closing `\"`"),
            LexErrorKind::InvalidEscape(_) => {
                Some("valid escapes are \\n \\r \\t \\0 \\\\ \\\" and \\xNN")
            }
            LexErrorKind::MalformedNumber => {
                Some("a digit must follow the prefix or decimal point")
            }
            LexErrorKind::IntegerOverflow => Some("the maximum is 9223372036854775807"),
            _ => None,
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {:?}", self.kind, self.span)
    }
}

impl std::error::Error for LexError {}
