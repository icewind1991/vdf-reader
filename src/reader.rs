use super::{Result, Token};
use crate::error::{NoValidTokenError, UnexpectedTokenError};
use crate::event::{
    EntryEvent, Event, EventType, GroupEndEvent, GroupStartEvent, Item, ValueContinuationEvent,
};
use logos::{Lexer, Logos, Span, SpannedIter};
use std::borrow::Cow;

/// A VDF token reader.
pub struct Reader<'a> {
    pub source: &'a str,
    pub last_event: Option<EventType>,
    lexer: SpannedIter<'a, Token>,
}

impl<'a> From<&'a str> for Reader<'a> {
    fn from(content: &'a str) -> Self {
        Reader {
            source: content,
            last_event: None,
            lexer: Lexer::new(content).spanned(),
        }
    }
}

impl<'a> Reader<'a> {
    fn token(&mut self) -> Option<(Result<Token, <Token as Logos>::Error>, Span)> {
        self.lexer.next()
    }

    pub fn span(&self) -> Span {
        self.lexer.span()
    }

    /// Get the next event, this does copies.
    pub fn event(&mut self) -> Option<Result<Event<'a>>> {
        let result = self.event_inner();
        if let Some(Ok(event)) = &result {
            self.last_event = Some(event.ty());
        }
        result
    }

    #[allow(dead_code)]
    fn event_inner(&mut self) -> Option<Result<Event<'a>>> {
        const VALID_KEY: &[Token] = &[
            Token::Item,
            Token::QuotedItem,
            Token::GroupEnd,
            Token::Statement,
            Token::QuotedStatement,
        ];

        let whitespace_start = self.span().end;

        let key = match self.token() {
            None => {
                return None;
            }
            Some((Err(_), span)) => {
                return Some(Err(NoValidTokenError::new(
                    VALID_KEY,
                    span.into(),
                    self.source.into(),
                )
                .into()));
            }
            Some((Ok(Token::GroupEnd), span)) => {
                return Some(Ok(Event::GroupEnd(GroupEndEvent { span })))
            }

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
                    self.source.into(),
                )
                .into()))
            }
        };

        let whitespace_end = self.span().start;
        let skipped_newline = self.source[whitespace_start..whitespace_end].contains('\n');
        let last_event_has_value = matches!(
            self.last_event,
            Some(EventType::Entry | EventType::ValueContinuation)
        );

        // multiple values on the same line create an array
        if last_event_has_value && !skipped_newline {
            return Some(Ok(Event::ValueContinuation(ValueContinuationEvent {
                value: key,
                span: self.span(),
            })));
        }

        const VALID_VALUE: &[Token] = &[
            Token::Item,
            Token::QuotedItem,
            Token::GroupStart,
            Token::Statement,
            Token::QuotedStatement,
        ];

        let value = match self.token() {
            None => {
                return Some(Err(UnexpectedTokenError::new(
                    VALID_VALUE,
                    None,
                    self.lexer.span().into(),
                    self.source.into(),
                )
                .into()));
            }

            Some((Err(_), span)) => {
                return Some(Err(NoValidTokenError::new(
                    VALID_VALUE,
                    span.into(),
                    self.source.into(),
                )
                .into()));
            }

            Some((Ok(Token::GroupStart), span)) => {
                return Some(Ok(Event::GroupStart(GroupStartEvent {
                    name: key.into_content(),
                    span,
                })))
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
                    self.source.into(),
                )
                .into()))
            }
        };

        let span = key.span().start..value.span().end;
        Some(Ok(Event::Entry(EntryEvent { key, value, span })))
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.event()
    }
}

pub(crate) fn quoted_string(source: &str) -> Cow<str> {
    let source = &source[1..source.len() - 1];

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

fn string(source: &str) -> Cow<str> {
    source.into()
}
