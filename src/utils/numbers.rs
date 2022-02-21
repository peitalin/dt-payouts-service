use serde::de;
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Unexpected, Visitor};
use serde_derive::*;
use std::fmt;

///////////////////////////////////////////////////////////////////////////
/// Deserializers
///////////////////////////////////////////////////////////////////////////

pub fn deserialize_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String to f64
    struct F64Visitor;

    impl<'de> de::Visitor<'de> for F64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing f64 data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // convert to f64
            Ok(serde_json::from_str(v).unwrap())
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(F64Visitor)
}

pub fn deserialize_as_maybe_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String to f64
    struct F64Visitor;

    impl<'de> de::Visitor<'de> for F64Visitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a Option(string) containing f64 data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // convert to f64
            Ok(Some(serde_json::from_str(v).unwrap()))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None) // convert to none
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(F64Visitor)
}

pub fn deserialize_as_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: de::Deserializer<'de>,
{
    // define a visitor that deserializes String to f32
    struct F32Visitor;

    impl<'de> de::Visitor<'de> for F32Visitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing f32 data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // convert to f32
            Ok(serde_json::from_str(v).unwrap())
        }
    }
    // use our visitor to deserialize
    deserializer.deserialize_any(F32Visitor)
}

