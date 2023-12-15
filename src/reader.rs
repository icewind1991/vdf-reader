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

    /// EOF has been reached.
    End { span: Span },
}

impl Event<'_> {
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Event::GroupStart { span, .. } => span.clone(),
            Event::GroupEnd { span, .. } => span.clone(),
            Event::Entry { span, .. } => span.clone(),
            Event::End { span, .. } => span.clone(),
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
    pub fn event(&mut self) -> Result<Event> {
        let key = match self.lexer.next() {
            None => {
                return Ok(Event::End {
                    span: self.lexer.span(),
                })
            }
            Some((Err(_), span)) => {
                return Err(NoValidTokenError::new(
                    &[Token::Item, Token::GroupEnd, Token::Statement],
                    span.into(),
                    self.content.into(),
                )
                .into());
            }
            Some((Ok(Token::GroupEnd), span)) => return Ok(Event::GroupEnd { span }),
            Some((Ok(Token::GroupStart), span)) => {
                return Err(UnexpectedTokenError::new(
                    &[Token::Item, Token::GroupEnd, Token::Statement],
                    Some(Token::GroupStart),
                    span.into(),
                    self.content.into(),
                )
                .into())
            }

            Some((Ok(Token::Item), span)) => Item::Value {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Statement), span)) => Item::Statement {
                content: string(self.lexer.slice()),
                span,
            },
        };

        let value = match self.lexer.next() {
            None => {
                return Err(UnexpectedTokenError::new(
                    &[Token::Item, Token::GroupEnd, Token::Statement],
                    None,
                    self.lexer.span().into(),
                    self.content.into(),
                )
                .into());
            }

            Some((Err(_), span)) => {
                return Err(NoValidTokenError::new(
                    &[Token::Item, Token::GroupEnd, Token::Statement],
                    span.into(),
                    self.content.into(),
                )
                .into());
            }

            Some((Ok(Token::GroupEnd), span)) => {
                return Err(UnexpectedTokenError::new(
                    &[Token::Item, Token::GroupStart, Token::Statement],
                    Some(Token::GroupEnd),
                    span.into(),
                    self.content.into(),
                )
                .into())
            }

            Some((Ok(Token::GroupStart), span)) => {
                return Ok(Event::GroupStart {
                    name: key.into_content(),
                    span,
                })
            }

            Some((Ok(Token::Item), span)) => Item::Value {
                content: string(self.lexer.slice()),
                span,
            },

            Some((Ok(Token::Statement), span)) => Item::Statement {
                content: string(self.lexer.slice()),
                span,
            },
        };

        let span = key.span().start..value.span().end;
        Ok(Event::Entry { key, value, span })
    }
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
