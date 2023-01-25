use std::{fmt, marker::PhantomData, str::FromStr};

use serde::{
  de::{self, Visitor},
  Deserialize, Deserializer,
};

#[allow(dead_code)]
pub fn na_or_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
  D: Deserializer<'de>,
{
  struct NaOrI64(PhantomData<fn() -> Option<i64>>);

  impl<'de> Visitor<'de> for NaOrI64 {
    type Value = Option<i64>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("`N/A` or i64")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      if value == "N/A" {
        Ok(None)
      } else {
        Err(de::Error::invalid_type(de::Unexpected::Str(value), &self))
      }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      Ok(Some(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      match i64::try_from(v) {
        Ok(ok) => Ok(Some(ok)),
        Err(_) => Err(de::Error::invalid_type(de::Unexpected::Unsigned(v), &self)),
      }
    }
  }

  deserializer.deserialize_any(NaOrI64(PhantomData))
}

pub fn na_or_from_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
  T: Deserialize<'de> + FromStr,
  D: Deserializer<'de>,
{
  struct FromStrVisitor<T>(PhantomData<fn() -> Option<T>>);

  impl<'de, T> Visitor<'de> for FromStrVisitor<T>
  where
    T: Deserialize<'de> + FromStr,
  {
    type Value = Option<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
      f.write_str(concat!("`N/A` or FromStr<", stringify!(T), ">"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      if v == "N/A" {
        return Ok(None);
      }

      match T::from_str(v) {
        Ok(ok) => Ok(Some(ok)),
        Err(_) => Err(de::Error::invalid_type(de::Unexpected::Str(v), &self)),
      }
    }
  }

  deserializer.deserialize_string(FromStrVisitor(PhantomData))
}

pub fn de_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: Deserialize<'de> + FromStr,
  D: Deserializer<'de>,
{
  struct FromStrVisitor<T>(PhantomData<fn() -> T>);

  impl<'de, T> Visitor<'de> for FromStrVisitor<T>
  where
    T: Deserialize<'de> + FromStr,
  {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
      f.write_str(concat!("FromStr<", stringify!(T), ">"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      match T::from_str(v) {
        Ok(ok) => Ok(ok),
        Err(_) => Err(de::Error::invalid_type(de::Unexpected::Str(v), &self)),
      }
    }
  }

  deserializer.deserialize_string(FromStrVisitor(PhantomData))
}
