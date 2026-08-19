use crate::span::Span;
use std::fmt;

/// Every distinct lexical unit Nex recognises.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // --- identifiers & literals ---
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),

    // --- keywords ---
    Fn,
    Let,
    Mut,
    If,
    Else,
    For,
    While,
    Return,
    Struct,
    Enum,
    Match,
    Use,
    Mod,
    Pub,
    True,
    False,
    In,
    Break,
    Continue,
    Const,
    Type,
    Impl,
    SelfValue,

    // --- arithmetic / bitwise / logical operators ---
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Shl,
    Shr,
    AmpAmp,
    PipePipe,
    Bang,

    // --- comparison ---
    Eq,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // --- compound assignment ---
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,

    // --- punctuation ---
    Arrow,
    FatArrow,
    Dot,
    DotDot,
    DotDotEq,
    ColonColon,
    Colon,
    Semi,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    /// End of input. Produced exactly once at the end of a token stream.
    Eof,
}

impl TokenKind {
    /// Maps an identifier spelling onto its keyword token, if it is one.
    pub fn keyword_from_str(word: &str) -> Option<TokenKind> {
        use TokenKind::*;
        Some(match word {
            "fn" => Fn,
            "let" => Let,
            "mut" => Mut,
            "if" => If,
            "else" => Else,
            "for" => For,
            "while" => While,
            "return" => Return,
            "struct" => Struct,
            "enum" => Enum,
            "match" => Match,
            "use" => Use,
            "mod" => Mod,
            "pub" => Pub,
            "true" => True,
            "false" => False,
            "in" => In,
            "break" => Break,
            "continue" => Continue,
            "const" => Const,
            "type" => Type,
            "impl" => Impl,
            "self" => SelfValue,
            _ => return None,
        })
    }

    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Fn | Let
                | Mut
                | If
                | Else
                | For
                | While
                | Return
                | Struct
                | Enum
                | Match
                | Use
                | Mod
                | Pub
                | True
                | False
                | In
                | Break
                | Continue
                | Const
                | Type
                | Impl
                | SelfValue
        )
    }

    pub fn is_literal(&self) -> bool {
        use TokenKind::*;
        matches!(self, Int(_) | Float(_) | Str(_) | True | False)
    }

    /// A short, human-readable name used in diagnostics ("expected `;`, found ...").
    pub fn describe(&self) -> &'static str {
        use TokenKind::*;
        match self {
            Ident(_) => "identifier",
            Int(_) => "integer literal",
            Float(_) => "float literal",
            Str(_) => "string literal",
            Fn => "`fn`",
            Let => "`let`",
            Mut => "`mut`",
            If => "`if`",
            Else => "`else`",
            For => "`for`",
            While => "`while`",
            Return => "`return`",
            Struct => "`struct`",
            Enum => "`enum`",
            Match => "`match`",
            Use => "`use`",
            Mod => "`mod`",
            Pub => "`pub`",
            True => "`true`",
            False => "`false`",
            In => "`in`",
            Break => "`break`",
            Continue => "`continue`",
            Const => "`const`",
            Type => "`type`",
            Impl => "`impl`",
            SelfValue => "`self`",
            Plus => "`+`",
            Minus => "`-`",
            Star => "`*`",
            Slash => "`/`",
            Percent => "`%`",
            Amp => "`&`",
            Pipe => "`|`",
            Caret => "`^`",
            Shl => "`<<`",
            Shr => "`>>`",
            AmpAmp => "`&&`",
            PipePipe => "`||`",
            Bang => "`!`",
            Eq => "`=`",
            EqEq => "`==`",
            BangEq => "`!=`",
            Lt => "`<`",
            LtEq => "`<=`",
            Gt => "`>`",
            GtEq => "`>=`",
            PlusEq => "`+=`",
            MinusEq => "`-=`",
            StarEq => "`*=`",
            SlashEq => "`/=`",
            Arrow => "`->`",
            FatArrow => "`=>`",
            Dot => "`.`",
            DotDot => "`..`",
            DotDotEq => "`..=`",
            ColonColon => "`::`",
            Colon => "`:`",
            Semi => "`;`",
            Comma => "`,`",
            LParen => "`(`",
            RParen => "`)`",
            LBrace => "`{`",
            RBrace => "`}`",
            LBracket => "`[`",
            RBracket => "`]`",
            Eof => "end of file",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        match self {
            Ident(name) => write!(f, "{name}"),
            Int(v) => write!(f, "{v}"),
            Float(v) => write!(f, "{v}"),
            Str(s) => write!(f, "{s:?}"),
            other => f.write_str(other.describe().trim_matches('`')),
        }
    }
}

/// A token: what it is, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip() {
        for word in [
            "fn", "let", "mut", "if", "else", "for", "while", "return", "struct", "enum", "match",
            "use", "mod", "pub", "true", "false", "in", "break", "continue", "const", "type",
            "impl", "self",
        ] {
            let kind = TokenKind::keyword_from_str(word)
                .unwrap_or_else(|| panic!("`{word}` should be a keyword"));
            assert!(kind.is_keyword(), "`{word}` should report as a keyword");
        }
    }

    #[test]
    fn non_keywords_are_none() {
        for word in ["main", "iff", "Self", "letx", "_", "returns"] {
            assert!(TokenKind::keyword_from_str(word).is_none(), "`{word}`");
        }
    }

    #[test]
    fn literals_report_as_literals() {
        assert!(TokenKind::Int(1).is_literal());
        assert!(TokenKind::Float(1.0).is_literal());
        assert!(TokenKind::Str("x".into()).is_literal());
        assert!(TokenKind::True.is_literal());
        assert!(!TokenKind::Ident("x".into()).is_literal());
    }
}
