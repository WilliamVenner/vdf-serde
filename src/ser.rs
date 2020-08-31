//! Serialize a Rust data structure into VDF data

use serde::{ser::{self, Impossible}, Serialize};

use crate::error::{Error, Result};
use serde::ser::SerializeMap;

/// A structure for serializing Rust values into VDF
pub struct Serializer {
    output: String,
    indent_level: usize,
}

/// Serialize the given data structure as a String of VDF
///
/// # Errors
///
/// If `T` uses an unsupported Serde data type, or `T`'s `Serialize` implementation
/// itself returns an error, an error will be returned.
///
/// Notably, if `T` has a map with a non-atomic key, an error will not be returned.
/// In these cases, this behavior is likely incompatible with other VDF parsers.
pub fn to_string<T>(value: &T) -> Result<String>
    where
        T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
        indent_level: 0,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { r#""1""# } else { r#""0""# };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += "\"";
        self.output += &v.to_string();
        self.output += "\"";
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += "\"";
        self.output += &v.to_string();
        self.output += "\"";
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += "\"";
        self.output += &v.to_string();
        self.output += "\"";
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let escaped = v
            .replace('\\', r"\\")
            .replace('\n', r"\n")
            .replace('\t', r"\t")
            .replace('"', r#"\""#);
        self.output += "\"";
        self.output += &escaped;
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::Message("can't serialize arrays in VDF".to_string()))
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::Message("can't serialize None in VDF".to_string()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::Message("can't serialize () in VDF".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        Err(Error::Message("can't serialize newtype variant in VDF".to_string()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Message("can't serialize arrays in VDF".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Message("can't serialize tuples in VDF".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Message("can't serialize tuples in VDF".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Message("can't serialize tuples in VDF".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if self.output.ends_with('\t') {
            self.output.pop();
        }
        self.output += "\n";
        self.output += &"\t".repeat(self.indent_level);
        self.output += "{\n";
        self.indent_level += 1;
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        if self.indent_level == 0 {
            self.output += &format!("\"{}\"", name);
        }
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Message("can't serialize struct variants in VDF".to_string()))
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<()> where
        T: std::fmt::Display {
        self.serialize_str(&value.to_string())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        self.output += &"\t".repeat(self.indent_level);
        key.serialize(&mut **self)?;
        self.output += "\t";
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += "\n";
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.output += &"\t".repeat(self.indent_level);
        self.output += "}";
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Inner {
            foo: String,
            bar: bool,
        }

        #[derive(Serialize)]
        struct Test {
            int: u32,
            inner: Inner,
        }

        let test = Test {
            int: 1,
            inner: Inner {
                foo: "baz".to_string(),
                bar: false,
            },
        };
        let expected = concat!(
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
        assert_eq!(to_string(&test).unwrap(), expected);
    }
}
