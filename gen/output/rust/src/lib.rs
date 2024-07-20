#![allow(clippy::too_many_arguments)]

pub mod error;
// This file was code-generated using an experimental CDDL to rust tool:
// https://github.com/dcSpark/cddl-codegen

pub mod serialization;

use crate::error::*;
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct Anyy;

impl Anyy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Anyy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Int {
    Uint(u64),
    Nint(u64),
}

impl Int {
    pub fn new_uint(value: u64) -> Self {
        Self::Uint(value)
    }

    /// * `value` - Value as encoded in CBOR - note: a negative `x` here would be `|x + 1|` due to CBOR's `nint` encoding e.g. to represent -5, pass in 4.
    pub fn new_nint(value: u64) -> Self {
        Self::Nint(value)
    }
}

impl std::fmt::Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uint(x) => write!(f, "{}", x),
            Self::Nint(x) => write!(f, "-{}", x + 1),
        }
    }
}

impl std::str::FromStr for Int {
    type Err = IntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = i128::from_str(s).map_err(IntError::Parsing)?;
        Self::try_from(x).map_err(IntError::Bounds)
    }
}

impl TryFrom<i128> for Int {
    type Error = std::num::TryFromIntError;

    fn try_from(x: i128) -> Result<Self, Self::Error> {
        if x >= 0 {
            u64::try_from(x).map(Self::Uint)
        } else {
            u64::try_from((x + 1).abs()).map(Self::Nint)
        }
    }
}

#[derive(Clone, Debug)]
pub enum IntError {
    Bounds(std::num::TryFromIntError),
    Parsing(std::num::ParseIntError),
}

#[derive(Clone, Debug)]
pub enum IntOrText {
    Int(Int),
    Text(String),
}

impl IntOrText {
    pub fn new_int(int: Int) -> Self {
        Self::Int(int)
    }

    pub fn new_text(text: String) -> Self {
        Self::Text(text)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Key {
    Int(Int),
    Text(String),
}

impl Key {
    pub fn new_int(int: Int) -> Self {
        Self::Int(int)
    }

    pub fn new_text(text: String) -> Self {
        Self::Text(text)
    }
}

#[derive(Clone, Debug)]
pub enum Salted {
    SaltedClaim(SaltedClaim),
    SaltedElement(SaltedElement),
}

impl Salted {
    pub fn new_salted_claim(salted_claim: SaltedClaim) -> Self {
        Self::SaltedClaim(salted_claim)
    }

    pub fn new_salted_element(salted_element: SaltedElement) -> Self {
        Self::SaltedElement(salted_element)
    }
}

pub type SaltedClaim = SaltedClaimItem;

#[derive(Clone, Debug)]
pub struct SaltedClaimItem {
    pub salt: Vec<u8>,
    pub index_1: IntOrText,
    pub value: Anyy,
}

impl SaltedClaimItem {
    pub fn new(salt: Vec<u8>, index_1: IntOrText, value: Anyy) -> Result<Self, DeserializeError> {
        if salt.len() < 16 || salt.len() > 16 {
            return Err(DeserializeFailure::RangeCheck {
                found: salt.len() as isize,
                min: Some(16),
                max: Some(16),
            }
            .into());
        }
        Ok(Self {
            salt,
            index_1,
            value,
        })
    }
}

pub type SaltedElement = SaltedElementItem;

#[derive(Clone, Debug)]
pub struct SaltedElementItem {
    pub salt: Vec<u8>,
    pub value: Anyy,
}

impl SaltedElementItem {
    pub fn new(salt: Vec<u8>, value: Anyy) -> Result<Self, DeserializeError> {
        if salt.len() < 16 || salt.len() > 16 {
            return Err(DeserializeFailure::RangeCheck {
                found: salt.len() as isize,
                min: Some(16),
                max: Some(16),
            }
            .into());
        }
        Ok(Self { salt, value })
    }
}

#[derive(Clone, Debug)]
pub struct SdCwt {
    pub protected: SdProtected,
    pub unprotected: Unprotected,
    pub payload: SdPayload,
    pub signature: Vec<u8>,
}

impl SdCwt {
    pub fn new(
        protected: SdProtected,
        unprotected: Unprotected,
        payload: SdPayload,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            protected,
            unprotected,
            payload,
            signature,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SdPayload {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: String,
    pub key_4: Option<Int>,
    pub key_5: Option<Int>,
    pub key_6: Int,
    pub cnonce: Option<Vec<u8>>,
    pub cnf: Option<BTreeMap<Key, Anyy>>,
    pub sd_hash: Option<Vec<u8>>,
    pub sd_alg: Option<Int>,
    pub redacted_keys: Option<Vec<Vec<u8>>>,
    pub custom: BTreeMap<Key, Anyy>,
}

impl SdPayload {
    pub fn new(aud: String, key_6: Int, custom: BTreeMap<Key, Anyy>) -> Self {
        Self {
            iss: None,
            sub: None,
            aud,
            key_4: None,
            key_5: None,
            key_6,
            cnonce: None,
            cnf: None,
            sd_hash: None,
            sd_alg: None,
            redacted_keys: None,
            custom,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SdProtected {
    pub alg: Int,
    pub typ: String,
    pub custom: BTreeMap<Key, Anyy>,
}

impl SdProtected {
    pub fn new(alg: Int, typ: String, custom: BTreeMap<Key, Anyy>) -> Self {
        Self { alg, typ, custom }
    }
}

#[derive(Clone, Debug)]
pub struct Unprotected {
    pub sd_claims: Option<Vec<Salted>>,
    pub sd_kbt: Option<Vec<u8>>,
    pub custom: BTreeMap<Key, Anyy>,
}

impl Unprotected {
    pub fn new(custom: BTreeMap<Key, Anyy>) -> Self {
        Self {
            sd_claims: None,
            sd_kbt: None,
            custom,
        }
    }
}
