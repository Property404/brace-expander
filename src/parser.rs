use crate::{
    Options,
    error::Error,
    tokenizer::{Token, TokenKind},
};
use either::Either;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AstToken {
    Text(String),
    CommaExpansion(Vec<Vec<AstToken>>),
    NumericExpansion {
        start: i32,
        end: i32,
        step: u16,
        leading_zeros: usize,
    },
    CharExpansion {
        start: u8,
        end: u8,
        step: u16,
    },
}

fn parse_as_char_or_int<'a>(token: &Token<'a>) -> Result<Either<u8, (usize, i32)>, Error> {
    if let Ok(number) = token.span.raw().parse::<i32>() {
        return Ok(Either::Right((
            token
                .span
                .chars()
                .take(token.span.len() - 1)
                .take_while(|c| *c == '0')
                .count(),
            number,
        )));
    }
    if token.span.len() == 1
        && let Some(Ok(c)) = token.span.chars().next().map(u8::try_from)
        && c.is_ascii_alphabetic()
    {
        return Ok(Either::Left(c));
    }

    Err(Error::with_context(
        "Expansion expected either number of character",
        token,
    ))
}

fn parse_numeric_expansion<'a>(tokens: Vec<Token<'a>>) -> Result<AstToken, Error> {
    let mut tokens = tokens.iter();
    let Some(start) = tokens.next() else {
        return Err(Error::new("Ellipses expansion missing start token"));
    };
    let Some(end) = tokens.next() else {
        return Err(Error::new("Ellipses expansion missing end token"));
    };
    // Optional step-by token
    let mut step = tokens
        .next()
        .map(|token| {
            token
                .span
                .raw()
                .parse::<i16>()
                .map_err(|_| {
                    Error::with_context("Ellipses expansion step-by must be an integer", token)
                    // Why is this unsigned_abs here? Because bash allows negative steps, but they're
                    // treated the same as positive steps
                })
                .map(i16::unsigned_abs)
        })
        .transpose()?
        .unwrap_or(1);
    // Bash treats '0' as '1' for some reason, so we will as well
    if step == 0 {
        step = 1
    }

    // We max out at three tokens
    if let Some(extraneous) = tokens.next() {
        return Err(Error::with_context(
            "Ellipses expansion has too many tokens",
            extraneous,
        ));
    }

    match (parse_as_char_or_int(start)?, parse_as_char_or_int(end)?) {
        (Either::Left(start), Either::Left(end)) => {
            Ok(AstToken::CharExpansion { start, end, step })
        }
        (Either::Right((start_pad, start)), Either::Right((end_pad, end))) => {
            Ok(AstToken::NumericExpansion {
                start,
                end,
                step,
                leading_zeros: start_pad.max(end_pad),
            })
        }
        _ => Err(Error::with_context(
            "Ellipses expansion start/end mismatch",
            end,
        )),
    }
}

// Parse expansion
fn parse_expansion<'a>(
    mut input: &'a [Token<'a>],
    options: &Options,
) -> Result<(&'a [Token<'a>], AstToken), Error> {
    let original_input = input;
    let mut can_be_numeric = true;
    let mut found_comma = false;
    let mut numexp_ast = Vec::<Token>::new();
    let mut numexp_current: Option<&Token> = None;

    let mut comexp_ast = Vec::<Vec<AstToken>>::new();
    let mut comexp_current = Vec::<AstToken>::new();

    while let Some(token) = input.first() {
        input = &input[1..];

        match token.kind {
            TokenKind::Start => {
                can_be_numeric = false;
                match parse_expansion(input, options) {
                    Ok((new_input, new_ast)) => {
                        comexp_current.push(new_ast);
                        input = new_input;
                    }
                    Err(err) => {
                        if options.strict {
                            return Err(err);
                        }
                        unreachable!(
                            "Can't return errors if we don't return errors (taps forehead)"
                        );
                    }
                }
            }
            TokenKind::End => {
                if found_comma {
                    comexp_ast.push(comexp_current);
                    return Ok((input, AstToken::CommaExpansion(comexp_ast)));
                }
                if let Some(token) = numexp_current.take() {
                    numexp_ast.push(token.clone());
                }
                if can_be_numeric {
                    match parse_numeric_expansion(numexp_ast) {
                        Ok(val) => {
                            return Ok((input, val));
                        }
                        Err(err) => {
                            if options.strict {
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
                let mut swap = Vec::new();
                std::mem::swap(&mut swap, &mut comexp_current);
                comexp_ast.push(swap);
            }
            TokenKind::Text | TokenKind::Ellipses => {
                if let Some(AstToken::Text(last)) = comexp_current.last_mut() {
                    // Merge text
                    last.extend(token.span.chars());
                } else {
                    comexp_current.push(AstToken::Text(token.span.into()));
                }

                if token.kind == TokenKind::Ellipses {
                    if let Some(token) = numexp_current.take() {
                        numexp_ast.push(token.clone());
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
    if options.strict {
        return Err(Error::new("Failed to parse"));
    }

    Ok((original_input, AstToken::Text("{".into())))
}

// Parse section without whitespace
pub(crate) fn parse_section<'a>(
    mut input: &'a [Token<'a>],
    options: &Options,
) -> Result<Vec<AstToken>, Error> {
    let mut ast = Vec::<AstToken>::new();
    while let Some(token) = input.first() {
        input = &input[1..];

        match token.kind {
            TokenKind::Start => match parse_expansion(input, options) {
                Ok((new_input, ast_token)) => {
                    ast.push(ast_token);
                    input = new_input;
                }
                Err(err) => {
                    if options.strict {
                        return Err(err);
                    }
                    unreachable!("Inner shouldn't error on non-strict mode");
                }
            },
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
