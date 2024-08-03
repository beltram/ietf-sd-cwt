//! See https://www.ietf.org/archive/id/draft-prorock-spice-cose-sd-cwt-01.html

use std::str::FromStr;

use jwt_simple::prelude::*;

use crate::error::SdCwtResult;

pub mod error;
pub mod input;

pub struct IssuerPrivateKey(Ed25519KeyPair);

impl IssuerPrivateKey {
    pub fn generate() -> Self {
        Self(Ed25519KeyPair::generate())
    }

    pub fn sign(&self) -> SdCwtResult<SdCwt> {
        todo!()
    }
}

pub struct SdCwt(Vec<u8>);

impl std::fmt::Display for SdCwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}
