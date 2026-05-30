use core::fmt;

use crate::error::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TokenKind {
    Start,
    End,
    Comma,
    Ellipses,
    Whitespace,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct EscapeStr<'a>(&'a str);

impl<'a> EscapeStr<'a> {
    pub(crate) fn raw(&self) -> &'a str {
        self.0
    }

    pub(crate) fn chars(&self) -> impl Iterator<Item = char> {
        let mut escaped = false;
        self.0.chars().filter(move |c| {
            if escaped {
                escaped = false;
                true
            } else if *c == '\\' {
                escaped = true;
                false
            } else {
                true
            }
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.chars().count()
    }
}

impl<'a> fmt::Display for EscapeStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl<'a> From<EscapeStr<'a>> for String {
    fn from(value: EscapeStr<'a>) -> Self {
        value.chars().collect()
    }
}

#[derive(Debug)]
struct PartialToken {
    kind: TokenKind,
    pos: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Token<'a> {
    pub kind: TokenKind,
    pub pos: usize,
    pub span: EscapeStr<'a>,
}

impl PartialToken {
    fn with_span<'a>(self, span: &'a str, end: usize) -> Token<'a> {
        debug_assert!(self.pos < end);
        Token {
            kind: self.kind,
            pos: self.pos,
            span: EscapeStr(&span[self.pos..end]),
        }
    }
}

pub(crate) fn tokenize<'a>(input: &'a str) -> Result<Vec<Token<'a>>, Error> {
    let mut tokens = Vec::<Token>::new();
    let mut token: Option<PartialToken> = None;
    let mut brace_stack: u32 = 0;

    let mut bytes = input.bytes().enumerate().peekable();
    while let Some((pos, c)) = bytes.next() {
        match c {
            b'{' => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos));
                }
                brace_stack += 1;
                tokens.push(Token {
                    kind: TokenKind::Start,
                    pos,
                    span: EscapeStr(&input[pos..pos + 1]),
                });
            }
            b'}' => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos));
                }
                brace_stack = brace_stack.saturating_sub(1);
                tokens.push(Token {
                    kind: TokenKind::End,
                    pos,
                    span: EscapeStr(&input[pos..pos + 1]),
                });
            }
            b',' if brace_stack > 0 => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos));
                }
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    pos,
                    span: EscapeStr(&input[pos..pos + 1]),
                });
            }
            b'.' if brace_stack > 0
                && let Some((_, b'.')) = bytes.peek() =>
            {
                bytes.next();
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos));
                }
                tokens.push(Token {
                    kind: TokenKind::Ellipses,
                    pos,
                    span: EscapeStr(&input[pos..pos + 2]),
                });
            }
            b'\n' | b'\t' | b' ' | b'\r' => {
                if let Some(token) = token.take_if(|token| token.kind != TokenKind::Whitespace) {
                    tokens.push(token.with_span(input, pos));
                }
                if token.is_none() {
                    token = Some(PartialToken {
                        kind: TokenKind::Whitespace,
                        pos,
                    });
                }
            }
            _ => {
                // Backslash escapes so we skip interpreting the next char
                if c == b'\\' && bytes.next().is_none() {
                    return Err(Error::IncompleteEscape);
                }
                if let Some(token) = token.take_if(|token| token.kind != TokenKind::Text) {
                    tokens.push(token.with_span(input, pos));
                }
                if token.is_none() {
                    token = Some(PartialToken {
                        kind: TokenKind::Text,
                        pos,
                    });
                }
            }
        }
    }

    if let Some(token) = token.take() {
        tokens.push(token.with_span(input, input.len()));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escape_backslashes() {
        assert_eq!(EscapeStr("he\\llo").to_string(), "hello");
        assert_eq!(EscapeStr("he\\\\llo").to_string(), "he\\llo");
    }
}
