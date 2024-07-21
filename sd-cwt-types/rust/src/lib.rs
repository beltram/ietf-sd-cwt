#![allow(clippy::too_many_arguments)]

pub mod error;
pub mod ordered_hash_map;
extern crate derivative;
// This file was code-generated using an experimental CDDL to rust tool:
// https://github.com/dcSpark/cddl-codegen

pub mod cbor_encodings;
pub mod serialization;

use crate::error::*;
use crate::ordered_hash_map::OrderedHashMap;
use crate::serialization::{LenEncoding, StringEncoding};
use cbor_encodings::{
    AnyyEncoding, SaltedClaimItemEncoding, SaltedElementItemEncoding, SdCwtEncoding,
    SdPayloadEncoding, SdProtectedEncoding, UnprotectedEncoding,
};
use std::collections::BTreeMap;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct Anyy {
    pub encodings: Option<AnyyEncoding>,
}

impl Anyy {
    pub fn new() -> Self {
        Self { encodings: None }
    }
}

impl Default for Anyy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, derivative::Derivative)]
#[derivative(
    Eq,
    PartialEq,
    Ord = "feature_allow_slow_enum",
    PartialOrd = "feature_allow_slow_enum",
    Hash
)]
pub enum Int {
    Uint {
        value: u64,
        #[derivative(
            PartialEq = "ignore",
            Ord = "ignore",
            PartialOrd = "ignore",
            Hash = "ignore"
        )]
        encoding: Option<cbor_event::Sz>,
    },
    Nint {
        value: u64,
        #[derivative(
            PartialEq = "ignore",
            Ord = "ignore",
            PartialOrd = "ignore",
            Hash = "ignore"
        )]
        encoding: Option<cbor_event::Sz>,
    },
}

impl Int {
    pub fn new_uint(value: u64) -> Self {
        Self::Uint {
            value,
            encoding: None,
        }
    }

    /// * `value` - Value as encoded in CBOR - note: a negative `x` here would be `|x + 1|` due to CBOR's `nint` encoding e.g. to represent -5, pass in 4.
    pub fn new_nint(value: u64) -> Self {
        Self::Nint {
            value,
            encoding: None,
        }
    }
}

impl std::fmt::Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uint { value, .. } => write!(f, "{}", value),
            Self::Nint { value, .. } => write!(f, "-{}", value + 1),
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
            u64::try_from(x).map(|x| Self::Uint {
                value: x,
                encoding: None,
            })
        } else {
            u64::try_from((x + 1).abs()).map(|x| Self::Nint {
                value: x,
                encoding: None,
            })
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
    Text {
        text: String,
        text_encoding: StringEncoding,
    },
}

impl IntOrText {
    pub fn new_int(int: Int) -> Self {
        Self::Int(int)
    }

    pub fn new_text(text: String) -> Self {
        Self::Text {
            text,
            text_encoding: StringEncoding::default(),
        }
    }
}

#[derive(Clone, Debug, derivative::Derivative)]
#[derivative(
    Eq,
    PartialEq,
    Ord = "feature_allow_slow_enum",
    PartialOrd = "feature_allow_slow_enum",
    Hash
)]
pub enum Keyy {
    Int(Int),
    Text {
        text: String,
        #[derivative(
            PartialEq = "ignore",
            Ord = "ignore",
            PartialOrd = "ignore",
            Hash = "ignore"
        )]
        text_encoding: StringEncoding,
    },
}

impl Keyy {
    pub fn new_int(int: Int) -> Self {
        Self::Int(int)
    }

    pub fn new_text(text: String) -> Self {
        Self::Text {
            text,
            text_encoding: StringEncoding::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Salted {
    SaltedClaim {
        salted_claim: SaltedClaim,
        salted_claim_bytes_encoding: StringEncoding,
    },
    SaltedElement {
        salted_element: SaltedElement,
        salted_element_bytes_encoding: StringEncoding,
    },
}

impl Salted {
    pub fn new_salted_claim(salted_claim: SaltedClaim) -> Self {
        Self::SaltedClaim {
            salted_claim,
            salted_claim_bytes_encoding: StringEncoding::default(),
        }
    }

    pub fn new_salted_element(salted_element: SaltedElement) -> Self {
        Self::SaltedElement {
            salted_element,
            salted_element_bytes_encoding: StringEncoding::default(),
        }
    }
}

pub type SaltedClaim = SaltedClaimItem;

#[derive(Clone, Debug)]
pub struct SaltedClaimItem {
    pub salt: Vec<u8>,
    pub index_1: IntOrText,
    pub value: Anyy,
    pub encodings: Option<SaltedClaimItemEncoding>,
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
            encodings: None,
        })
    }
}

pub type SaltedElement = SaltedElementItem;

#[derive(Clone, Debug)]
pub struct SaltedElementItem {
    pub salt: Vec<u8>,
    pub value: Anyy,
    pub encodings: Option<SaltedElementItemEncoding>,
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
        Ok(Self {
            salt,
            value,
            encodings: None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SdCwt {
    pub protected: SdProtected,
    pub unprotected: Unprotected,
    pub payload: SdPayload,
    pub signature: Vec<u8>,
    pub encodings: Option<SdCwtEncoding>,
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
            encodings: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SdPayload {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: String,
    pub exp: Option<Int>,
    pub nbf: Option<Int>,
    pub iat: Int,
    pub cnonce: Option<Vec<u8>>,
    pub cnf: Option<OrderedHashMap<Keyy, Anyy>>,
    pub sd_hash: Option<Vec<u8>>,
    pub sd_alg: Option<Int>,
    pub redacted_keys: Option<Vec<Vec<u8>>>,
    pub custom: OrderedHashMap<Keyy, Anyy>,
    pub encodings: Option<SdPayloadEncoding>,
}

impl SdPayload {
    pub fn new(aud: String, iat: Int, custom: OrderedHashMap<Keyy, Anyy>) -> Self {
        Self {
            iss: None,
            sub: None,
            aud,
            exp: None,
            nbf: None,
            iat,
            cnonce: None,
            cnf: None,
            sd_hash: None,
            sd_alg: None,
            redacted_keys: None,
            custom,
            encodings: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SdProtected {
    pub alg: Int,
    pub typ: String,
    pub custom: OrderedHashMap<Keyy, Anyy>,
    pub encodings: Option<SdProtectedEncoding>,
}

impl SdProtected {
    pub fn new(alg: Int, typ: String, custom: OrderedHashMap<Keyy, Anyy>) -> Self {
        Self {
            alg,
            typ,
            custom,
            encodings: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Unprotected {
    pub sd_claims: Option<Vec<Salted>>,
    pub sd_kbt: Option<Vec<u8>>,
    pub custom: OrderedHashMap<Keyy, Anyy>,
    pub encodings: Option<UnprotectedEncoding>,
}

impl Unprotected {
    pub fn new(custom: OrderedHashMap<Keyy, Anyy>) -> Self {
        Self {
            sd_claims: None,
            sd_kbt: None,
            custom,
            encodings: None,
        }
    }
}
