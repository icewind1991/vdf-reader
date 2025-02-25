use crate::reader::quoted_string;
use crate::Token;
use logos::{Lexer, Span};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn string<'source>(&self, source: &'source str) -> Cow<'source, str> {
        let full = &source[self.span.clone()];
        match self.token {
            Token::QuotedItem | Token::QuotedStatement => quoted_string(full),
            _ => full.into(),
        }
    }
}

pub struct Tokenizer<'source> {
    lexer: Lexer<'source, Token>,
    /// The number of tokens tokenized so far
    pub count: usize,
}

impl<'source> Tokenizer<'source> {
    pub fn from_str(input: &'source str) -> Self {
        Tokenizer {
            lexer: Lexer::new(input),
            count: 0,
        }
    }

    pub fn source(&self) -> &'source str {
        self.lexer.source()
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<SpannedToken, Span>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = match self.lexer.next() {
            Some(Ok(token)) => token,
            Some(Err(_)) => {
                return Some(Err(self.lexer.span()));
            }
            None => {
                return None;
            }
        };
        self.count += 1;
        Some(Ok(SpannedToken {
            token,
            span: self.lexer.span(),
        }))
    }
}
