use nex_lexer::{tokenize, LexErrorKind, TokenKind};

#[test]
fn unknown_characters_are_skipped_and_reported() {
    let (tokens, errors) = tokenize("let @ x");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, LexErrorKind::UnknownChar('@'));
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Ident("x".into()));
}

#[test]
fn multiple_errors_are_all_reported() {
    let (_, errors) = tokenize("@ # \"unterminated");
    assert_eq!(errors.len(), 3);
    assert_eq!(errors[0].kind, LexErrorKind::UnknownChar('@'));
    assert_eq!(errors[1].kind, LexErrorKind::UnknownChar('#'));
    assert_eq!(errors[2].kind, LexErrorKind::UnterminatedString);
}

#[test]
fn error_spans_point_at_the_offending_text() {
    let src = "let @ x";
    let (_, errors) = tokenize(src);
    assert_eq!(errors[0].span.text(src), "@");
}

#[test]
fn a_long_run_of_invalid_characters_does_not_overflow_the_stack() {
    let src = "@".repeat(200_000);
    let (tokens, errors) = tokenize(&src);
    assert_eq!(tokens.len(), 1);
    assert_eq!(errors.len(), 200_000);
}

/// A tiny deterministic fuzzer: mutate a known-good program in many ways and
/// require that the lexer always terminates, never panics, and always emits a
/// final `Eof` token.
#[test]
fn lexer_never_panics_on_mutated_input() {
    let seed = include_str!("../../../examples/tour.nex");
    let bytes: Vec<char> = seed.chars().collect();
    let alphabet: Vec<char> = "abzZ019_\"\\'`@#$?~\n\t {}()[]<>+-*/%!=&|^.,:;"
        .chars()
        .collect();

    // xorshift so the corpus is reproducible without a dependency.
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = |bound: usize| -> usize {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };

    for _ in 0..2_000 {
        let mut chars = bytes.clone();
        for _ in 0..8 {
            if chars.is_empty() {
                break;
            }
            let idx = next(chars.len());
            match next(3) {
                0 => chars[idx] = alphabet[next(alphabet.len())],
                1 => {
                    chars.remove(idx);
                }
                _ => chars.insert(idx, alphabet[next(alphabet.len())]),
            }
        }
        let src: String = chars.into_iter().collect();
        let (tokens, _errors) = tokenize(&src);
        assert_eq!(
            tokens.last().map(|t| &t.kind),
            Some(&TokenKind::Eof),
            "token stream must always end in Eof"
        );
    }
}

#[test]
fn spans_are_always_within_bounds_and_ordered() {
    let src = include_str!("../../../examples/tour.nex");
    let (tokens, _) = tokenize(src);
    let mut previous_end = 0;
    for token in &tokens {
        assert!(token.span.start <= token.span.end, "{token:?}");
        assert!(token.span.end as usize <= src.len(), "{token:?}");
        assert!(
            token.span.start >= previous_end,
            "tokens must not overlap: {token:?}"
        );
        previous_end = token.span.start;
    }
}
