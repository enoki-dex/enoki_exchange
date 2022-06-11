use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::types::StableNat;

impl Serialize for StableNat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for StableNat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct StableNatVisitor;

        impl<'de> Visitor<'de> for StableNatVisitor {
            type Value = StableNat;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("StableNat")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(StableNat(v.parse().unwrap()))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(StableNat(v.parse().unwrap()))
            }
        }

        deserializer.deserialize_str(StableNatVisitor)
    }
}
