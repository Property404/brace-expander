//! Library to support bash-like brace expansions
//!
//! [![Repository](https://img.shields.io/badge/github-brace--expander-/)](https://github.com/Property404/brace-expander)
//! [![crates.io](https://img.shields.io/crates/v/brace-expander.svg)](https://crates.io/crates/brace-expander)
//! [![Documentation](https://docs.rs/brace-expander/badge.svg)](https://docs.rs/brace-expander)
//!
//! ## Examples
//!
//! ```rust
//! use brace_expander::BraceExpander;
//! let be = BraceExpander::default();
//!
//! // Basic cartesian product
//! assert_eq!(be.expand("{a,b}").unwrap().join(" "), "a b");
//! assert_eq!(be.expand("hello_{a,b}").unwrap().join(" "), "hello_a hello_b");
//! assert_eq!(be.expand("G{o,u}{b,g}").unwrap().join(" "), "Gob Gog Gub Gug");
//!
//! // Nested product
//! assert_eq!(be
//!     .expand("{{a,b}{c,{e,f}},g}")
//!     .unwrap()
//!     .join(" "), "ac ae af bc be bf g");
//!
//! // Numeric expansion
//! assert_eq!(be.expand("thing{1..3}").unwrap().join(" "), "thing1 thing2 thing3");
//!
//! // Or backwards
//! assert_eq!(be.expand("thing{3..1}").unwrap().join(" "), "thing3 thing2 thing1");
//!
//! // char expansion
//! assert_eq!(be.expand("{A..C}").unwrap().join(" "), "A B C");
//!
//! // Step by
//! assert_eq!(be.expand("{A..E..2}").unwrap().join(" "), "A C E");
//!
//! // Leading zeroes
//! assert_eq!(be.expand("Agent{006..008}").unwrap().join(" "), "Agent006 Agent007 Agent008");
//!
//! ```
//!
//! ## License
//!
//! MIT or Apache-2.0
//!
//! Pull requests encouraged
#![warn(missing_docs)]
#![forbid(unsafe_code)]
use either::Either;
mod error;
use error::Error;
mod tokenizer;
use tokenizer::{Token, TokenKind};
mod parser;
use parser::AstToken;

#[derive(Debug, Clone)]
struct Options {
    strict: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { strict: true }
    }
}

/// Brace expander
#[derive(Clone, Default)]
pub struct BraceExpander {
    options: Options,
}

// Perf: cartesian product is probably the biggest bottleneck
fn cartesian_product<B>(list_a: Vec<String>, list_b: &[B]) -> Vec<String>
where
    B: AsRef<str>,
{
    list_a
        .into_iter()
        .flat_map(|s| std::iter::repeat_n(s, list_b.len()).zip(list_b.iter()))
        .map(|(mut a, b)| {
            a += b.as_ref();
            a
        })
        .collect()
}

pub(crate) fn expand_ast(input: &[AstToken]) -> Vec<String> {
    let mut segments = vec![String::new()];
    for token in input.iter() {
        match token {
            AstToken::Text(text) => {
                segments = cartesian_product(segments, &[text]);
            }
            AstToken::NumericExpansion {
                start,
                end,
                step,
                leading_zeros,
            } => {
                let leading_zeroes = "0".repeat(*leading_zeros);
                let range = if start <= end {
                    Either::Left(*start..=*end)
                } else {
                    Either::Right((*end..=*start).rev())
                }
                .step_by(usize::from(*step))
                .map(|int| format!("{leading_zeroes}{int}"))
                .collect::<Vec<_>>();
                segments = cartesian_product(segments, &range);
            }
            AstToken::CharExpansion { start, end, step } => {
                let range = if start <= end {
                    Either::Left(*start..=*end)
                } else {
                    Either::Right((*end..=*start).rev())
                }
                .step_by(usize::from(*step))
                .map(char::from)
                // Bash replaces backslash with space in char expansions
                .map(|c| if c == '\\' { ' ' } else { c })
                .map(|c| c.to_string())
                .collect::<Vec<_>>();
                segments = cartesian_product(segments, &range);
            }
            AstToken::CommaExpansion(asts) => {
                segments = cartesian_product(
                    segments,
                    &asts.iter().flat_map(|v| expand_ast(v)).collect::<Vec<_>>(),
                );
            }
        }
    }

    segments
}

