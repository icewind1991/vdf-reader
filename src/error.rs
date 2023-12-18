use crate::entry::Entry;
use crate::tokenizer::SpannedToken;
use crate::{Event, Item, Token};
use logos::Span;
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
    /// Wrong event to for conversion
    WrongEntryType(Box<WrongEventTypeError>),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// Failed to parse entry into type
    ParseEntry(#[from] ParseEntryError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// Failed to parse item into type
    ParseItem(#[from] ParseItemError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// Failed to parse string into type
    ParseString(#[from] ParseStringError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// Failed to find an enum variant that matches the found tag
    UnknownVariant(#[from] UnknownVariantError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    /// Failed to parse serde string
    SerdeParse(#[from] SerdeParseError),
    #[error("{0}")]
    Other(String),
}

impl From<WrongEventTypeError> for VdfError {
    fn from(value: WrongEventTypeError) -> Self {
        Self::WrongEntryType(value.into())
    }
}

impl VdfError {
    pub(crate) fn with_source_span<Sp: Into<SourceSpan>, Sr: Into<String>>(
        self,
        span: Sp,
        source: Sr,
    ) -> VdfError {
        match self {
            VdfError::UnexpectedToken(e) => UnexpectedTokenError {
                src: source.into(),
                err_span: span.into(),
                ..e
            }
            .into(),
            VdfError::NoValidToken(e) => NoValidTokenError {
                src: source.into(),
                err_span: span.into(),
                ..e
            }
            .into(),
            VdfError::WrongEntryType(e) => WrongEventTypeError {
                src: source.into(),
                err_span: span.into(),
                ..*e
            }
            .into(),
            VdfError::SerdeParse(e) => SerdeParseError {
                src: source.into(),
                err_span: span.into(),
                ..e
            }
            .into(),
            VdfError::UnknownVariant(e) => UnknownVariantError {
                src: source.into(),
                err_span: span.into(),
                ..e
            }
            .into(),
            _ => self,
        }
    }
}

struct CommaSeperated<'a, T>(&'a [T]);

impl<T: Display> Display for CommaSeperated<'_, T> {
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
#[diagnostic(code(vmt_reader::unexpected_token))]
pub struct UnexpectedTokenError {
    #[label("Expected {}", CommaSeperated(self.expected))]
    err_span: SourceSpan,
    pub expected: &'static [Token],
    pub found: Option<Token>,
    #[source_code]
    src: String,
}

impl UnexpectedTokenError {
    pub fn new(
        expected: &'static [Token],
        found: Option<Token>,
        err_span: SourceSpan,
        src: String,
    ) -> Self {
        UnexpectedTokenError {
            err_span,
            expected,
            found,
            src,
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
                CommaSeperated(self.expected)
            ),
            None => write!(
                f,
                "Unexpected end of input expected one of {}",
                CommaSeperated(self.expected)
            ),
        }
    }
}

impl Error for UnexpectedTokenError {}

/// A token that wasn't expected was found while parsing
#[derive(Debug, Clone, Diagnostic, Error)]
#[diagnostic(code(vmt_reader::no_valid_token))]
#[error("No valid token found, expected one of {}", CommaSeperated(self.expected))]
pub struct NoValidTokenError {
    #[label("Expected {}", CommaSeperated(self.expected))]
    err_span: SourceSpan,
    pub expected: &'static [Token],
    #[source_code]
    src: String,
}

impl NoValidTokenError {
    pub fn new(expected: &'static [Token], err_span: SourceSpan, src: String) -> Self {
        NoValidTokenError {
            err_span,
            expected,
            src,
        }
    }
}

/// Wrong event to for conversion
#[derive(Debug, Clone, Diagnostic, Error)]
#[diagnostic(code(vmt_reader::wrong_value_type))]
#[error("Wrong event to for conversion, expected a {expected} but found a {got}")]
pub struct WrongEventTypeError {
    pub expected: &'static str,
    pub got: &'static str,
    pub event: Event<'static>,
    #[label("Expected a {}", self.expected)]
    err_span: SourceSpan,
    #[source_code]
    src: String,
}

impl WrongEventTypeError {
    pub fn new(event: Event, expected: &'static str, got: &'static str) -> Self {
        WrongEventTypeError {
            err_span: event.span().into(),
            event: event.into_owned(),
            expected,
            got,
            src: String::new(),
        }
    }
    pub fn new_with_source(
        event: Event,
        expected: &'static str,
        got: &'static str,
        src: String,
    ) -> Self {
        WrongEventTypeError {
            err_span: event.span().into(),
            event: event.into_owned(),
            expected,
            got,
            src,
        }
    }

    pub fn with_source(self, src: String) -> Self {
        WrongEventTypeError { src, ..self }
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Can't parse entry {value:?} as {ty}")]
#[diagnostic(code(vmt_parser::parse_value))]
pub struct ParseEntryError {
    pub ty: &'static str,
    pub value: Entry,
}

impl ParseEntryError {
    pub fn new(ty: &'static str, value: Entry) -> Self {
        ParseEntryError { ty, value }
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Can't parse entry {value:?} as {ty}")]
#[diagnostic(code(vmt_parser::parse_item))]
pub struct ParseItemError {
    pub ty: &'static str,
    pub value: Item<'static>,
}

impl ParseItemError {
    pub fn new(ty: &'static str, value: Item) -> Self {
        ParseItemError {
            ty,
            value: value.into_owned(),
        }
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Can't parse string {value:?} as {ty}")]
#[diagnostic(code(vmt_parser::parse_string))]
pub struct ParseStringError {
    pub ty: &'static str,
    pub value: String,
}

impl ParseStringError {
    pub fn new(ty: &'static str, value: &str) -> Self {
        ParseStringError {
            ty,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Can't parse {value:?} as {ty}")]
#[diagnostic(code(vmt_parser::parse_serde))]
pub struct SerdeParseError {
    pub ty: &'static str,
    pub value: String,
    #[label("Expected a {ty}")]
    err_span: SourceSpan,
    #[source_code]
    src: String,
}

impl SerdeParseError {
    pub fn new(ty: &'static str, value: &str, span: Span, src: &str) -> Self {
        SerdeParseError {
            ty,
            value: value.into(),
            err_span: span.into(),
            src: src.into(),
        }
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("Unknown variant {variant:?} expected on of {}", ExpectedVariants(self.expected))]
#[diagnostic(code(vmt_parser::unknown_variant))]
pub struct UnknownVariantError {
    variant: String,
    expected: &'static [&'static str],
    #[label("{}", ExpectedVariants(self.expected))]
    err_span: SourceSpan,
    #[source_code]
    src: String,
}

struct ExpectedVariants(&'static [&'static str]);

impl Display for ExpectedVariants {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "there are no variants")
        } else {
            write!(f, "expected on of {}", CommaSeperated(self.0))
        }
    }
}

impl UnknownVariantError {
    pub fn new(variant: &str, expected: &'static [&'static str], span: Span, src: &str) -> Self {
        UnknownVariantError {
            variant: variant.into(),
            expected,
            err_span: span.into(),
            src: src.into(),
        }
    }
}

pub trait ExpectToken<'source> {
    fn expect_token(
        self,
        expected: &'static [Token],
        source: &'source str,
    ) -> Result<SpannedToken, VdfError>;
}

impl<'source, T: ExpectToken<'source>> ExpectToken<'source> for Option<T> {
    fn expect_token(
        self,
        expected: &'static [Token],
        source: &'source str,
    ) -> Result<SpannedToken, VdfError> {
        self.ok_or_else(|| {
            NoValidTokenError::new(expected, (source.len()..source.len()).into(), source.into())
                .into()
        })
        .and_then(|token| token.expect_token(expected, source))
    }
}

impl<'source> ExpectToken<'source> for Result<SpannedToken, Span> {
    fn expect_token(
        self,
        expected: &'static [Token],
        source: &'source str,
    ) -> Result<SpannedToken, VdfError> {
        self.map_err(|span| NoValidTokenError::new(expected, span.into(), source.into()).into())
            .and_then(|token| token.expect_token(expected, source))
    }
}

impl<'source> ExpectToken<'source> for SpannedToken {
    fn expect_token(
        self,
        expected: &'static [Token],
        source: &'source str,
    ) -> Result<SpannedToken, VdfError> {
        if expected.iter().any(|expect| self.token.eq(expect)) {
            Ok(self)
        } else {
            Err(UnexpectedTokenError::new(
                expected,
                Some(self.token),
                self.span.into(),
                source.into(),
            )
            .into())
        }
    }
}

impl serde::de::Error for VdfError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        VdfError::Other(msg.to_string())
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        UnknownVariantError::new(variant, expected, 0..0, "").into()
    }
}
