use super::{Result, Token};
use crate::error::{NoValidTokenError, UnexpectedTokenError};
use logos::{Lexer, Logos, Span, SpannedIter};
use std::borrow::Cow;

/// Kinds of item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item<'a> {
    /// A statement, the ones starting with #.
    Statement { content: Cow<'a, str>, span: Span },

    /// A value.
    Item { content: Cow<'a, str>, span: Span },
}

impl<'a> Item<'a> {
    pub fn span(&self) -> Span {
        match self {
            Item::Statement { span, .. } => span.clone(),
            Item::Item { span, .. } => span.clone(),
        }
    }

    pub fn into_content(self) -> Cow<'a, str> {
        match self {
            Item::Statement { content, .. } => content,
            Item::Item { content, .. } => content,
        }
    }
}

/// Reader event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event<'a> {
    /// A group with the given name is starting.
    GroupStart { name: Cow<'a, str>, span: Span },

    /// A group has ended.
    GroupEnd { span: Span },

    /// An entry.
    Entry {
        key: Item<'a>,
        value: Item<'a>,
        span: Span,
    },
}

impl Event<'_> {
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Event::GroupStart { span, .. } => span.clone(),
            Event::GroupEnd { span, .. } => span.clone(),
            Event::Entry { span, .. } => span.clone(),
        }
    }
}

/// A VDF token reader.
pub struct Reader<'a> {
    pub(crate) content: &'a str,
    lexer: SpannedIter<'a, Token>,
    peeked: Option<(Result<Token, <Token as Logos<'a>>::Error>, Span)>,
}

impl<'a> From<&'a str> for Reader<'a> {
    fn from(content: &'a str) -> Self {
        Reader {
            content,
            lexer: Lexer::new(content).spanned(),
            peeked: None,
        }
    }
}

impl<'a> Reader<'a> {
    fn token(&mut self) -> Option<(Result<Token, <Token as Logos>::Error>, Span)> {
        if let Some((token, span)) = self.peeked.take() {
            Some((token, span))
        } else {
            self.lexer.next()
        }
    }

    fn peek(&mut self) -> Option<(Result<Token, <Token as Logos>::Error>, Span)> {
        if self.peeked.is_none() {
            self.peeked = self.lexer.next();
        }
        self.peeked.clone()
    }

    fn token_eat_newlines(&mut self) -> Option<(Result<Token, <Token as Logos>::Error>, Span)> {
        loop {
            let (token, span) = self.token()?;
            match token {
                Err(e) => return Some((Err(e), span)),
                Ok(Token::NewLine) => {
                    continue;
                }
                Ok(token) => return Some((Ok(token), span)),
            }
        }
    }

    /// Get the next event, this does copies.
    #[allow(dead_code)]
    pub fn event(&mut self) -> Option<Result<Event<'a>>> {
        const VALID_KEY: &[Token] = &[
            Token::Item,
            Token::QuotedItem,
            Token::GroupEnd,
            Token::Statement,
            Token::QuotedStatement,
        ];

        let key = match self.token_eat_newlines() {
            None => {
                return None;
            }
            Some((Err(_), span)) => {
                return Some(Err(NoValidTokenError::new(
                    VALID_KEY,
                    span.into(),
                    self.content.into(),
                )
                .into()));
            }
            Some((Ok(Token::GroupEnd), span)) => return Some(Ok(Event::GroupEnd { span })),

            Some((Ok(Token::Item), span)) => Item::Item {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::QuotedItem), span)) => Item::Item {
                content: quoted_string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Statement), span)) => Item::Statement {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::QuotedStatement), span)) => Item::Statement {
                content: quoted_string(self.lexer.slice()),
                span,
            },

            Some((Ok(token), span)) => {
                return Some(Err(UnexpectedTokenError::new(
                    VALID_KEY,
                    Some(token),
                    span.into(),
                    self.content.into(),
                )
                .into()))
            }
        };

        const VALID_VALUE: &[Token] = &[Token::Item, Token::QuotedItem, Token::GroupStart];

        // only a group start is allowed to have newlines between the key and value
        while matches!(self.peek(), Some((Ok(Token::NewLine), _))) {
            let _newline = self.token();
            if !matches!(
                self.peek(),
                Some((Ok(Token::GroupStart | Token::NewLine), _))
            ) {
                let span = key.span().end..key.span().end;
                match self.peeked.clone() {
                    Some((Ok(token), _)) => {
                        return Some(Err(UnexpectedTokenError::new(
                            &[Token::GroupStart],
                            Some(token),
                            span.into(),
                            self.content.into(),
                        )
                        .into()))
                    }
                    Some((Err(_), _)) => {
                        return Some(Err(NoValidTokenError::new(
                            VALID_VALUE,
                            span.into(),
                            self.content.into(),
                        )
                        .into()));
                    }
                    None => {
                        return Some(Err(UnexpectedTokenError::new(
                            VALID_VALUE,
                            None,
                            span.into(),
                            self.content.into(),
                        )
                        .into()))
                    }
                }
            }
        }

        let value = match self.token() {
            None => {
                return Some(Err(UnexpectedTokenError::new(
                    VALID_VALUE,
                    None,
                    self.lexer.span().into(),
                    self.content.into(),
                )
                .into()));
            }

            Some((Err(_), span)) => {
                return Some(Err(NoValidTokenError::new(
                    VALID_VALUE,
                    span.into(),
                    self.content.into(),
                )
                .into()));
            }

            Some((Ok(Token::GroupStart), span)) => {
                return Some(Ok(Event::GroupStart {
                    name: key.into_content(),
                    span,
                }))
            }

            Some((Ok(Token::QuotedItem), span)) => Item::Item {
                content: quoted_string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Item), span)) => Item::Item {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::QuotedStatement), span)) => Item::Statement {
                content: quoted_string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Statement), span)) => Item::Statement {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(token), span)) => {
                return Some(Err(UnexpectedTokenError::new(
                    VALID_VALUE,
                    Some(token),
                    span.into(),
                    self.content.into(),
                )
                .into()))
            }
        };

        let span = key.span().start..value.span().end;
        Some(Ok(Event::Entry { key, value, span }))
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.event()
    }
}

fn quoted_string(source: &str) -> Cow<str> {
    string(&source[1..source.len() - 1])
}

fn string(source: &str) -> Cow<str> {
    if source.contains(r#"\""#) || source.contains(r#"\\"#) {
        let mut buffer = source.bytes();
        let mut string = Vec::with_capacity(buffer.len());

        while let Some(byte) = buffer.next() {
            if byte == b'\\' {
                match buffer.next() {
                    Some(b'\\') => string.push(b'\\'),
                    Some(b'"') => string.push(b'"'),
                    Some(byte) => string.extend_from_slice(&[b'\\', byte]),
                    None => break,
                }
            } else {
                string.push(byte);
            }
        }

        String::from_utf8(string).unwrap().into()
    } else {
        source.into()
    }
}
