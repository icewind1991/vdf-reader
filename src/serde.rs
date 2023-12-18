use crate::entry::ParseItem;
use crate::error::{ExpectToken, NoValidTokenError, SerdeParseError};
use crate::tokenizer::{SpannedToken, Tokenizer};
use crate::{Token, VdfError};
use logos::Span;
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserialize;
use std::borrow::Cow;

type Result<T, E = VdfError> = std::result::Result<T, E>;

pub struct Deserializer<'de> {
    tokenizer: Tokenizer<'de>,
    peeked: Option<Result<SpannedToken, Span>>,
    last_key: Cow<'de, str>,
}

const STRING_ITEMS: &[Token] = &[
    Token::Item,
    Token::QuotedItem,
    Token::Statement,
    Token::QuotedStatement,
];

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            tokenizer: Tokenizer::from_str(input),
            peeked: None,
            last_key: "".into(),
        }
    }

    pub fn source(&self) -> &'de str {
        self.tokenizer.source()
    }

    pub fn next(&mut self) -> Option<Result<SpannedToken, Span>> {
        self.peeked.take().or_else(|| self.tokenizer.next())
    }

    pub fn peek(&mut self) -> Option<Result<SpannedToken, Span>> {
        if self.peeked.is_none() {
            self.peeked = self.tokenizer.next();
        }
        self.peeked.clone()
    }

    pub fn push_peeked(&mut self, token: SpannedToken) {
        self.peeked = Some(Ok(token))
    }

    fn read_str(&mut self) -> Result<(Cow<'de, str>, Span)> {
        let token = self.next().expect_token(STRING_ITEMS, self.source())?;
        Ok((token.string(self.source()), token.span))
    }

    fn parse<T: ParseItem>(&mut self) -> Result<T> {
        let (str, span) = self.read_str()?;
        T::from_str(str.as_ref())
            .map_err(|e| SerdeParseError::new(e.ty, &e.value, span, self.source()).into())
    }

    fn set_last_key(&mut self, key: Cow<'de, str>) {
        self.last_key = key;
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    T::deserialize(&mut deserializer)
}

