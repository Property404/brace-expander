use either::Either;
mod error;
use error::Error;

/// Brace expander
#[derive(Clone)]
pub struct BraceExpander {
    strict: bool,
}

impl Default for BraceExpander {
    fn default() -> Self {
        BraceExpander::new()
    }
}

impl BraceExpander {
    pub const fn new() -> Self {
        BraceExpander { strict: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TokenKind {
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
struct Token<'a> {
    kind: TokenKind,
    pos: usize,
    span: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
enum AstToken {
    Text(String),
    CommaExpansion(Vec<Vec<AstToken>>),
    NumericExpansion {
        start: i32,
        end: i32,
        // TODO: leading zeros
        // TODO: step
    }, // TODO: char expansion
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

impl BraceExpander {
    fn tokenize<'a>(&self, input: &'a str) -> Result<Vec<Token<'a>>, Error> {
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
                    brace_stack = brace_stack.wrapping_sub(1);
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
                    if let Some(token) = token.take_if(|token| token.kind != TokenKind::Whitespace)
                    {
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

        Ok(tokens)
    }

    fn parse_numeric_expansion<'a>(tokens: Vec<Token<'a>>) -> Result<AstToken, Error> {
        let mut tokens = tokens.into_iter();
        let Some(start) = tokens.next() else {
            return Err(Error::new("Expansion missing starting token"));
        };
        let Some(end) = tokens.next() else {
            return Err(Error::new("Numeric expansion missing end token"));
        };

        if tokens.next().is_some() {
            todo!("This is not yet supported");
        }

        Ok(AstToken::NumericExpansion {
            start: start.span.parse::<i32>().map_err(|_| {
                Error::with_context("Numeric expansion start token is not a number", start)
            })?,
            end: end.span.parse::<i32>().map_err(|_| {
                Error::with_context("Numeric expansion end token is not a number", end)
            })?,
        })
    }

    // Parse expansion
    fn parse_expansion<'a, T>(&self, input: &mut T) -> Result<AstToken, Error>
    where
        T: Iterator<Item = Token<'a>>,
    {
        let mut can_be_numeric = true;
        let mut found_comma = false;
        let mut numexp_ast = Vec::<Token>::new();
        let mut numexp_current: Option<Token> = None;

        let mut comexp_ast = Vec::<Vec<AstToken>>::new();
        let mut comexp_current = Vec::<AstToken>::new();

        while let Some(token) = input.next() {
            match token.kind {
                TokenKind::Start => {
                    can_be_numeric = false;
                    comexp_current.push(self.parse_expansion(input)?);
                }
                TokenKind::End => {
                    if found_comma {
                        comexp_ast.push(comexp_current);
                        return Ok(AstToken::CommaExpansion(comexp_ast));
                    }
                    if let Some(token) = numexp_current.take() {
                        numexp_ast.push(token);
                    }
                    if can_be_numeric {
                        match Self::parse_numeric_expansion(numexp_ast) {
                            Ok(val) => {
                                return Ok(val);
                            }
                            Err(err) => {
                                if self.strict {
                                    return Err(err);
                                }
                            }
                        }
                    }
                    break;
                }
                TokenKind::Comma => {
                    can_be_numeric = false;
                    found_comma = true;
                    // perf: make this clone a swap
                    comexp_ast.push(comexp_current.clone());
                    comexp_current.clear();
                }
                TokenKind::Text | TokenKind::Ellipses => {
                    if let Some(AstToken::Text(last)) = comexp_current.last_mut() {
                        // Merge text
                        *last += token.span;
                    } else {
                        comexp_current.push(AstToken::Text(token.span.into()));
                    }

                    if token.kind == TokenKind::Ellipses {
                        if let Some(token) = numexp_current.take() {
                            numexp_ast.push(token);
                        } else {
                            // Numeric expansion can't start with ellipses
                            can_be_numeric = false;
                        }
                    } else {
                        numexp_current = Some(token);
                    }
                }
                TokenKind::Whitespace => {
                    unreachable!("No whitespace allowed in this function");
                }
            }
        }
        if self.strict {
            return Err(Error::new("Failed to parse"));
        }
        todo!("Deal with inability to parse");
    }

    // Parse section without whitespace
    fn parse_section<'a, T>(&self, mut input: T) -> Result<Vec<AstToken>, Error>
    where
        T: Iterator<Item = Token<'a>>,
    {
        let mut ast = Vec::<AstToken>::new();
        while let Some(token) = input.next() {
            match token.kind {
                TokenKind::Start => {
                    ast.push(self.parse_expansion(&mut input)?);
                }
                TokenKind::Whitespace => {
                    unreachable!("Whitespace not allowed at this level");
                }
                _ => {
                    ast.push(AstToken::Text(token.span.into()));
                }
            }
        }
        Ok(ast)
    }

    pub(crate) fn expand_ast(&self, input: &[AstToken]) -> Vec<String> {
        let mut segments = vec![String::new()];
        for token in input.iter() {
            match token {
                AstToken::Text(text) => {
                    for segment in &mut segments {
                        *segment += text;
                    }
                }
                AstToken::NumericExpansion { start, end } => {
                    let mut new_segments = Vec::new();
                    let range = if start <= end {
                        Either::Left(*start..=*end)
                    } else {
                        Either::Right((*end..=*start).rev())
                    };
                    for segment in &segments {
                        for i in range.clone() {
                            let i = i.to_string();
                            new_segments.push(segment.clone() + &i);
                        }
                    }
                    std::mem::swap(&mut segments, &mut new_segments);
                }
                AstToken::CommaExpansion(asts) => {
                    let mut new_segments = Vec::new();
                    // Perf: just look at this mess
                    for segment in &segments {
                        for ast in asts {
                            let ast = self.expand_ast(ast);
                            for exp in &ast {
                                new_segments.push(segment.clone() + exp);
                            }
                        }
                    }
                    std::mem::swap(&mut segments, &mut new_segments);
                }
            }
        }

        segments
    }

    pub fn expand(&self, input: &str) -> Result<Vec<String>, Error> {
        let mut expansions = Vec::new();
        let tokens_barrel = self.tokenize(input)?;
        let tokens_barrel = tokens_barrel.split(|token| token.kind == TokenKind::Whitespace);

        for tokens in tokens_barrel {
            let tokens = tokens.iter().cloned();
            let ast = self.parse_section(tokens)?;
            expansions.extend(self.expand_ast(&ast));
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
        let be = BraceExpander::default();

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
    }
}
