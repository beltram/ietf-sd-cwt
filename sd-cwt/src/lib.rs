use jwt_simple::prelude::*;

use crate::error::SdCwtResult;
use crate::types::SdCwtPayload;

pub mod error;
pub mod types;
pub mod input;

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

    pub fn sign(&self, claims: SdCwtPayload) -> SdCwtResult<SdCwt> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_succeed() {
        let issuer = IssuerPrivateKey::generate();
        // let sd_cwt = issuer.sign().unwrap();
        // println!("{sd_cwt}");
    }
}