//! Deserialize VDF data to a Rust data structure

use std::collections::VecDeque;

use serde::Deserialize;
use serde::de::{self, Visitor, MapAccess, DeserializeSeed, IntoDeserializer};
use steamy_vdf::parser::{self as vdf_parser, Token};

use crate::error::{Error, Result};
use std::str::FromStr;
use std::borrow::{Borrow, Cow};

/// A structure that deserializes VDF into Rust values
pub struct Deserializer<'de> {
    input: &'de str,
    parsed_input: VecDeque<Token<'de>>,
    top_level: bool,
}

impl<'de> Deserializer<'de> {
    /// Creates a VDF deserializer from a `&str`
    pub fn from_str(input: &'de str) -> Self {
        Self {
            input,
            parsed_input: VecDeque::new(),
            top_level: true,
        }
    }
}

/// Deserialize an instance of type `T` from a string of VDF text
///
/// # Errors
///
/// If `s` is not valid VDF, or `T` uses an unsupported Serde data type,
/// or `T`'s `Deserialize` implementation itself returns an error, an error will be
/// returned.
pub fn from_str<'a, T>(s: &'a str) -> Result<T> where T: Deserialize<'a> {
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    // before we toss a LateEOF, let's make sure we're not erroring on some whitespace
    let remaining_input = deserializer.input.trim_end();
    if remaining_input.is_empty() {
        Ok(t)
    } else {
        Err(Error::LateEOF)
    }
}

impl<'de> Deserializer<'de> {
    fn parse_more(&mut self) -> Result<()> {
        let parsed = vdf_parser::next(self.input.as_bytes());
        match parsed {
            nom::IResult::Done(remainder, token) => {
                // since it came from `as_bytes` this is safe
                self.input = unsafe { std::str::from_utf8_unchecked(remainder) };
                self.parsed_input.push_back(token);
            }
            nom::IResult::Incomplete(_) => return Err(Error::EarlyEOF),
            nom::IResult::Error(err) => return Err(Error::Tokenize(err.to_string())),
        }
        Ok(())
    }

    fn parse_more_if_needed(&mut self) -> Result<()> {
        if self.parsed_input.is_empty() {
            self.parse_more()?;
        }
        Ok(())
    }

    fn peek_token(&mut self) -> Result<&Token<'de>> {
        self.parse_more_if_needed()?;
        self.parsed_input.get(0).ok_or(Error::EarlyEOF)
    }

    fn next_token(&mut self) -> Result<Token<'de>> {
        self.parse_more_if_needed()?;
        self.parsed_input.pop_front().ok_or(Error::EarlyEOF)
    }

    fn next_token_item(&mut self) -> Result<Cow<'de, str>> {
        match self.next_token()? {
            Token::Item(data) => Ok(data),
            got => Err(Error::Expected("Item", format!("{:?}", got))),
        }
    }

    fn parse_next_token_data<T: FromStr>(&mut self) -> Result<T> where T::Err : std::fmt::Display {
        self.next_token_item()?.parse().map_err(|err: T::Err| Error::StringParse(err.to_string()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("any"))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.next_token()? {
            Token::Item(data) if data == "0" => visitor.visit_bool(false),
            Token::Item(data) if data == "1" => visitor.visit_bool(true),
            got => Err(Error::Expected("bool (\"0\" or \"1\")", format!("{:?}", got))),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse_next_token_data()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse_next_token_data()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse_next_token_data()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse_next_token_data()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse_next_token_data()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse_next_token_data()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse_next_token_data()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse_next_token_data()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.parse_next_token_data()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.parse_next_token_data()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_char(self.parse_next_token_data()?)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_str(self.next_token_item()?.borrow())
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_string(String::from(self.next_token_item()?))
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("byte array"))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("byte array"))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("unit"))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("unit_struct"))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value> {
        if self.top_level {
            let name_token = self.next_token()?;
            match name_token {
                Token::Item(name_token) if name_token == name => {},
                got => return Err(Error::Expected(name, format!("{:?}", got))),
            }
            self.top_level = false;
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("seq"))
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("tuple"))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedType("tuple_struct"))
    }

    fn deserialize_map<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        match self.next_token()? {
            Token::GroupStart => {
                let value = visitor.visit_map(TabNewlineSeparated::new(&mut self))?;
                match self.next_token()? {
                    Token::GroupEnd => Ok(value),
                    got => Err(Error::Expected("'}'", format!("{:?}", got))),
                }
            }
            got => Err(Error::Expected("'{'", format!("{:?}", got))),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value> {
        if self.top_level {
            let name_token = self.next_token()?;
            match name_token {
                Token::Item(name_token) if name_token == name => {},
                got => return Err(Error::Expected(name, format!("{:?}", got))),
            }
            self.top_level = false;
        }
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value> {
        visitor.visit_enum(self.next_token_item()?.into_deserializer())
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    serde::forward_to_deserialize_any! {
        ignored_any
    }
}

struct TabNewlineSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> TabNewlineSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for TabNewlineSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where
            K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek_token()? == &Token::GroupEnd {
            return Ok(None);
        }
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where
            V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Inner {
            foo: String,
            bar: bool,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            inner: Inner,
        }

        let j = concat!(
            "\"Test\"\n",
            "{\n",
            "\t\"int\"\t\"1\"\n",
            "\t\"inner\"\n",
            "\t{\n",
            "\t\t\"foo\"\t\"baz\"\n",
            "\t\t\"bar\"\t\"0\"\n",
            "\t}\n",
            "}"
        );
        let expected = Test {
            int: 1,
            inner: Inner {
                foo: "baz".to_string(),
                bar: false,
            },
        };
        assert_eq!(expected, from_str(j).unwrap());
    }
}