const VALUE_TOKEN: &[Token] = &[
    Token::Item,
    Token::QuotedItem,
    Token::Statement,
    Token::QuotedStatement,
    Token::GroupStart,
];

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = VdfError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let source = self.source();
        let peek = self.peek().expect_token(VALUE_TOKEN, source)?;
        match peek.token {
            Token::Item | Token::QuotedItem | Token::Statement | Token::QuotedStatement => {
                self.deserialize_str(visitor)
            }
            Token::GroupStart => self.deserialize_map(visitor),
            _ => unreachable!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (str, span) = self.read_str()?;
        let mut chars = str.chars();
        match (chars.next(), chars.next()) {
            (Some(_), None) => Ok(()),
            _ => Err(SerdeParseError::new(
                "char",
                str.as_ref(),
                span,
                self.source(),
            )),
        }?;

        visitor.visit_str(str.as_ref())
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.read_str()?.0.as_ref())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.read_str()?.0.into())
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_str()?.0.as_bytes().into())
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let token = match self.next() {
            None => return visitor.visit_none(),
            Some(Err(span)) => {
                return Err(
                    NoValidTokenError::new(VALUE_TOKEN, span.into(), self.source().into()).into(),
                )
            }
            Some(Ok(token)) => token,
        };
        if token.span.len() == 0 {
            return visitor.visit_none();
        }
        self.push_peeked(token);
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (str, span) = self.read_str()?;
        if !str.is_empty() {
            return Err(SerdeParseError::new("unit", str.as_ref(), span, self.source()).into());
        }
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (str, span) = self.read_str()?;
        if !str.is_empty() {
            return Err(SerdeParseError::new("unit", str.as_ref(), span, self.source()).into());
        }
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let token = self.peek().expect_token(STRING_ITEMS, self.source())?;
        let value_str = &self.source()[token.span.clone()];
        if value_str.starts_with("\"[") && value_str.ends_with("]\"") {
            let _ = self.next();
            let seq = &value_str[2..value_str.len() - 2];
            let span = token.span.start + 2..token.span.end - 2;
            visitor.visit_seq(StringArrayWalker::new(self.source(), seq, span))
        } else {
            let key = self.last_key.clone();
            visitor.visit_seq(SeqWalker::new(&mut self, key))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // as a special case we allow a map without a `{` at the start of the file to create a top level struct
        let toplevel = match self
            .peek()
            .expect_token(&[Token::GroupStart], self.source())
        {
            Ok(_) => {
                let _ = self.next();
                false
            }
            Err(VdfError::UnexpectedToken(e)) => {
                if self.tokenizer.count > 1 {
                    return Err(e.into());
                }
                true
            }
            Err(e) => {
                return Err(e);
            }
        };

        let value = visitor.visit_map(TableWalker::new(&mut self, toplevel))?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(Enum::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct TableWalker<'source, 'a> {
    de: &'a mut Deserializer<'source>,
    done: bool,
    toplevel: bool,
}

const KEY_TOKEN: &[Token] = &[
    Token::Item,
    Token::QuotedItem,
    Token::Statement,
    Token::QuotedStatement,
    Token::GroupEnd,
];

impl<'source, 'a> TableWalker<'source, 'a> {
    pub fn new(de: &'a mut Deserializer<'source>, toplevel: bool) -> Self {
        TableWalker {
            de,
            done: false,
            toplevel,
        }
    }

    fn source(&self) -> &'source str {
        self.de.source()
    }

    fn key_token(&mut self) -> Result<Option<SpannedToken>> {
        if self.done {
            return Ok(None);
        }

        let expected = if self.toplevel {
            STRING_ITEMS
        } else {
            KEY_TOKEN
        };

        let token = match (self.de.next(), self.toplevel) {
            (Some(token), _) => token,
            (None, true) => {
                self.done = true;
                return Ok(None);
            }
            (None, false) => {
                return Err(None::<SpannedToken>
                    .expect_token(expected, self.source())
                    .unwrap_err())
            }
        };

        let key = token.expect_token(expected, self.source())?;

        if key.token == Token::GroupEnd {
            self.done = true;
            return Ok(None);
        }
        Ok(Some(key))
    }
}

impl<'de, 'a> MapAccess<'de> for TableWalker<'de, 'a> {
    type Error = VdfError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let key = match self.key_token() {
            Ok(Some(key)) => key,
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        self.de.set_last_key(key.string(self.source()));
        self.de.push_peeked(key);
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct SeqWalker<'source, 'a> {
    table: TableWalker<'source, 'a>,
    key: Cow<'source, str>,
    done: bool,
}

impl<'source, 'a> SeqWalker<'source, 'a> {
    pub fn new(de: &'a mut Deserializer<'source>, key: Cow<'source, str>) -> Self {
        SeqWalker {
            done: false,
            key,
            table: TableWalker::new(de, false),
        }
    }

    fn source(&self) -> &'source str {
        self.table.source()
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqWalker<'de, 'a> {
    type Error = VdfError;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.done {
            return Ok(None);
        }
        let value = seed.deserialize(&mut *self.table.de).map(Some)?;

        let key_token = match self.table.key_token() {
            Ok(Some(key)) => key,
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let key = key_token.string(self.source());
        if key != self.key {
            self.table.de.push_peeked(key_token);
            self.done = true;
        }

        Ok(value)
    }
}

struct StringArrayWalker<'source> {
    source: &'source str,
    remaining: &'source str,
    span: Span,
}

impl<'source> StringArrayWalker<'source> {
    fn new(source: &'source str, array: &'source str, span: Span) -> Self {
        StringArrayWalker {
            source,
            remaining: array,
            span,
        }
    }
}

impl<'de, 'source> SeqAccess<'de> for StringArrayWalker<'source>
where
    'source: 'de,
{
    type Error = VdfError;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining.is_empty() {
            return Ok(None);
        }

        let (item, rest) = self
            .remaining
            .split_once(" ")
            .unwrap_or((self.remaining, ""));
        let item_span = self.span.start..(self.span.start + item.len());
        self.remaining = rest.trim();
        self.span = (self.span.end - self.remaining.len())..self.span.end;

        let mut de = Deserializer::from_str(item);
        let val = seed
            .deserialize(&mut de)
            .map_err(|e| e.with_source_span(item_span, self.source))?;
        Ok(Some(val))
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }

    fn source(&self) -> &'de str {
        self.de.source()
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = VdfError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = VdfError;

    fn unit_variant(self) -> Result<()> {
        let (str, span) = self.de.read_str()?;

        if !str.is_empty() {
            return Err(SerdeParseError::new("unit", str.as_ref(), span, self.source()).into());
        }

        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::VdfError;
    use serde::Deserialize;

    fn unwrap_err<T>(r: Result<T, VdfError>) -> T {
        r.map_err(miette::Error::from).unwrap()
    }

    fn from_str<'a, T>(source: &'a str) -> super::Result<T>
    where
        T: serde::Deserialize<'a>,
    {
        match super::from_str(source) {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("{}", err);
                Err(err)
            }
        }
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: String,
        }

        let j = r#"{"int" 1 "seq" "b"}"#;
        let expected = Test {
            int: 1,
            seq: "b".into(),
        };
        assert_eq!(expected, unwrap_err(from_str(j)));
    }

    #[test]
    fn test_struct_toplevel() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: String,
        }

        let j = r#""int" 1 "seq" "b""#;
        let expected = Test {
            int: 1,
            seq: "b".into(),
        };
        assert_eq!(expected, unwrap_err(from_str(j)));
    }

    #[test]
    fn test_struct_nested() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Inner {
            a: f32,
            b: bool,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            nested: Inner,
        }

        let j = r#"{"int" 1 "nested" {"a" 1.0 "b" false}}"#;
        let expected = Test {
            int: 1,
            nested: Inner { a: 1.0, b: false },
        };
        assert_eq!(expected, unwrap_err(from_str(j)));
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype1(u32),
            Newtype2(u32),
            Struct { a: u32 },
            Struct2 { a: u32 },
        }

        let j = r#""Unit" """#;
        let expected = E::Unit;
        assert_eq!(expected, unwrap_err(from_str(j)));

        let j = r#""Newtype1" 1"#;
        let expected = E::Newtype1(1);
        assert_eq!(expected, unwrap_err(from_str(j)));

        let j = r#"Newtype2 1"#;
        let expected = E::Newtype2(1);
        assert_eq!(expected, unwrap_err(from_str(j)));

        let j = r#"Struct {"a" 1}"#;
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, unwrap_err(from_str(j)));

        let j = r#"Struct2 {"a" 1}"#;
        let expected = E::Struct2 { a: 1 };
        assert_eq!(expected, unwrap_err(from_str(j)));
    }
}