impl BraceExpander {
    /// Ignore parse failures instead of erroring out, making the parsing stage infallible. This is
    /// how Bash behaves.
    ///
    /// Default: `false`
    pub fn ignore_parse_failures(mut self, ignore_parse_failures: bool) -> Self {
        self.options.strict = !ignore_parse_failures;
        self
    }

    /// Expand a string
    pub fn expand(&self, input: &str) -> Result<Vec<String>, Error> {
        let mut expansions = Vec::new();
        let tokens_barrel = tokenizer::tokenize(input)?;
        let tokens_barrel = tokens_barrel.split(|token| token.kind == TokenKind::Whitespace);

        for tokens in tokens_barrel {
            if !tokens.is_empty() {
                let ast = parser::parse_section(tokens, &self.options)?;
                // Perf note: expansion takes MUCH longer than tokenization or parsing
                // Start here for perf improvements
                expansions.extend(expand_ast(&ast));
            }
        }

        Ok(expansions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tv(bc: &BraceExpander, input: &str, expected: &[&'static str]) {
        let actual = bc.expand(input).unwrap();
        let expected = expected
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn expand() {
        let be = BraceExpander::default().ignore_parse_failures(false);
        test_tv(&be, "a", &["a"]);
        test_tv(&be, "a,4", &["a,4"]);
        test_tv(&be, "a..4", &["a..4"]);
        test_tv(&be, "{1..4}", &["1", "2", "3", "4"]);
        test_tv(&be, "a{1..4}", &["a1", "a2", "a3", "a4"]);
        test_tv(&be, "a{1..4}b", &["a1b", "a2b", "a3b", "a4b"]);
        test_tv(&be, "{1..2}{1..2}", &["11", "12", "21", "22"]);
        test_tv(&be, "a{1..2}{1..2}b", &["a11b", "a12b", "a21b", "a22b"]);
        test_tv(&be, "{a,b}{c,d}", &["ac", "ad", "bc", "bd"]);
        test_tv(&be, "{_{a,b}_,c}", &["_a_", "_b_", "c"]);
        test_tv(&be, "{_{1..3}_,c}", &["_1_", "_2_", "_3_", "c"]);
        test_tv(&be, "{1..1}", &["1"]);
        test_tv(&be, "{3..1}", &["3", "2", "1"]);
        test_tv(&be, "{3..1}", &["3", "2", "1"]);
        test_tv(&be, "a{,,}", &["a", "a", "a"]);
        test_tv(&be, "{1..2}{,}", &["1", "1", "2", "2"]);
        test_tv(&be, "{,}{1..2}", &["1", "2", "1", "2"]);
        test_tv(&be, "{a,}{1..2}", &["a1", "a2", "1", "2"]);
        test_tv(&be, "a b", &["a", "b"]);
        test_tv(&be, "{1..2} {a,b}c", &["1", "2", "ac", "bc"]);
        test_tv(&be, "", &[]);
        test_tv(&be, "  ", &[]);
        test_tv(&be, "a  ", &["a"]);
        test_tv(&be, "} {1..2}", &["}", "1", "2"]);
        test_tv(&be, "{1..2}}", &["1}", "2}"]);
        test_tv(&be, "{1..2}..", &["1..", "2.."]);
        test_tv(&be, "{1..3..1}", &["1", "2", "3"]);
        test_tv(&be, "{1..3..0}", &["1", "2", "3"]);
        test_tv(&be, "{1..3..-1}", &["1", "2", "3"]);
        test_tv(&be, "{1..3..2}", &["1", "3"]);
        test_tv(&be, "{1..3..3}", &["1"]);
        test_tv(&be, "{3..1..3}", &["3"]);
        test_tv(&be, "{1..03..2}", &["01", "03"]);
        test_tv(&be, "{01..03..2}", &["01", "03"]);
        test_tv(&be, "{01..00003..2}", &["00001", "00003"]);
        test_tv(&be, "{1..3..002}", &["1", "3"]);
        test_tv(&be, "{a..c}", &["a", "b", "c"]);
        test_tv(&be, "{a..c..2}", &["a", "c"]);
        test_tv(&be, "{c..a..2}", &["c", "a"]);
        test_tv(&be, "{a..Z..2}", &["a", "_", "]", "["]);
        test_tv(&be, "{Z..a..2}", &["Z", " ", "^", "`"]);
        test_tv(&be, "{Z..a..2}", &["Z", " ", "^", "`"]);
        test_tv(&be, "{0..-2}", &["0", "-1", "-2"]);
        test_tv(&be, "{-1..-2}", &["-1", "-2"]);
        test_tv(&be, "{-1..1}", &["-1", "0", "1"]);
        test_tv(&be, "{1..-1}", &["1", "0", "-1"]);
        test_tv(&be, "{-1..+1}", &["-1", "0", "1"]);
        test_tv(
            &be,
            "{a,b}{c,d}{e,f}",
            &["ace", "acf", "ade", "adf", "bce", "bcf", "bde", "bdf"],
        );
        test_tv(&be, "\\{a,b}", &["{a,b}"]);
        test_tv(&be, "\\\\", &["\\"]);
    }

    #[test]
    fn parse_failures() {
        let tvs: &[(&str, &[&str])] = &[
            ("{a..", &["{a.."]),
            ("{", &["{"]),
            ("{a}", &["{a}"]),
            ("{a}{b,c}", &["{a}b", "{a}c"]),
            ("{1..2..z}", &["{1..2..z}"]),
            ("{1..z}", &["{1..z}"]),
            ("{1..z}{,}", &["{1..z}", "{1..z}"]),
            ("{1..}", &["{1..}"]),
            ("{..}", &["{..}"]),
            ("{{{", &["{{{"]),
            ("{,{,{", &["{,{,{"]),
        ];

        // When strict mode is false, we ignore parse failures like bash
        let be = BraceExpander::default().ignore_parse_failures(true);
        for tv in tvs {
            test_tv(&be, tv.0, tv.1);
        }

        // When strict mode is on, we error out
        let be = BraceExpander::default().ignore_parse_failures(false);
        for tv in tvs {
            assert!(be.expand(tv.0).is_err());
        }
    }

    #[test]
    fn fuzz() {
        use rand::prelude::*;
        let mut rng = rand::rng();
        fn build_fuzzy_string(rng: &mut rand::rngs::ThreadRng) -> String {
            let length = rng.random_range(0..20);
            let mut string = String::new();
            for _ in 0..length {
                let chars = [
                    '\\',
                    '{',
                    '}',
                    '.',
                    ',',
                    '\'',
                    '"',
                    ' ',
                    '\t',
                    '\r',
                    '\n',
                    rng.random(),
                ];
                string.push(*chars.choose(rng).unwrap());
            }
            string
        }

        let strict_be = BraceExpander::default().ignore_parse_failures(false);
        let loose_be = BraceExpander::default().ignore_parse_failures(true);
        for _ in 0..1000 {
            let string = build_fuzzy_string(&mut rng);

            // BraceExpander in loose mood shouldn't error at all, even on garbage
            if let Err(err @ Error::ParserError { .. }) = loose_be.expand(&string) {
                panic!("Unexpected error on `{string}`: {err}");
            }

            // Just make sure we don't panic on strict
            let _ = strict_be.expand(&string);
        }
    }
}
