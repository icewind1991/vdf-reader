use crate::error::WrongEventTypeError;
use crate::VdfError;
use logos::Span;
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

    pub fn as_str(&self) -> &str {
        match self {
            Item::Statement { content, .. } => content.as_ref(),
            Item::Item { content, .. } => content.as_ref(),
        }
    }

    pub fn into_owned(self) -> Item<'static> {
        match self {
            Item::Statement { content, span } => Item::Statement {
                content: content.into_owned().into(),
                span,
            },
            Item::Item { content, span } => Item::Item {
                content: content.into_owned().into(),
                span,
            },
        }
    }
}

/// Reader event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event<'a> {
    /// A group with the given name is starting.
    GroupStart(GroupStartEvent<'a>),

    /// A group has ended.
    GroupEnd(GroupEndEvent),

    /// An entry.
    Entry(EntryEvent<'a>),

    /// An additional value for the previous entry.
    ValueContinuation(ValueContinuationEvent<'a>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventType {
    GroupStart,
    GroupEnd,
    Entry,
    ValueContinuation,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupStartEvent<'a> {
    pub name: Cow<'a, str>,
    pub span: Span,
}

impl GroupStartEvent<'_> {
    pub fn into_owned(self) -> GroupStartEvent<'static> {
        GroupStartEvent {
            name: self.name.into_owned().into(),
            span: self.span,
        }
    }
}

impl<'a> TryFrom<Event<'a>> for GroupStartEvent<'a> {
    type Error = VdfError;

    fn try_from(event: Event<'a>) -> Result<Self, Self::Error> {
        match event {
            Event::GroupStart(event) => Ok(event),
            Event::GroupEnd(_) => {
                Err(WrongEventTypeError::new(event, "group start", "group end").into())
            }
            Event::Entry(_) => Err(WrongEventTypeError::new(event, "group start", "entry").into()),
            Event::ValueContinuation(_) => {
                Err(WrongEventTypeError::new(event, "group start", "value continuation").into())
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupEndEvent {
    pub span: Span,
}

impl<'a> TryFrom<Event<'a>> for GroupEndEvent {
    type Error = VdfError;

    fn try_from(event: Event<'a>) -> Result<Self, Self::Error> {
        match event {
            Event::GroupEnd(event) => Ok(event),
            Event::GroupStart(_) => {
                Err(WrongEventTypeError::new(event, "group end", "group start").into())
            }
            Event::Entry(_) => Err(WrongEventTypeError::new(event, "group start", "entry").into()),
            Event::ValueContinuation(_) => {
                Err(WrongEventTypeError::new(event, "group start", "value continuation").into())
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EntryEvent<'a> {
    pub key: Item<'a>,
    pub value: Item<'a>,
    pub span: Span,
}

impl EntryEvent<'_> {
    pub fn into_owned(self) -> EntryEvent<'static> {
        EntryEvent {
            key: self.key.into_owned(),
            value: self.value.into_owned(),
            span: self.span,
        }
    }
}

impl<'a> TryFrom<Event<'a>> for EntryEvent<'a> {
    type Error = VdfError;

    fn try_from(event: Event<'a>) -> Result<Self, Self::Error> {
        match event {
            Event::Entry(event) => Ok(event),
            Event::GroupEnd(_) => Err(WrongEventTypeError::new(event, "entry", "group end").into()),
            Event::GroupStart(_) => {
                Err(WrongEventTypeError::new(event, "entry", "group start").into())
            }
            Event::ValueContinuation(_) => {
                Err(WrongEventTypeError::new(event, "entry", "value continuation").into())
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ValueContinuationEvent<'a> {
    pub value: Item<'a>,
    pub span: Span,
}

impl ValueContinuationEvent<'_> {
    pub fn into_owned(self) -> ValueContinuationEvent<'static> {
        ValueContinuationEvent {
            value: self.value.into_owned(),
            span: self.span,
        }
    }
}

impl Event<'_> {
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Event::GroupStart(GroupStartEvent { span, .. }) => span.clone(),
            Event::GroupEnd(GroupEndEvent { span, .. }) => span.clone(),
            Event::Entry(EntryEvent { span, .. }) => span.clone(),
            Event::ValueContinuation(ValueContinuationEvent { span, .. }) => span.clone(),
        }
    }
    pub fn into_owned(self) -> Event<'static> {
        match self {
            Event::GroupStart(event) => Event::GroupStart(event.into_owned()),
            Event::GroupEnd(event) => Event::GroupEnd(event),
            Event::Entry(event) => Event::Entry(event.into_owned()),
            Event::ValueContinuation(event) => Event::ValueContinuation(event.into_owned()),
        }
    }

    pub fn ty(&self) -> EventType {
        match self {
            Event::GroupStart(GroupStartEvent { .. }) => EventType::GroupStart,
            Event::GroupEnd(GroupEndEvent { .. }) => EventType::GroupEnd,
            Event::Entry(EntryEvent { .. }) => EventType::Entry,
            Event::ValueContinuation(ValueContinuationEvent { .. }) => EventType::ValueContinuation,
        }
    }
}
