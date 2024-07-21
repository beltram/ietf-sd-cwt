use std::str::FromStr;

use jwt_simple::prelude::*;

use crate::error::SdCwtResult;

pub mod error;

pub struct IssuerPrivateKey(Ed25519KeyPair);

pub struct SdCwt(Vec<u8>);

impl std::fmt::Display for SdCwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

impl IssuerPrivateKey {
    pub fn generate() -> Self {
        Self(Ed25519KeyPair::generate())
    }

    pub fn sign(&self) -> SdCwtResult<SdCwt> {
        todo!()
    }
}
