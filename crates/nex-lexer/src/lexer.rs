use crate::error::{LexError, LexErrorKind};
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// A hand-written scanner over a UTF-8 source string.
///
/// The lexer is infallible in the sense that it never panics and never stops
/// early: recoverable problems are pushed onto [`Lexer::errors`] and scanning
/// continues so that a single run reports every lexical error in a file.
pub struct Lexer<'src> {
    src: &'src str,
    /// Byte offset of the next character to be consumed.
    pos: usize,
    /// Set once [`TokenKind::Eof`] has been produced.
    finished: bool,
    errors: Vec<LexError>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            src,
            pos: 0,
            finished: false,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &[LexError] {
        &self.errors
    }

    pub fn into_errors(self) -> Vec<LexError> {
        self.errors
    }

    // ---------------------------------------------------------------- cursor

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(offset)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Consumes the next character if it equals `expected`.
    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn error(&mut self, kind: LexErrorKind, span: Span) {
        self.errors.push(LexError::new(kind, span));
    }

    // ------------------------------------------------------------ whitespace

    /// Skips whitespace and `//` line comments until real content is reached.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => return,
            }
        }
    }

    // ---------------------------------------------------------------- tokens

    /// Produces the next token. Returns `None` only after [`TokenKind::Eof`]
    /// has already been handed out.
    pub fn next_token(&mut self) -> Option<Token> {
        if self.finished {
            return None;
        }

        // Loops (rather than recurses) so that a long run of invalid characters
        // cannot exhaust the stack.
        loop {
            self.skip_trivia();

            if self.at_end() {
                self.finished = true;
                let end = self.src.len();
                return Some(Token::new(TokenKind::Eof, Span::from_usize(end, end)));
            }

            let start = self.pos;
            let c = self.bump().expect("not at end");

            let kind = match c {
                c if is_ident_start(c) => self.ident_or_keyword(start),
                c if c.is_ascii_digit() => self.number(start, c),
                '"' => self.string(start),
                _ => match self.operator(c) {
                    Some(kind) => kind,
                    None => {
                        let span = Span::from_usize(start, self.pos);
                        self.error(LexErrorKind::UnknownChar(c), span);
                        // Skip the offending character and keep scanning.
                        continue;
                    }
                },
            };

            return Some(Token::new(kind, Span::from_usize(start, self.pos)));
        }
    }

    fn ident_or_keyword(&mut self, start: usize) -> TokenKind {
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                self.bump();
            } else {
                break;
            }
        }
        let word = &self.src[start..self.pos];
        TokenKind::keyword_from_str(word).unwrap_or_else(|| TokenKind::Ident(word.to_string()))
    }

    // --------------------------------------------------------------- numbers

    fn number(&mut self, start: usize, first: char) -> TokenKind {
        if first == '0' {
            match self.peek() {
                Some('x') | Some('X') => {
                    self.bump();
                    return self.radix_number(start, 16);
                }
                Some('b') | Some('B') => {
                    self.bump();
                    return self.radix_number(start, 2);
                }
                Some('o') | Some('O') => {
                    self.bump();
                    return self.radix_number(start, 8);
                }
                _ => {}
            }
        }

        self.eat_digits(10);

        let mut is_float = false;
        // Where the parseable numeric text ends. Normally this is the cursor,
        // but a malformed literal may consume trailing characters that must not
        // be handed to `parse`.
        let mut text_end: Option<usize> = None;

        // A `.` only starts a fraction when a digit follows; `0..10` must stay
        // an integer followed by `..`.
        if self.peek() == Some('.') {
            match self.peek_at(1) {
                Some(d) if d.is_ascii_digit() => {
                    is_float = true;
                    self.bump();
                    self.eat_digits(10);
                }
                Some('.') => {}
                Some(c) if is_ident_start(c) => {
                    // `1.foo` — there is no method-call-on-literal syntax yet.
                    // Swallow the dot so the parser is not also confused by it,
                    // but keep it out of the value we parse.
                    text_end = Some(self.pos);
                    self.bump();
                    let span = Span::from_usize(start, self.pos);
                    self.error(LexErrorKind::MalformedNumber, span);
                }
                _ => {}
            }
        }

        if matches!(self.peek(), Some('e') | Some('E')) {
            let exp_start = self.pos;
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            if matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                is_float = true;
                self.eat_digits(10);
            } else {
                // Not an exponent after all (e.g. `1e` or `2 else`): rewind.
                self.pos = exp_start;
            }
        }

        let value_end = text_end.unwrap_or(self.pos);
        let text: String = self.src[start..value_end]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        let span = Span::from_usize(start, self.pos);

        if is_float {
            match text.parse::<f64>() {
                Ok(v) => TokenKind::Float(v),
                Err(_) => {
                    self.error(LexErrorKind::InvalidFloat, span);
                    TokenKind::Float(0.0)
                }
            }
        } else {
            match text.parse::<i64>() {
                Ok(v) => TokenKind::Int(v),
                Err(_) => {
                    self.error(LexErrorKind::IntegerOverflow, span);
                    TokenKind::Int(0)
                }
            }
        }
    }

    /// Scans the digits of an already-consumed `0x` / `0b` / `0o` prefix.
    fn radix_number(&mut self, start: usize, radix: u32) -> TokenKind {
        let digits_start = self.pos;
        self.eat_digits(radix);
        let digits: String = self.src[digits_start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        let span = Span::from_usize(start, self.pos);

        if digits.is_empty() {
            self.error(LexErrorKind::MalformedNumber, span);
            return TokenKind::Int(0);
        }

        match i64::from_str_radix(&digits, radix) {
            Ok(v) => TokenKind::Int(v),
            Err(_) => {
                self.error(LexErrorKind::IntegerOverflow, span);
                TokenKind::Int(0)
            }
        }
    }

    /// Consumes digits valid in `radix`, allowing `_` separators.
    fn eat_digits(&mut self, radix: u32) {
        while let Some(c) = self.peek() {
            if c == '_' || c.is_digit(radix) {
                self.bump();
            } else {
                break;
            }
        }
    }

    // --------------------------------------------------------------- strings

    fn string(&mut self, start: usize) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    let span = Span::from_usize(start, self.pos);
                    self.error(LexErrorKind::UnterminatedString, span);
                    return TokenKind::Str(value);
                }
                Some('"') => {
                    self.bump();
                    return TokenKind::Str(value);
                }
                Some('\\') => {
                    let esc_start = self.pos;
                    self.bump();
                    self.escape(esc_start, &mut value);
                }
                Some(c) => {
                    self.bump();
                    value.push(c);
                }
            }
        }
    }

    /// Handles the character(s) after a `\` inside a string literal.
    fn escape(&mut self, esc_start: usize, out: &mut String) {
        let Some(c) = self.bump() else {
            return;
        };
        match c {
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '0' => out.push('\0'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            'x' => {
                let hi = self.peek().filter(|c| c.is_ascii_hexdigit());
                let lo = self.peek_at(1).filter(|c| c.is_ascii_hexdigit());
                match (hi, lo) {
                    (Some(hi), Some(lo)) => {
                        self.bump();
                        self.bump();
                        let code = (hi.to_digit(16).unwrap() * 16 + lo.to_digit(16).unwrap()) as u8;
                        out.push(code as char);
                    }
                    _ => {
                        let span = Span::from_usize(esc_start, self.pos);
                        self.error(LexErrorKind::InvalidHexEscape, span);
                    }
                }
            }
            other => {
                let span = Span::from_usize(esc_start, self.pos);
                self.error(LexErrorKind::InvalidEscape(other), span);
                out.push(other);
            }
        }
    }

    // ------------------------------------------------------------- operators

    /// Matches operators and punctuation, longest form first.
    fn operator(&mut self, c: char) -> Option<TokenKind> {
        use TokenKind::*;
        Some(match c {
            '+' => {
                if self.eat('=') {
                    PlusEq
                } else {
                    Plus
                }
            }
            '-' => {
                if self.eat('=') {
                    MinusEq
                } else if self.eat('>') {
                    Arrow
                } else {
                    Minus
                }
            }
            '*' => {
                if self.eat('=') {
                    StarEq
                } else {
                    Star
                }
            }
            '/' => {
                if self.eat('=') {
                    SlashEq
                } else {
                    Slash
                }
            }
            '%' => Percent,
            '&' => {
                if self.eat('&') {
                    AmpAmp
                } else {
                    Amp
                }
            }
            '|' => {
                if self.eat('|') {
                    PipePipe
                } else {
                    Pipe
                }
            }
            '^' => Caret,
            '!' => {
                if self.eat('=') {
                    BangEq
                } else {
                    Bang
                }
            }
            '=' => {
                if self.eat('=') {
                    EqEq
                } else if self.eat('>') {
                    FatArrow
                } else {
                    Eq
                }
            }
            '<' => {
                if self.eat('=') {
                    LtEq
                } else if self.eat('<') {
                    Shl
                } else {
                    Lt
                }
            }
            '>' => {
                if self.eat('=') {
                    GtEq
                } else if self.eat('>') {
                    Shr
                } else {
                    Gt
                }
            }
            '.' => {
                if self.eat('.') {
                    if self.eat('=') {
                        DotDotEq
                    } else {
                        DotDot
                    }
                } else {
                    Dot
                }
            }
            ':' => {
                if self.eat(':') {
                    ColonColon
                } else {
                    Colon
                }
            }
            ';' => Semi,
            ',' => Comma,
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            _ => return None,
        })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.next_token()
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}
