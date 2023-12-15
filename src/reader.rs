use super::{Result, Token};
use crate::error::{NoValidTokenError, UnexpectedTokenError};
use logos::{Lexer, Span, SpannedIter};
use std::borrow::Cow;

/// Kinds of item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item<'a> {
    /// A statement, the ones starting with #.
    Statement { content: Cow<'a, str>, span: Span },

    /// A value.
    Value { content: Cow<'a, str>, span: Span },
}

impl<'a> Item<'a> {
    pub fn span(&self) -> Span {
        match self {
            Item::Statement { span, .. } => span.clone(),
            Item::Value { span, .. } => span.clone(),
        }
    }

    pub fn into_content(self) -> Cow<'a, str> {
        match self {
            Item::Statement { content, .. } => content,
            Item::Value { content, .. } => content,
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
}

impl<'a> From<&'a str> for Reader<'a> {
    fn from(content: &'a str) -> Self {
        Reader {
            content,
            lexer: Lexer::new(content).spanned(),
        }
    }
}

impl<'a> Reader<'a> {
    /// Get the next event, this does copies.
    #[allow(dead_code)]
    pub fn event(&mut self) -> Option<Result<Event>> {
        const VALID_KEY: &[Token] = &[
            Token::Item,
            Token::QuotedItem,
            Token::GroupEnd,
            Token::Statement,
            Token::QuotedStatement,
        ];

        let key = match self.lexer.next() {
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

            Some((Ok(Token::Item), span)) => Item::Value {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::QuotedItem), span)) => Item::Value {
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

        let value = match self.lexer.next() {
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

            Some((Ok(Token::QuotedItem), span)) => Item::Value {
                content: quoted_string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Item), span)) => Item::Value {
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
