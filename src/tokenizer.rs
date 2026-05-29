#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TokenKind {
    Start,
    End,
    Comma,
    Ellipses,
    Whitespace,
    Text,
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
    pub span: &'a str,
}

impl PartialToken {
    fn with_span<'a>(self, span: &'a str, end: usize) -> Token<'a> {
        debug_assert!(self.pos < end);
        Token {
            kind: self.kind,
            pos: self.pos,
            span: &span[self.pos..end],
        }
    }
}

pub(crate) fn tokenize<'a>(input: &'a str) -> Vec<Token<'a>> {
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
                    span: &input[pos..pos + 1],
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
                    span: &input[pos..pos + 1],
                });
            }
            b',' if brace_stack > 0 => {
                if let Some(token) = token.take() {
                    tokens.push(token.with_span(input, pos));
                }
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    pos,
                    span: &input[pos..pos + 1],
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
                    span: &input[pos..pos + 2],
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
                if c == b'\\' {
                    bytes.next();
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

    tokens
}
