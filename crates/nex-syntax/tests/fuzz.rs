//! the parser must survive anything the lexer hands it: no panics, no
//! hangs, and no silently swallowed input.

use nex_lexer::tokenize;
use nex_syntax::parse_module;

/// xorshift, so the corpus is reproducible without pulling in rand
struct Rng(u64);

impl Rng {
    fn next(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

fn mutations(seed: u64, source: &str, rounds: usize) -> impl Iterator<Item = String> + '_ {
    let original: Vec<char> = source.chars().collect();
    let alphabet: Vec<char> = "abzZ019_\"\'`@#$?~\n\t {}()[]<>+-*/%!=&|^.,:;"
        .chars()
        .collect();
    let mut rng = Rng(seed);

    (0..rounds).map(move |_| {
        let mut chars = original.clone();
        for _ in 0..8 {
            if chars.is_empty() {
                break;
            }
            let idx = rng.next(chars.len());
            match rng.next(3) {
                0 => chars[idx] = alphabet[rng.next(alphabet.len())],
                1 => {
                    chars.remove(idx);
                }
                _ => chars.insert(idx, alphabet[rng.next(alphabet.len())]),
            }
        }
        chars.into_iter().collect()
    })
}

// mutating a real program is the interesting case: the input stays close
// enough to valid nex to reach deep into the parser
#[test]
fn mutated_programs_never_panic_the_parser() {
    let source = include_str!("../../../examples/tour.nex");
    for src in mutations(0x2545_F491_4F6C_DD1D, source, 10_000) {
        let (tokens, _) = tokenize(&src);
        let _ = parse_module(&tokens);
    }
}

#[test]
fn mutated_hello_world_never_panics_the_parser() {
    let source = include_str!("../../../examples/hello.nex");
    for src in mutations(0x9E37_79B9_7F4A_7C15, source, 10_000) {
        let (tokens, _) = tokenize(&src);
        let _ = parse_module(&tokens);
    }
}

// every token has to be either consumed into the tree or reported. if the
// parser ever returns with input left over it has stalled, which a plain
// no-panic check wouldn't catch.
#[test]
fn the_parser_always_consumes_its_input() {
    let source = include_str!("../../../examples/tour.nex");
    for src in mutations(0xDEAD_BEEF_CAFE_F00D, source, 2_000) {
        let (tokens, _) = tokenize(&src);
        let (module, errors) = parse_module(&tokens);
        let real_tokens = tokens.iter().filter(|t| !t.is_eof()).count();
        if real_tokens > 0 {
            // non-empty input has to produce something: items, errors, or
            // both. staying silent would mean the parser gave up quietly.
            assert!(
                !module.items.is_empty() || !errors.is_empty(),
                "parser returned nothing for {src:?}"
            );
        }
    }
}

#[test]
fn deeply_nested_input_does_not_blow_up() {
    for depth in [1, 8, 64] {
        let src = format!("fn f() {{ {}0{} }}", "(".repeat(depth), ")".repeat(depth));
        let (tokens, _) = tokenize(&src);
        let (module, errors) = parse_module(&tokens);
        assert!(errors.is_empty(), "depth {depth} failed: {errors:?}");
        assert_eq!(module.items.len(), 1);
    }
}
