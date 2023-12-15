use crate::Token;
use miette::{Diagnostic, SourceSpan};
use std::error::Error;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Any error that occurred while trying to parse the vdf file
#[derive(Error, Debug, Clone, Diagnostic)]
pub enum VdfError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// A token that wasn't expected was found while parsing
    UnexpectedToken(#[from] UnexpectedTokenError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// No valid token found
    NoValidToken(#[from] NoValidTokenError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// An unexpected statement was found inside a table
    StatementInTable(#[from] StatementInTableError),
}

struct ExpectedTokens<'a>(&'a [Token]);

impl Display for ExpectedTokens<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut tokens = self.0.iter();
        if let Some(token) = tokens.next() {
            write!(f, "{}", token)?;
        } else {
            return Ok(());
        }

        for token in tokens {
            write!(f, ", {}", token)?;
        }

        Ok(())
    }
}

/// A token that wasn't expected was found while parsing
#[derive(Debug, Clone, Diagnostic)]
#[diagnostic(code(php_literal_parser::unexpected_token))]
pub struct UnexpectedTokenError {
    #[label("Expected {}", ExpectedTokens(self.expected))]
    err_span: SourceSpan,
    pub expected: &'static [Token],
    pub found: Option<Token>,
    #[source_code]
    src: String,
}

impl UnexpectedTokenError {
    pub fn new(expected: &'static [Token], found: Option<Token>, err_span: SourceSpan, src: String) -> Self {
        UnexpectedTokenError {
            err_span,
            expected,
            found,
            src
        }
    }
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.found {
            Some(token) => write!(
                f,
                "Unexpected token, found {} expected one of {}",
                token,
                ExpectedTokens(self.expected)
            ),
            None => write!(
                f,
                "Unexpected end of input expected one of {}",
                ExpectedTokens(self.expected)
            ),
        }
    }
}

impl Error for UnexpectedTokenError {}

/// A token that wasn't expected was found while parsing
#[derive(Debug, Clone, Diagnostic)]
#[diagnostic(code(php_literal_parser::unexpected_token))]
pub struct NoValidTokenError {
    #[label("Expected {}", ExpectedTokens(self.expected))]
    err_span: SourceSpan,
    pub expected: &'static [Token],
    #[source_code]
    src: String,
}

impl NoValidTokenError {
    pub fn new(expected: &'static [Token], err_span: SourceSpan, src: String) -> Self {
        NoValidTokenError { err_span, expected ,src }
    }
}

impl Display for NoValidTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "No valid token found, expected one of {}",
            ExpectedTokens(self.expected)
        )
    }
}

impl Error for NoValidTokenError {}

/// An unexpected statement was found inside a table
#[derive(Debug, Clone, Diagnostic, Error)]
#[diagnostic(code(php_literal_parser::unexpected_token))]
#[error("An unexpected statement was found inside a table")]
pub struct StatementInTableError {
    #[label("Unexpected statement")]
    err_span: SourceSpan,
    #[source_code]
    src: String,
}
impl StatementInTableError {
    pub fn new(err_span: SourceSpan, src: String) -> Self {
        StatementInTableError { err_span, src }
    }
}
