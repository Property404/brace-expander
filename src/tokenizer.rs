use core::fmt;

use crate::{Options, error::Error};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TokenKind {
    Start { end: Option<usize> },
    End,
    Comma,
    Ellipses,
    Whitespace,
    Text,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct EscapeStr<'a> {
    interpret_backslashes: bool,
    raw: &'a str,
}

impl<'a> EscapeStr<'a> {
    pub(crate) fn new(raw: &'a str, options: &Options) -> Self {
        Self {
            interpret_backslashes: options.interpret_backslashes,
            raw,
        }
    }

    pub(crate) fn raw(&self) -> &'a str {
        self.raw
    }

    pub(crate) fn chars(&self) -> impl Iterator<Item = char> {
        let mut escaped = false;
        self.raw.chars().filter(move |c| {
            if escaped {
                escaped = false;
                true
            } else if self.interpret_backslashes && *c == '\\' {
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
    fn with_span<'a>(self, span: &'a str, end: usize, options: &Options) -> Token<'a> {
        debug_assert!(self.pos < end);
        Token {
            kind: self.kind,
            pos: self.pos,
            span: EscapeStr::new(&span[self.pos..end], options),
        }
    }
}

pub(crate) fn tokenize<'a>(input: &'a str, options: &Options) -> Result<Vec<Token<'a>>, Error> {
    let mut tokens = Vec::<Token>::new();
    let mut token: Option<PartialToken> = None;
    let mut brace_stack: Vec<usize> = Vec::new();

    let mut bytes = input.bytes().enumerate().peekable();
    while let Some((pos, c)) = bytes.next() {
        match c {
            b'{' => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos, options));
                }
                brace_stack.push(tokens.len());
                tokens.push(Token {
                    kind: TokenKind::Start { end: None },
                    pos,
                    span: EscapeStr::new(&input[pos..pos + 1], options),
                });
            }
            b'}' => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos, options));
                }
                if let Some(start_pos) = brace_stack.pop() {
                    let len = tokens.len();
                    assert!(start_pos < tokens.len());
                    if let Some(Token {
                        kind: TokenKind::Start { end },
                        ..
                    }) = tokens.get_mut(start_pos)
                    {
                        *end = Some(len);
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::End,
                    pos,
                    span: EscapeStr::new(&input[pos..pos + 1], options),
                });
            }
            b',' if !brace_stack.is_empty() => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos, options));
                }
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    pos,
                    span: EscapeStr::new(&input[pos..pos + 1], options),
                });
            }
            b'.' if !brace_stack.is_empty()
                && let Some((_, b'.')) = bytes.peek() =>
            {
                bytes.next();
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos, options));
                }
                tokens.push(Token {
                    kind: TokenKind::Ellipses,
                    pos,
                    span: EscapeStr::new(&input[pos..pos + 2], options),
                });
            }
            b'\n' | b'\t' | b' ' | b'\r' => {
                if let Some(token) = token.take_if(|token| token.kind != TokenKind::Whitespace) {
                    tokens.push(token.with_span(input, pos, options));
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
                if options.interpret_backslashes && c == b'\\' && bytes.next().is_none() {
                    return Err(Error::IncompleteEscape);
                }
                if let Some(token) = token.take_if(|token| token.kind != TokenKind::Text) {
                    tokens.push(token.with_span(input, pos, options));
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
        tokens.push(token.with_span(input, input.len(), options));
    }

    // Ok, yes, technically this is the tokenizer. But this was previously reported in the parser
    // so whatever, I don't care. It's a parse error. Fuck you
    if !options.ignore_parse_failures
        && let Some(idx) = brace_stack.pop()
        && let Some(token) = tokens.get(idx)
    {
        return Err(Error::with_context(
            "Start bracket is missing end bracket",
            token,
        ));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escape_backslashes() {
        let mut options = Options::new();
        options.interpret_backslashes = true;
        assert_eq!(EscapeStr::new("he\\llo", &options).to_string(), "hello");
        assert_eq!(EscapeStr::new("he\\\\llo", &options).to_string(), "he\\llo");

        options.interpret_backslashes = false;
        assert_eq!(EscapeStr::new("he\\llo", &options).to_string(), "he\\llo");
        assert_eq!(
            EscapeStr::new("he\\\\llo", &options).to_string(),
            "he\\\\llo"
        );
    }
}
