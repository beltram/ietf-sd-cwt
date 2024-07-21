// This file was code-generated using an experimental CDDL to rust tool:
// https://github.com/dcSpark/cddl-codegen

use crate::serialization::{LenEncoding, StringEncoding};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct AnyyEncoding {
    pub len_encoding: LenEncoding,
    pub index_0_encoding: Option<cbor_event::Sz>,
}

#[derive(Clone, Debug, Default)]
pub struct SaltedClaimItemEncoding {
    pub len_encoding: LenEncoding,
    pub salt_encoding: StringEncoding,
}

#[derive(Clone, Debug, Default)]
pub struct SaltedElementItemEncoding {
    pub len_encoding: LenEncoding,
    pub salt_encoding: StringEncoding,
}

#[derive(Clone, Debug, Default)]
pub struct SdCwtEncoding {
    pub len_encoding: LenEncoding,
    pub tag_encoding: Option<cbor_event::Sz>,
    pub protected_bytes_encoding: StringEncoding,
    pub payload_bytes_encoding: StringEncoding,
    pub signature_encoding: StringEncoding,
}

#[derive(Clone, Debug, Default)]
pub struct SdPayloadEncoding {
    pub len_encoding: LenEncoding,
    pub orig_deser_order: Vec<usize>,
    pub iss_encoding: StringEncoding,
    pub iss_key_encoding: Option<cbor_event::Sz>,
    pub sub_encoding: StringEncoding,
    pub sub_key_encoding: Option<cbor_event::Sz>,
    pub aud_encoding: StringEncoding,
    pub aud_key_encoding: Option<cbor_event::Sz>,
    pub exp_key_encoding: Option<cbor_event::Sz>,
    pub nbf_key_encoding: Option<cbor_event::Sz>,
    pub iat_key_encoding: Option<cbor_event::Sz>,
    pub cnonce_encoding: StringEncoding,
    pub cnonce_key_encoding: Option<cbor_event::Sz>,
    pub cnf_encoding: LenEncoding,
    pub cnf_key_encoding: Option<cbor_event::Sz>,
    pub sd_hash_encoding: StringEncoding,
    pub sd_hash_key_encoding: Option<cbor_event::Sz>,
    pub sd_alg_key_encoding: Option<cbor_event::Sz>,
    pub redacted_keys_encoding: LenEncoding,
    pub redacted_keys_elem_encodings: Vec<StringEncoding>,
    pub redacted_keys_key_encoding: Option<cbor_event::Sz>,
    pub custom_encoding: LenEncoding,
    pub custom_key_encoding: StringEncoding,
}

#[derive(Clone, Debug, Default)]
pub struct SdProtectedEncoding {
    pub len_encoding: LenEncoding,
    pub orig_deser_order: Vec<usize>,
    pub alg_key_encoding: Option<cbor_event::Sz>,
    pub typ_encoding: StringEncoding,
    pub typ_key_encoding: Option<cbor_event::Sz>,
    pub custom_encoding: LenEncoding,
    pub custom_key_encoding: StringEncoding,
}

#[derive(Clone, Debug, Default)]
pub struct UnprotectedEncoding {
    pub len_encoding: LenEncoding,
    pub orig_deser_order: Vec<usize>,
    pub sd_claims_encoding: LenEncoding,
    pub sd_claims_key_encoding: Option<cbor_event::Sz>,
    pub sd_kbt_encoding: StringEncoding,
    pub sd_kbt_key_encoding: Option<cbor_event::Sz>,
    pub custom_encoding: LenEncoding,
    pub custom_key_encoding: StringEncoding,
}
