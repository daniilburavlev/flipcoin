use std::str::FromStr;

use num_bigint::BigUint;
use serde::{Deserialize, Deserializer, Serializer};

use crate::error::AppError;

pub mod error;

pub type AppResult<T> = Result<T, AppError>;

pub fn serialize_biguint<S: Serializer>(val: &BigUint, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&val.to_string())
}

pub fn deserialize_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    BigUint::from_str(&s).map_err(serde::de::Error::custom)
}
